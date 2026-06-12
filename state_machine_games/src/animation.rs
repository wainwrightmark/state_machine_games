use std::marker::PhantomData;

use crate::prelude::*;
use glam::{FloatExt, Vec2, f32};

/// A set of animations to be run in parallel
pub struct AnimationList<T: GameArtifact> {
    pub inner: Vec<AnimationStage<T>>,
}

impl<T: GameArtifact> Default for AnimationList<T> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<T: GameArtifact> AnimationList<T> {
    pub const EMPTY: Self = Self { inner: vec![] };

    pub fn new_unfinished(
        artifact: &T,
        animations: impl IntoIterator<Item = AnimationStage<T>>,
    ) -> Self {
        let inner = animations
            .into_iter()
            .filter(|x| !x.is_finished(artifact))
            .collect();

        Self { inner }
    }

    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn push(&mut self, t: AnimationStage<T>) {
        self.inner.push(t);
    }

    pub fn retain_unfinished(&mut self, artifact: &T) {
        self.inner.retain(|x| !x.is_finished(artifact));
    }
}

impl<T: GameArtifact> From<Vec<AnimationStage<T>>> for AnimationList<T> {
    fn from(inner: Vec<AnimationStage<T>>) -> Self {
        Self { inner }
    }
}

pub struct AnimationStage<TArtifact: GameArtifact> {
    pub current: Box<dyn Animation<TArtifact>>,
    pub next: Option<Box<Self>>,
    pub loop_next: bool,
}

impl<TArtifact: GameArtifact> AnimationStage<TArtifact> {
    pub fn is_finished(&self, artifact: &TArtifact) -> bool {
        let mut stage = self;
        loop {
            if stage.loop_next {
                return false;
            };

            if !stage.current.is_finished(artifact) {
                return false;
            }
            match &stage.next {
                Some(n) => stage = n.as_ref(),
                None => {
                    return true;
                }
            }
        }
    }
}

impl<T: GameArtifact> Clone for AnimationStage<T> {
    fn clone(&self) -> Self {
        Self {
            current: self.current.box_clone(),
            next: self.next.clone(),
            loop_next: self.loop_next,
        }
    }
}

// pub fn animate_angle<L: GetValueLens + UpdateLens<Value: SpiralValue, Object: GameArtifact>>(
//     mut target_radians: f32,
//     center: L::Value,
//     velocity_radians_per_ms: f64,
// ) -> AnimateRadial<L::Object, L::Value, L> {
//     while target_radians > std::f32::consts::PI {
//         target_radians -= std::f32::consts::TAU
//     }
//     AnimateRadial::<L::Object, L::Value, L> {
//         target_radians,
//         center,
//         velocity_radians_per_ms,
//         phantom: PhantomData,
//     }
// }

#[derive(Debug, Clone)]
pub struct AnimateWait {
    duration_ms_remaining: f64,
}

impl<TArtifact: GameArtifact> Animation<TArtifact> for AnimateWait {
    fn step(&mut self, _object: &mut TArtifact, delta_ms: f64) -> AnimateResult {
        self.duration_ms_remaining -= delta_ms;
        if self.duration_ms_remaining <= 0.0 {
            self.duration_ms_remaining = 0.0;
            AnimateResult::FinishStep
        } else {
            AnimateResult::Continue
        }
    }

    fn is_finished(&self, _target: &TArtifact) -> bool {
        self.duration_ms_remaining <= 0.0
    }

    fn box_clone(&self) -> Box<dyn Animation<TArtifact>> {
        let c: AnimateWait = self.clone();
        Box::new(c)
    }
}

pub fn animate_wait(duration_ms: f64) -> AnimateWait {
    AnimateWait {
        duration_ms_remaining: duration_ms,
    }
}

pub fn animate_set_value<
    L: GetValueLens + SetValueLens<Value: ApproachValue, Object: GameArtifact>,
>(
    target_value: L::Value,
) -> AnimateSetValue<L::Object, L::Value, L> {
    AnimateSetValue {
        target_value,
        phantom: Default::default(),
    }
}

pub fn animate_towards<L: GetValueLens + UpdateLens<Value: ApproachValue, Object: GameArtifact>>(
    target_value: L::Value,
    velocity_units_per_ms: f64,
) -> AnimateMoveTowards<L::Object, L::Value, L> {
    AnimateMoveTowards::<L::Object, L::Value, L> {
        target_value,
        velocity_units_per_ms,
        phantom: Default::default(),
    }
}

pub fn animate_spring<L: GetValueLens + UpdateLens<Value: ApproachValue + DifferenceLogScaling, Object: GameArtifact>>(
    target_value: L::Value,
    base_velocity_units_per_ms: f64,
    log_scaling: f64
)-> AnimateSpringTowards<L::Object, L::Value, L>{
    AnimateSpringTowards { target_value, base_velocity_units_per_ms, log_scaling, phantom: PhantomData }
}

impl<TArtifact: GameArtifact> AnimationStage<TArtifact> {
    fn continue_with(&mut self, new_stage: Box<Self>, set_loop: bool) {
        //todo find a way to avoid tail recursion
        if let Some(next) = self.next.as_mut() {
            next.as_mut().continue_with(new_stage, set_loop);
        } else {
            self.loop_next = set_loop;
            self.next = Some(new_stage)
        }
    }

    pub fn loop_forever(&mut self) {
        let clone = self.clone();
        self.continue_with(Box::new(clone), true);
    }

    /// Step through this animation, proceeding to next stages when necessary
    pub fn step(&mut self, object: &mut TArtifact, delta_ms: f64) -> AnimateResult {
        match self.current.step(object, delta_ms) {
            AnimateResult::Continue => {
                return AnimateResult::Continue;
            }
            AnimateResult::FinishStep => {
                let Some(mut next) = self.next.take() else {
                    //leptos::logging::log!("Animation Stage Finished {}",  std::any::type_name::<TArtifact>());
                    return AnimateResult::FinishStep;
                };

                if self.loop_next {
                    next.loop_forever();
                }

                *self = *next;
                return AnimateResult::Continue;
            }
        }
    }

    /// Returns the predecessor
    pub fn precede_with(self, previous: impl Animation<TArtifact>) -> Self {
        let next = Some(Box::new(self));
        let current = Box::new(previous);

        let new_stage = AnimationStage {
            next,
            current,
            loop_next: false,
        };
        new_stage
    }
}

pub trait Animation<TArtifact: GameArtifact>: Send + Sync + 'static {
    fn step(&mut self, object: &mut TArtifact, delta_ms: f64) -> AnimateResult;

    fn is_finished(&self, target: &TArtifact) -> bool;

    fn box_clone(&self) -> Box<dyn Animation<TArtifact>>;

    fn to_stage(self) -> AnimationStage<TArtifact>
    where
        Self: Sized,
    {
        AnimationStage {
            current: Box::new(self),
            next: None,
            loop_next: false,
        }
    }
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

// #[derive(Debug)]
// pub struct AnimateRadial<
//     T: GameArtifact,
//     V: SpiralValue,
//     Lens: GetValueLens + UpdateLens<Object = T, Value = V>,
// > {
//     target_radians: f32,
//     center: V,
//     velocity_radians_per_ms: f64,
//     phantom: PhantomData<Lens>,
// }

// impl<T: GameArtifact, V: SpiralValue, Lens: GetValueLens + UpdateLens<Object = T, Value = V>>
//     Animation<T> for AnimateRadial<T, V, Lens>
// {
//     fn step(&self, object: &mut T, delta_ms: f64) -> AnimateResult {
//         let mut result: AnimateResult = AnimateResult::FinishStep;
//         Lens::update(object, |current_value| {
//             let mut current_radians = V::get_radians(current_value, &self.center);
//             let current_distance = V::get_distance_from(current_value, &self.center);

//             let mut adj_target_radians = self.target_radians;
//             if self.velocity_radians_per_ms >= 0.0{
//                 current_radians += std::f32::consts::TAU;
//                 adj_target_radians += std::f32::consts::TAU;
//             }else{
//                 current_radians -= std::f32::consts::TAU;
//                 adj_target_radians -= std::f32::consts::TAU;
//             }

//             let p = current_radians;

//             if f32::approach(
//                 &mut current_radians,
//                 &adj_target_radians,
//                 self.velocity_radians_per_ms,
//                 delta_ms,
//             ) {
//                 result = AnimateResult::FinishStep;
//             } else {
//                 result = AnimateResult::Continue;
//             }

//             *current_value =
//                 V::from_radians_and_distance(current_radians, current_distance, &self.center);

//             leptos::logging::log!("prev {p} target {adj_target_radians} new {current_radians}  new calc {}", V::get_radians(current_value, &self.center) + std::f32::consts::TAU);
//         });

//         result
//     }

//     fn is_finished(&self, target: &T) -> bool {
//         self.target_radians == V::get_radians(&Lens::get_value(target), &self.center)
//     }

//     fn box_clone(&self) -> Box<dyn Animation<T>> {
//         let clone: AnimateRadial<T, V, Lens> = self.clone();
//         Box::new(clone)
//     }
// }

// impl<T: GameArtifact, V: SpiralValue, Lens: GetValueLens + UpdateLens<Object = T, Value = V>> Clone
//     for AnimateRadial<T, V, Lens>
// {
//     fn clone(&self) -> Self {
//         Self {
//             target_radians: self.target_radians.clone(),
//             center: self.center.clone(),
//             velocity_radians_per_ms: self.velocity_radians_per_ms.clone(),
//             phantom: self.phantom.clone(),
//         }
//     }
// }

#[derive(Debug)]
pub struct AnimateSetValue<
    T: GameArtifact,
    V: Send + Sync + 'static + Clone + PartialEq,
    Lens: SetValueLens<Object = T, Value = V>,
> {
    target_value: V,
    phantom: PhantomData<Lens>,
}

impl<T: GameArtifact, V: ApproachValue, Lens: SetValueLens<Object = T, Value = V>> Clone
    for AnimateSetValue<T, V, Lens>
{
    fn clone(&self) -> Self {
        Self {
            target_value: self.target_value.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<
    T: GameArtifact,
    V: ApproachValue + Clone + PartialEq,
    Lens: GetValueLens + SetValueLens<Object = T, Value = V>,
> Animation<T> for AnimateSetValue<T, V, Lens>
{
    fn step(&mut self, object: &mut T, _delta_ms: f64) -> AnimateResult {
        Lens::set(object, self.target_value.clone());
        AnimateResult::FinishStep
    }

    fn is_finished(&self, target: &T) -> bool {
        self.target_value == Lens::get_value(target)
    }

    fn box_clone(&self) -> Box<dyn Animation<T>> {
        let clone: AnimateSetValue<T, V, Lens> = self.clone();
        Box::new(clone)
    }
}

pub trait DifferenceLogScaling {
    fn calculate_scaled_velocity(
        current_value: &Self,
        target_value: &Self,
        log_scaling: f64,
        base_velocity_units_per_ms: f64,
    ) -> f64;
}

impl DifferenceLogScaling for f64 {
    fn calculate_scaled_velocity(
        current_value: &Self,
        target_value: &Self,
        log_scaling: f64,
        base_velocity_units_per_ms: f64,
    ) -> f64 {
        let distance = (*current_value - target_value).abs();
        let power = (distance / log_scaling).max(1.0) as f64;
        let scale = 2.0f64.powf(power);
        let velocity = base_velocity_units_per_ms * scale;

        velocity
    }
}

impl DifferenceLogScaling for f32 {
    fn calculate_scaled_velocity(
        current_value: &Self,
        target_value: &Self,
        log_scaling: f64,
        base_velocity_units_per_ms: f64,
    ) -> f64 {
        let distance = (*current_value - *target_value).abs();
        let power = (distance as f64 / log_scaling).max(1.0) as f64;
        let scale = 2.0f64.powf(power);
        let velocity = base_velocity_units_per_ms as f64 * scale;

        velocity
    }
}

#[derive(Debug)]
pub struct AnimateSpringTowards<
    T: GameArtifact,
    V: ApproachValue + DifferenceLogScaling,
    Lens: UpdateLens<Object = T, Value = V>,
> {
    target_value: V,
    base_velocity_units_per_ms: f64,
    /// if the current distance is x, the velocity is:
    /// base_velocity * 2^max(1.0, x/log_scaling)
    log_scaling: f64,
    phantom: PhantomData<Lens>,
}

impl<
    T: GameArtifact,
    V: ApproachValue + DifferenceLogScaling,
    Lens: UpdateLens<Object = T, Value = V>,
> Clone for AnimateSpringTowards<T, V, Lens>
{
    fn clone(&self) -> Self {
        Self {
            target_value: self.target_value.clone(),
            base_velocity_units_per_ms: self.base_velocity_units_per_ms,
            log_scaling: self.log_scaling,
            phantom: self.phantom.clone(),
        }
    }
}

impl<
    T: GameArtifact,
    V: ApproachValue + DifferenceLogScaling,
    Lens: GetValueLens + UpdateLens<Object = T, Value = V>,
> Animation<T> for AnimateSpringTowards<T, V, Lens>
{
    fn step(&mut self, object: &mut T, delta_ms: f64) -> AnimateResult {
        let mut result: AnimateResult = AnimateResult::FinishStep;
        Lens::update(object, |current_value| {
            let velocity = V::calculate_scaled_velocity(
                current_value,
                &self.target_value,
                self.log_scaling,
                self.base_velocity_units_per_ms,
            );

            if V::approach(current_value, &self.target_value, velocity, delta_ms) {
                result = AnimateResult::FinishStep;
            } else {
                result = AnimateResult::Continue;
            }
        });

        result
    }

    fn is_finished(&self, target: &T) -> bool {
        self.target_value == Lens::get_value(target)
    }

    fn box_clone(&self) -> Box<dyn Animation<T>> {
        let clone: AnimateSpringTowards<T, V, Lens> = self.clone();
        Box::new(clone)
    }
}

#[derive(Debug)]
pub struct AnimateMoveTowards<
    T: GameArtifact,
    V: ApproachValue,
    Lens: UpdateLens<Object = T, Value = V>,
> {
    target_value: V,
    velocity_units_per_ms: f64,
    phantom: PhantomData<Lens>,
}

impl<T: GameArtifact, V: ApproachValue, Lens: UpdateLens<Object = T, Value = V>> Clone
    for AnimateMoveTowards<T, V, Lens>
{
    fn clone(&self) -> Self {
        Self {
            target_value: self.target_value.clone(),
            velocity_units_per_ms: self.velocity_units_per_ms.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<
    T: GameArtifact,
    V: ApproachValue + Clone + PartialEq,
    Lens: GetValueLens + UpdateLens<Object = T, Value = V>,
> Animation<T> for AnimateMoveTowards<T, V, Lens>
{
    fn step(&mut self, object: &mut T, delta_ms: f64) -> AnimateResult {
        let mut result: AnimateResult = AnimateResult::FinishStep;
        Lens::update(object, |current_value| {
            if V::approach(
                current_value,
                &self.target_value,
                self.velocity_units_per_ms,
                delta_ms,
            ) {
                result = AnimateResult::FinishStep;
            } else {
                result = AnimateResult::Continue;
            }
        });

        result
    }

    fn is_finished(&self, target: &T) -> bool {
        self.target_value == Lens::get_value(target)
    }

    fn box_clone(&self) -> Box<dyn Animation<T>> {
        let clone: AnimateMoveTowards<T, V, Lens> = self.clone();
        Box::new(clone)
    }
}

pub trait ApproachValue: Send + Sync + 'static + Clone + PartialEq {
    ///Returns `true` if the value has now reached the target
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: f64,
        delta_ms: f64,
    ) -> bool;
}

#[cfg(feature = "bevy_color")]
impl ApproachValue for bevy_color::Srgba {
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: f64,
        delta_ms: f64,
    ) -> bool {
        use bevy_color::ColorToComponents;
        let v4 = value.to_vec4();
        let v4 = v4.move_towards(
            target_value.to_vec4(),
            (velocity_units_per_ms * delta_ms) as f32,
        );
        *value = bevy_color::Srgba::from_vec4(v4);
        value == target_value
    }
}

impl ApproachValue for Vec2 {
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: f64,
        delta_ms: f64,
    ) -> bool {
        *value = value.move_towards(*target_value, (velocity_units_per_ms * delta_ms) as f32);
        value == target_value
    }
}

impl ApproachValue for f64 {
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: f64,
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
        velocity_units_per_ms: f64,
        delta_ms: f64,
    ) -> bool {
        //log!("Animating {value} towards {target_value} at {velocity_units_per_ms}");
        match value.total_cmp(target_value) {
            std::cmp::Ordering::Less => {
                *value += (velocity_units_per_ms * delta_ms) as f32;
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
                *value -= (velocity_units_per_ms * delta_ms) as f32;
                if *value <= *target_value {
                    *value = *target_value;
                    return true;
                }
                return false;
            }
        }
    }
}

pub trait SpiralValue: ApproachValue {
    fn get_radians(&self, center: &Self) -> f32;
    fn get_distance_from(&self, center: &Self) -> f32;
    fn from_radians_and_distance(radians: f32, distance: f32, center: &Self) -> Self;
}

impl SpiralValue for Vec2 {
    fn from_radians_and_distance(radians: f32, distance: f32, center: &Self) -> Self {
        center + (Vec2::from_angle(radians) * distance)
    }

    fn get_distance_from(&self, center: &Self) -> f32 {
        Vec2::distance(*self, *center)
    }

    fn get_radians(&self, center: &Self) -> f32 {
        center.angle_to(*self)
    }
}
