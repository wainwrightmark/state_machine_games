use std::marker::PhantomData;

use crate::prelude::*;
use glam::{FloatExt, f32};

pub type AnimationList<T> = Vec<Box<dyn Animation<T>>>;

pub fn animate_towards<L: UpdateLens<Value: ApproachValue>>(
    target_value: L::Value,
    velocity_units_per_ms: L::Value,
) -> Box<dyn Animation<L::Object>> {
    let animation = AnimateMoveTowards::<L::Object, L::Value, L> {
        target_value,
        velocity_units_per_ms,
        phantom: Default::default(),
    };

    Box::new(animation)
}

pub trait Animation<TArtifact>: Send + Sync + 'static {
    fn step(&self, target: &mut TArtifact, delta_ms: f64) -> AnimateResult;
}

pub trait EaseFunction<V>: Clone + Copy + Send + Sync + 'static {
    fn ease(self, from: V, to: V, ratio: f64) -> V;
}

#[derive(Debug, Clone, Copy)]
pub struct Lerp32;

impl EaseFunction<f32> for Lerp32 {
    fn ease(self, from: f32, to: f32, ratio: f64) -> f32 {
        from.lerp(to, ratio as f32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Lerp64;

impl EaseFunction<f64> for Lerp64 {
    fn ease(self, from: f64, to: f64, ratio: f64) -> f64 {
        from.lerp(to, ratio)
    }
}

#[derive(Debug)]
pub struct AnimateMoveTowards<T, V: ApproachValue, Lens: UpdateLens<Object = T, Value = V>> {
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
        match value.total_cmp(target_value) {
            std::cmp::Ordering::Less => {
                *value += velocity_units_per_ms * delta_ms;
                if *value >= *target_value {
                    *value = *target_value;
                    return true;
                }
                return false;
            }
            std::cmp::Ordering::Equal => {
                return true;
            }
            std::cmp::Ordering::Greater => {
                *value -= velocity_units_per_ms * delta_ms;
                if *value <= *target_value {
                    *value = *target_value;
                    return true;
                }
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
        match value.total_cmp(target_value) {
            std::cmp::Ordering::Less => {
                *value += velocity_units_per_ms * delta_ms as f32;
                if *value >= *target_value {
                    *value = *target_value;
                    return true;
                }
                return false;
            }
            std::cmp::Ordering::Equal => {
                return true;
            }
            std::cmp::Ordering::Greater => {
                *value -= velocity_units_per_ms * delta_ms as f32;
                if *value <= *target_value {
                    *value = *target_value;
                    return true;
                }
                return false;
            }
        }
    }
}

impl<T: 'static, V: ApproachValue, Lens: UpdateLens<Object = T, Value = V>> Animation<T>
    for AnimateMoveTowards<T, V, Lens>
{
    fn step(&self, object: &mut T, delta_ms: f64) -> AnimateResult {
        let mut result: AnimateResult = AnimateResult::DeleteAnimation;
        Lens::update(object, |current_value| {
            if V::approach(
                current_value,
                &self.target_value,
                &self.velocity_units_per_ms,
                delta_ms,
            ) {
                result = AnimateResult::DeleteAnimation;
            } else {
                result = AnimateResult::Continue;
            }
        });

        result
    }
}
