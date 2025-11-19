use std::{marker::PhantomData};

use strum::EnumIs;

use crate::{game_state::GameState, lens::GetMutLens, prelude::GameEntityKey};

pub type AnimationList<T> = Vec<Box<dyn Animation<T>>>;

pub trait GameArtifact : Send + Sync + 'static{}

pub trait GameEntity: PartialEq + Send + Sync + 'static {
    type GameState: GameState<Entity = Self>;
    type Artifact : GameArtifact;
    type EntityKey: GameEntityKey;

    fn key(&self) -> Self::EntityKey;

    /// animations to run when this entity dies
    /// the entity will not be deleted until all animations have finished
    fn on_death(artifact: &mut Self::Artifact, previous_template: &Self)-> AnimationList<Self::Artifact>;

    /// what to do when this entity is new
    fn on_new(&self)-> (Self::Artifact, AnimationList<Self::Artifact>);

    fn on_update(&self, artifact: &mut Self::Artifact, former_entity_state: EntityState) -> AnimationList<Self::Artifact>;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, EnumIs)]
pub enum EntityState{
    Alive,
    Dead
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, EnumIs)]
pub enum AnimateResult {
    Continue,
    DeleteAnimation,
}

pub trait Animation<T> : Send + Sync + 'static {
    fn step(&mut self, target: &mut T, delta_ms: f64) -> AnimateResult;
}

//todo easing
#[derive(Debug)]
pub struct AnimateMoveTowards<T, V: ApproachValue, Lens: GetMutLens<Object = T, Value = V>> {
    pub target_value: V,
    pub velocity_units_per_ms: V,
    pub phantom: PhantomData<Lens>,
}

pub trait ApproachValue: Send + Sync + 'static {
    ///Returns `true` if the value has now reached the target
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: &Self,
        delta_ms: f64,
    ) -> bool;
}

impl ApproachValue for f64 {
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: &Self,
        delta_ms: f64,
    ) -> bool {
        *value += velocity_units_per_ms * delta_ms;

        match (
            value.total_cmp(target_value),
            velocity_units_per_ms.is_sign_positive(),
        ) {
            (std::cmp::Ordering::Equal, ..)
            | (std::cmp::Ordering::Greater, true)
            | (std::cmp::Ordering::Less, false) => {
                *value = *target_value;
                return true;
            }

            _ => {
                return false;
            }
        }
    }
}

impl ApproachValue for f32 {
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: &Self,
        delta_ms: f64,
    ) -> bool {
        *value += velocity_units_per_ms * delta_ms as f32;

        match (
            value.total_cmp(target_value),
            velocity_units_per_ms.is_sign_positive(),
        ) {
            (std::cmp::Ordering::Equal, ..)
            | (std::cmp::Ordering::Greater, true)
            | (std::cmp::Ordering::Less, false) => {
                *value = *target_value;
                return true;
            }

            _ => {
                return false;
            }
        }
    }
}

impl<T : 'static, V: ApproachValue, Lens: GetMutLens<Object = T, Value = V>> Animation<T>
    for AnimateMoveTowards<T, V, Lens>
{
    fn step(&mut self, object: &mut T, delta_ms: f64) -> AnimateResult {
        let current_value = Lens::get_mut(object);

        if V::approach(
            current_value,
            &self.target_value,
            &self.velocity_units_per_ms,
            delta_ms,
        ) {
            AnimateResult::DeleteAnimation
        } else {
            AnimateResult::Continue
        }
    }
}
