//! Support for creating futures that represent intervals.
//!
//! This module contains the `Interval` type which is a stream that will
//! resolve at a fixed intervals in future

use std::io;
use std::time::{Duration, Instant};

use futures::{Poll, Async, Future};
use futures::stream::{Stream};

use tokio_core::reactor::{Remote, Handle, Timeout};

/// A stream representing notifications at fixed interval
///
/// Intervals are created through the `Interval::new` or
/// `Interval::new_at` methods indicating when a first notification
/// should be triggered and when it will be repeated.
///
/// Note that timeouts are not intended for high resolution timers, but rather
/// they will likely fire some granularity after the exact instant that they're
/// otherwise indicated to fire at.
pub struct Interval {
    next: Instant,
    interval: Duration,
    timeout: Timeout,
    handle: Handle,
}

impl Interval {
    /// Creates a new interval which will fire at `dur` time into the future,
    /// and will repeat every `dur` interval after
    ///
    /// This function will return a future that will resolve to the actual
    /// interval object. The interval object itself is then a stream which will
    /// be set to fire at the specified intervals
    pub fn new(dur: Duration, handle: &Handle) -> io::Result<Interval> {
        Interval::new_at(Instant::now() + dur, dur, handle)
    }

    /// Creates a new interval which will fire at the time specified by `at`,
    /// and then will repeat every `dur` interval after
    ///
    /// This function will return a future that will resolve to the actual
    /// timeout object. The timeout object itself is then a future which will be
    /// set to fire at the specified point in the future.
    pub fn new_at(at: Instant, dur: Duration, handle: &Handle)
        -> io::Result<Interval>
    {
        Ok(Interval {
            next: at,
            interval: dur,
            timeout: Timeout::new_at(at, handle).unwrap(),
            handle: handle.clone(),
        })
    }
}

impl Stream for Interval {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<()>, io::Error> {
        self.timeout.poll().map(|a| {
            match a {
                Async::NotReady => Async::NotReady,
                Async::Ready(_) => {
                    let now = Instant::now();
                    self.next = next_interval(self.next, now, self.interval);
                    self.timeout = Timeout::new_at(self.next, &self.handle).unwrap();
                    Async::Ready(Some(()))
                }
            }
        })
    }
}

/// Converts Duration object to raw nanoseconds if possible
///
/// This is useful to divide intervals.
///
/// While technically for large duration it's impossible to represent any
/// duration as nanoseconds, the largest duration we can represent is about
/// 427_000 years. Large enough for any interval we would use or calculate in
/// tokio.
fn duration_to_nanos(dur: Duration) -> Option<u64> {
    dur.as_secs()
        .checked_mul(1_000_000_000)
        .and_then(|v| v.checked_add(dur.subsec_nanos() as u64))
}

fn next_interval(prev: Instant, now: Instant, interval: Duration) -> Instant {
    let new = prev + interval;
    if new > now {
        return new;
    } else {
        let spent_ns = duration_to_nanos(now.duration_since(prev))
            .expect("interval should be expired");
        let interval_ns = duration_to_nanos(interval)
            .expect("interval is less that 427 thousand years");
        let mult = spent_ns/interval_ns + 1;
        assert!(mult < (1 << 32),
            "can't skip more than 4 billion intervals of {:?} \
             (trying to skip {})", interval, mult);
        return prev + interval * (mult as u32);
    }
}
