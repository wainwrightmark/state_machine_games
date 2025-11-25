use crate::prelude::*;

//todo split state into components and track each individually
pub trait GameEntity: PartialEq + Send + Sync + 'static + Sized {
    type Artifact: GameArtifact;
    type Key: GameEntityKey;
    type GameState: GameState;

    fn key(&self) -> Self::Key;

    //todo give the game state the opportunity to promise that the entities are sorted
    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self>;

    //todo reuse the same vec for all the animations

    #[allow(unused)]
    /// animations to run when this entity dies
    /// the entity will not be deleted until all animations have finished
    fn on_death(&self, artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    /// what to do when this entity is new
    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>);

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact>;
}
