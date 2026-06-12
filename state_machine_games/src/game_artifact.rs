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

use leptos::prelude::*;

use crate::prelude::{AnimationList, AnimationStage};

// pub trait LeptosGameArtifact: GameArtifact {
//     fn render(self) -> impl IntoView;
// }

pub trait LeptosRender{
    type Artifact: GameArtifact;

    fn render(artifact: Self::Artifact)-> impl IntoView + 'static;
}
