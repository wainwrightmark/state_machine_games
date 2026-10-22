use strum::EnumIs;


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, EnumIs)]
pub enum EntityLifecycle {
    Alive,
    Dead,
    Reset
}