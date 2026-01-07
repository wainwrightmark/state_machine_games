pub trait GameArtifact: Clone + Send + Sync + 'static + Sized {
    fn with_animations(
        self,
        animations: impl IntoIterator<Item = AnimationStage<Self>>,
    ) -> (Self, AnimationList<Self>) {
        let list = AnimationList::new_unfinished(&self, animations);
        (self, list)
    }

    fn update_animations(
        &self,
        animations: impl IntoIterator<Item = AnimationStage<Self>>,
    ) -> AnimationList<Self> {
        AnimationList::new_unfinished(&self, animations)
    }
}

#[cfg(feature = "leptos")]
use leptos::prelude::*;

use crate::prelude::{AnimationList, AnimationStage};

#[cfg(feature = "leptos")]
pub trait LeptosGameArtifact<Argument: Clone + Send + Sync + 'static = ()>: GameArtifact {
    type Command: Send + 'static;
    fn render(
        self,
        argument: Argument,
        sender: impl crate::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView;
}
