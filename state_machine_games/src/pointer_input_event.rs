use leptos::{prelude::{Get, NodeRef}, svg::Svg};
use strum::EnumIs;
use web_sys::{PointerEvent, SvgGraphicsElement, wasm_bindgen::JsCast};

#[derive(Debug, Clone, PartialEq, Copy, EnumIs)]
pub enum PointerInputEvent {
    Start(Location),
    Move(Location),
    End(Option<Location>),
}

impl PointerInputEvent {
    pub fn new(t: PointerEventType, event: PointerEvent, node_ref: NodeRef<Svg>)-> Option<Self> {
        if t.is_cancel(){
            return Some(PointerInputEvent::End(None));
        }
        let location =convert_pointer_event(event, node_ref)?;

        Some(match t{
            PointerEventType::Start => PointerInputEvent::Start(location),
            PointerEventType::Move => PointerInputEvent::Move(location),
            PointerEventType::End => PointerInputEvent::End(Some(location)),
            PointerEventType::Cancel => PointerInputEvent::End(None),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, EnumIs)]
pub enum PointerEventType {
    Start,
    Move,
    End,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Location {
    pub x: f32,
    pub y: f32,
}

impl Location {
    pub fn from_event(event: PointerEvent) -> Self {
        let x = event.client_x() as f32;
        let y = event.client_y() as f32;

        Location { x, y }
    }

    pub fn apply_matrix(self, matrix: &web_sys::SvgMatrix) -> Self {
        let (a, b, c, d, e, f) = (
            matrix.a(),
            matrix.b(),
            matrix.c(),
            matrix.d(),
            matrix.e(),
            matrix.f(),
        );

        let Location{x,y} = self;
        let (x, y) = ((a * x) + (c * y) + e, (b * x) + (d * y) + f);

        Location { x, y }
    }
}

fn convert_pointer_event(event: PointerEvent, node_ref: NodeRef<Svg>) -> Option<Location> {
    let client_location = Location::from_event(event);

    let element = node_ref.get().unwrap();
    let element = element.dyn_ref::<SvgGraphicsElement>()?;

    let matrix: web_sys::SvgMatrix = element.get_screen_ctm()?;
    let inverse = matrix.inverse().ok()?;

    let new_location = client_location.apply_matrix(&inverse);

    Some(new_location)    
}
