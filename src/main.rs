#[macro_use] extern crate conrod;
#[macro_use] extern crate tokio_core;
extern crate futures;
extern crate piston_window;

use piston_window::*;

use std::thread;

mod server;
mod interval;

use std::sync::mpsc::{channel, TryRecvError};


widget_ids! {
    struct Ids {
        canvas,
        point_path,
    }
}

pub fn spirograph(components: &Vec<(f64, f64)>, v_scale: f64, r_scale: f64, n_steps: usize)
                  -> Vec<[f64; 2]>
{
    let mut phases = vec![0.0; components.len()];
    let mut out = Vec::with_capacity(n_steps);
    for _ in 0..n_steps {
        let mut x = 0.0;
        let mut y = 0.0;
        for (phase, &(speed, radius)) in phases.iter_mut().zip(components) {
            *phase += speed * v_scale;
            x += phase.cos() * radius * r_scale;
            y += phase.sin() * radius * r_scale;
        }
        out.push([x, y]);
    }
    out
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow =
        WindowSettings::new("Primitive Demo", [1280, 720])
        .opengl(opengl).samples(4).exit_on_esc(true).build().unwrap();
    window.set_ups(60);

    let mut ui = conrod::UiBuilder::new().build();
    let ids = Ids::new(ui.widget_id_generator());
    let mut text_texture_cache = conrod::backend::piston_window::GlyphCache::new(&mut window, 0, 0);
    let image_map = conrod::image::Map::new();

    let (sender, receiver) = channel::<Vec<server::ClientData>>();

    thread::spawn(|| {
        server::run(sender);
    });

    let mut params = Vec::new();

    while let Some(event) = window.next() {
        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        match receiver.try_recv() {
            Ok(res) => params = res,
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        // Update the widgets.
        event.update(|_| set_ui(&params, ui.set_widgets(), &ids));

        // Draw the `Ui`.
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });
    }
}


fn set_ui(params: &Vec<server::ClientData>, ref mut ui: conrod::UiCell, ids: &Ids) {
    use conrod::{Positionable, Widget};
    use conrod::widget::{Canvas, PointPath};

    // The background canvas upon which we'll place our widgets.
    Canvas::new().pad(80.0).set(ids.canvas, ui);

    let points = spirograph(params, 0.05, 100.0, 256);
    PointPath::centred(points).middle().set(ids.point_path, ui);
}
