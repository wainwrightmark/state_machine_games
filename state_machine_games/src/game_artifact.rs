pub trait GameArtifact: Send + Sync + 'static + Clone {}

#[cfg(feature = "leptos")]
use leptos::prelude::*;

#[cfg(feature = "leptos")]
pub trait LeptosGameArtifact<Argument: Clone + Send + Sync + 'static = ()>: GameArtifact {
    type Command: Send + 'static;
    fn render(
        self,
        argument: Argument,
        sender: impl crate::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView;
}
