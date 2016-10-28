#[macro_use] extern crate conrod;
extern crate piston_window;

use piston_window::*;

widget_ids! {
    struct Ids {
        canvas,
        line,
        point_path,
        rectangle_fill,
        rectangle_outline,
        trapezoid,
        oval_fill,
        oval_outline,
        circle,
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow =
        WindowSettings::new("Primitive Demo", [400, 720])
        .opengl(opengl).samples(4).exit_on_esc(true).build().unwrap();
    window.set_ups(60);

    let mut ui = conrod::UiBuilder::new().build();

    let ids = Ids::new(ui.widget_id_generator());

    let mut text_texture_cache = conrod::backend::piston_window::GlyphCache::new(&mut window, 0, 0);

    let image_map = conrod::image::Map::new();

    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        // Update the widgets.
        event.update(|_| set_ui(ui.set_widgets(), &ids));

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


fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids) {
    use conrod::{Positionable, Widget};
    use conrod::widget::{Canvas, Circle, Line, Oval, PointPath, Polygon, Rectangle};
    use std::iter::once;

    // The background canvas upon which we'll place our widgets.
    Canvas::new().pad(80.0).set(ids.canvas, ui);

    Line::centred([-40.0, -40.0], [40.0, 40.0]).top_left_of(ids.canvas).set(ids.line, ui);

    let left = [-40.0, -40.0];
    let top = [0.0, 40.0];
    let right = [40.0, -40.0];
    let points = once(left).chain(once(top)).chain(once(right));
    PointPath::centred(points).down(80.0).set(ids.point_path, ui);

    Rectangle::fill([80.0, 80.0]).down(80.0).set(ids.rectangle_fill, ui);

    Rectangle::outline([80.0, 80.0]).down(80.0).set(ids.rectangle_outline, ui);

    let bl = [-40.0, -40.0];
    let tl = [-20.0, 40.0];
    let tr = [20.0, 40.0];
    let br = [40.0, -40.0];
    let points = once(bl).chain(once(tl)).chain(once(tr)).chain(once(br));
    Polygon::centred_fill(points).right_from(ids.line, 80.0).set(ids.trapezoid, ui);

    Oval::fill([40.0, 80.0]).down(80.0).align_middle_x().set(ids.oval_fill, ui);

    Oval::outline([80.0, 40.0]).down(100.0).align_middle_x().set(ids.oval_outline, ui);

    Circle::fill(40.0).down(100.0).align_middle_x().set(ids.circle, ui);
}
