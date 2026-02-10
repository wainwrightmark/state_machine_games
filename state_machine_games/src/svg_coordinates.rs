
use crate::prelude::Vec2;

#[cfg(feature = "leptos")]
pub fn get_svg_coordinates(
    pointer_event: leptos::web_sys::PointerEvent,
    element: leptos::web_sys::SvgElement,
    view_box_top_left: Vec2,
    view_box_bottom_size: Vec2,
) -> Vec2 {
    let svg_element_position = element.get_bounding_client_rect();

    let svg_sizes_factor = Vec2 {
        x: view_box_bottom_size.x / (svg_element_position.width() as f32),
        y: view_box_bottom_size.y / (svg_element_position.height() as f32),
    };

    // calculates the position of the cursor relative to the svg viewbox.
    return Vec2 {
        x: (pointer_event.client_x() as f32 - svg_element_position.x() as f32) * svg_sizes_factor.x
            + view_box_top_left.x,
        y: (pointer_event.client_y() as f32 - svg_element_position.y() as f32) * svg_sizes_factor.y
            + view_box_top_left.y,
    };
}
