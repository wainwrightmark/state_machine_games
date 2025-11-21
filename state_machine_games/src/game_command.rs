pub trait GameCommand: Send + Sync + 'static + std::fmt::Debug + Clone {}

impl GameCommand for () {}