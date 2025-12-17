pub trait GameArtifact: Send + Sync + 'static + Clone {
    type Command: Send + 'static;
}

#[cfg(feature = "leptos")]
use leptos::prelude::*;

#[cfg(feature = "leptos")]
pub trait LeptosGameArtifact: GameArtifact {
    fn render(self, sender: impl crate::prelude::CommandSender<Self::Command>) -> impl IntoView;
}
