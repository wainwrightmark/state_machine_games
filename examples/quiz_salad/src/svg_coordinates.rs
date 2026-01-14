use state_machine_games::prelude::Vec2;
use web_sys::PointerEvent;

pub fn get_svg_coordinates(
    mouse_event: PointerEvent,
    element: web_sys::SvgElement,
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
        x: (mouse_event.client_x() as f32 - svg_element_position.x() as f32) * svg_sizes_factor.x
            + view_box_top_left.x,
        y: (mouse_event.client_y() as f32 - svg_element_position.y() as f32) * svg_sizes_factor.y
            + view_box_top_left.y,
    };
}

// function getMouseCoordinatesRelativelySvgElement( SVGElement, MoveMouseEvent ) {
//     // get a sizes and position of svg element, relative browser viewport (page piece, showed on a screen, which we can see)
//     let svg_element_position = SVGElement.getBoundingClientRect(),
//         // difference coefficients between HTML element and svg viewbox sizes.
//         svg_sizes_factor = {
//             x: SVGElement.viewBox.baseVal.width / svg_element_position.width,
//             y: SVGElement.viewBox.baseVal.height / svg_element_position.height
//         };

//     // calculates the position of the cursor relative to the svg viewbox.
//     return {
//         x: Math.round( (MoveMouseEvent.clientX - svg_element_position.x) * svg_sizes_factor.x + SVGElement.viewBox.baseVal.x ),
//         y: Math.round( (MoveMouseEvent.clientY - svg_element_position.y) * svg_sizes_factor.y + SVGElement.viewBox.baseVal.y )
//     };
// }
