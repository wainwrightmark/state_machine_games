use crate::prelude::*;
use leptos::prelude::*;
pub trait GameArtifact: Send + Sync + 'static + Clone {
    type Command: AnyGameCommand;
    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView;
}
