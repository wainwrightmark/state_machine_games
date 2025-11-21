#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MutationResult {
    /// Whether anything changed
    pub changed: bool,
    ///How long to wait before calling back for another transition
    pub transition_callback_in_ms: Option<f64>,
}

impl MutationResult {
    pub const NO_CHANGE: Self = Self {
        changed: false,
        transition_callback_in_ms: None,
    };

    pub const CHANGED_NO_TRANSITION: Self = Self {
        changed: true,
        transition_callback_in_ms: None,
    };
}