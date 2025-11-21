use crate::prelude::*;

pub trait GameEntity: PartialEq + Send + Sync + 'static {
    type Artifact: GameArtifact<Command = Self::Command>;
    type EntityKey: GameEntityKey;
    type Command: GameCommand;

    fn key(&self) -> Self::EntityKey;

    //todo reuse the same vec for all the animations

    /// animations to run when this entity dies
    /// the entity will not be deleted until all animations have finished
    fn on_death(&self, artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact>;

    /// what to do when this entity is new
    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>);

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact>;
}