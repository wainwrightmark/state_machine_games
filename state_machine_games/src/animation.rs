use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use crate::prelude::*;
use glam::{Vec2, f32};

pub trait AnimationCollection<T: GameArtifact>: Default + Send + Sync + 'static {
    fn is_empty(&self) -> bool;

    /// Remove all elements where the function returns false
    fn retain_mut(&mut self, f: impl FnMut(&mut AnimationStage<T>) -> bool);

    fn clear(&mut self);
}

impl<T: GameArtifact> AnimationCollection<T> for () {
    fn is_empty(&self) -> bool {
        true
    }

    fn retain_mut(&mut self, _f: impl FnMut(&mut AnimationStage<T>) -> bool) {
        ()
    }

    fn clear(&mut self) {
        ()
    }
}

/// A set of animations to be run in parallel
pub struct AnimationList<T: GameArtifact> {
    pub inner: Vec<AnimationStage<T>>,
}

impl<T: GameArtifact> AnimationCollection<T> for AnimationList<T> {
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn retain_mut(&mut self, f: impl FnMut(&mut AnimationStage<T>) -> bool) {
        self.inner.retain_mut(f);
    }

    fn clear(&mut self) {
        self.inner.clear();
    }
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
    pub fn empty() -> Self {
        Self {
            current: Box::new(AnimateEmpty),
            next: None,
            loop_next: false,
        }
    }

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

    pub fn end_value_for_lens<L: Lens<Object = TArtifact>>(&self, lens: L) -> Option<L::Value> {
        let mut end_value: Option<<L as Lens>::Value> = None;
        let mut next: Option<&Self> = Some(self);

        let type_id = Any::type_id(&lens);

        'l: while let Some(current) = next {
            if current.loop_next {
                return None;
            }

            if let Some(target_value) = current
                .current
                .as_ref()
                .target_value_if_lens(type_id)
                .and_then(|x| x.downcast_ref::<L::Value>())
            {
                end_value = Some(target_value.clone());
            }

            match &current.next {
                Some(n) => {
                    next = Some(n.as_ref());
                }
                None => break 'l,
            }
        }

        end_value
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

    fn target_value_if_lens(&self, _lens_type_id: TypeId) -> Option<&dyn Any> {
        None
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
) -> AnimateSetValue<L> {
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

pub fn animate_ease<
    Lens: GetValueLens + UpdateLens<Value: EaseValue, Object: GameArtifact>,
    F: Fn(f64) -> f64 + Clone + Send + Sync + 'static,
>(
    from: Lens::Value,
    to: Lens::Value,
    duration_ms: f64,
    _lens: Lens,
    func: F,
) -> AnimateEase<Lens, F> {
    AnimateEase {
        from,
        to,
        progress: 0.0,
        progress_per_ms: duration_ms.recip(),
        func,
    }
}

pub fn animate_spring<
    L: GetValueLens + UpdateLens<Value: ApproachValue + DifferenceLogScaling, Object: GameArtifact>,
>(
    target_value: L::Value,
    base_velocity_units_per_ms: f64,
    log_scaling: f64,
) -> AnimateSpringTowards<L> {
    AnimateSpringTowards {
        target_value,
        base_velocity_units_per_ms,
        log_scaling,
        phantom: PhantomData,
    }
}

impl<TArtifact: GameArtifact> AnimationStage<TArtifact> {
    fn continue_with(&mut self, new_stage: Box<Self>, set_loop: bool) {
        let mut s = self;

        while let Some(ref mut next) = s.next {
            s = next.as_mut();
        }
        s.loop_next = set_loop;
        s.next = Some(new_stage);
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

    /// Adds 'next' to the end of the chain. Returns self.    
    pub fn and_then(mut self, next: impl Animation<TArtifact>) -> Self {
        self.continue_with(Box::new(next.to_stage()), false);
        self
    }
}

pub trait UpdateIfTargetDifferent<TArtifact: GameArtifact> {
    fn update_if_target_different<L: Lens<Object = TArtifact>>(
        &mut self,
        lens: L,
        target: L::Value,
        f: impl FnOnce() -> AnimationStage<TArtifact>,
    );
}

impl<TArtifact: GameArtifact> UpdateIfTargetDifferent<TArtifact> for Option<AnimationStage<TArtifact>> {
    fn update_if_target_different<L: Lens<Object = TArtifact>>(
        &mut self,
        lens: L,
        target: L::Value,
        f: impl FnOnce() -> AnimationStage<TArtifact>,
    ) {
        {
            match self {
                Some(current_animation) => {
                    if current_animation
                        .end_value_for_lens(lens)
                        .is_some_and(|x| x.eq(&target))
                    {
                        *self = Some(f())
                    }
                    // if current_animation.end_value_for_lens(lens)
                }
                None => *self = Some(f()),
            }
        }
    }
}

pub trait Animation<TArtifact: GameArtifact>: std::any::Any + Send + Sync + 'static {
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

    /// Returns the target value if one exists and the lens of this animation matches the given type id
    fn target_value_if_lens(&self, lens_type_id: TypeId) -> Option<&dyn Any>;
}

// pub trait AnimationTargetValue<TArtifact: GameArtifact, V> : Animation<TArtifact>{
//     fn target_value(&self)-> V;
// }

pub struct AnimateEmpty;

impl<TArtifact: GameArtifact> Animation<TArtifact> for AnimateEmpty {
    fn step(&mut self, _object: &mut TArtifact, _delta_ms: f64) -> AnimateResult {
        AnimateResult::FinishStep
    }

    fn is_finished(&self, _target: &TArtifact) -> bool {
        true
    }

    fn box_clone(&self) -> Box<dyn Animation<TArtifact>> {
        Box::new(Self)
    }

    fn target_value_if_lens(&self, _lens_type_id: TypeId) -> Option<&dyn Any> {
        None
    }
}

pub trait EaseFunction<V>: Clone + Copy + Send + Sync + 'static {
    fn ease(self, from: V, to: V, ratio: f64) -> V;
}

#[derive(Debug)]
pub struct AnimateSetValue<
    Lens: SetValueLens<Object: GameArtifact, Value: Send + Sync + 'static + Clone + PartialEq + Sized>,
> {
    target_value: Lens::Value,
    phantom: PhantomData<Lens>,
}

impl<
    Lens: SetValueLens<
            Object: GameArtifact,
            Value: std::any::Any + Send + Sync + 'static + Clone + PartialEq + Sized,
        >,
> AnimateSetValue<Lens>
{
    pub const fn new(target_value: Lens::Value) -> Self {
        Self {
            target_value,
            phantom: PhantomData,
        }
    }

    // pub fn get_end_value_for_lens<Lens2 : SetValueLens<Object = Lens::Object, Value : Sized>>(&self, option: &mut Option< Lens2::Value>){
    //     if TypeId::of::<Lens2>()  == TypeId::of::<Lens>(){
    //         let tv: Lens::Value = self.target_value.clone();

    //         let v2 = std::any::Any::type_id(&self) tv.downcast_ref();
    //         let v2: Lens2::Value = unsafe{ std::mem::transmute(tv)};
    //         *option = Some(v2);
    //     }
    // }
}

impl<Lens: SetValueLens<Object: GameArtifact, Value: ApproachValue>> Clone
    for AnimateSetValue<Lens>
{
    fn clone(&self) -> Self {
        Self {
            target_value: self.target_value.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<
    Lens: GetValueLens<Object: GameArtifact, Value: ApproachValue + Clone + PartialEq> + SetValueLens,
> Animation<Lens::Object> for AnimateSetValue<Lens>
{
    fn step(&mut self, object: &mut Lens::Object, _delta_ms: f64) -> AnimateResult {
        Lens::set(object, self.target_value.clone());
        AnimateResult::FinishStep
    }

    fn is_finished(&self, target: &Lens::Object) -> bool {
        self.target_value == Lens::get_value(target)
    }

    fn box_clone(&self) -> Box<dyn Animation<Lens::Object>> {
        let clone: AnimateSetValue<Lens> = self.clone();
        Box::new(clone)
    }

    fn target_value_if_lens(&self, lens_type_id: TypeId) -> Option<&dyn Any> {
        if lens_type_id == Lens::default().type_id() {
            Some(&self.target_value)
        } else {
            None
        }
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
    Lens: GetValueLens + UpdateLens<Object: GameArtifact, Value: ApproachValue + DifferenceLogScaling>,
> {
    target_value: Lens::Value,
    base_velocity_units_per_ms: f64,
    /// if the current distance is x, the velocity is:
    /// base_velocity * 2^max(1.0, x/log_scaling)
    log_scaling: f64,
    phantom: PhantomData<Lens>,
}

impl<
    Lens: GetValueLens + UpdateLens<Object: GameArtifact, Value: ApproachValue + DifferenceLogScaling>,
> Clone for AnimateSpringTowards<Lens>
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
    Lens: GetValueLens + UpdateLens<Object: GameArtifact, Value: ApproachValue + DifferenceLogScaling>,
> Animation<Lens::Object> for AnimateSpringTowards<Lens>
{
    fn step(&mut self, object: &mut Lens::Object, delta_ms: f64) -> AnimateResult {
        let mut result: AnimateResult = AnimateResult::FinishStep;
        Lens::update(object, |current_value| {
            let velocity = Lens::Value::calculate_scaled_velocity(
                current_value,
                &self.target_value,
                self.log_scaling,
                self.base_velocity_units_per_ms,
            );

            if Lens::Value::approach(current_value, &self.target_value, velocity, delta_ms) {
                result = AnimateResult::FinishStep;
            } else {
                result = AnimateResult::Continue;
            }
        });

        result
    }

    fn is_finished(&self, target: &Lens::Object) -> bool {
        self.target_value == Lens::get_value(target)
    }

    fn box_clone(&self) -> Box<dyn Animation<Lens::Object>> {
        let clone: AnimateSpringTowards<Lens> = self.clone();
        Box::new(clone)
    }

    fn target_value_if_lens(&self, lens_type_id: TypeId) -> Option<&dyn Any> {
        if lens_type_id == Lens::default().type_id() {
            Some(&self.target_value)
        } else {
            None
        }
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

    fn target_value_if_lens(&self, lens_type_id: TypeId) -> Option<&dyn Any> {
        if lens_type_id == Lens::default().type_id() {
            Some(&self.target_value)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct AnimateEase<
    Lens: UpdateLens<Object: GameArtifact, Value: EaseValue>,
    F: Fn(f64) -> f64 + Clone + Send + Sync + 'static,
> {
    pub from: Lens::Value,
    pub to: Lens::Value,
    /// Between 0.0 and 1.0
    pub progress: f64,
    pub progress_per_ms: f64,
    pub func: F,
}

impl<
    Lens: UpdateLens<Object: GameArtifact, Value: EaseValue>,
    F: Fn(f64) -> f64 + Clone + Send + Sync + 'static,
> Animation<Lens::Object> for AnimateEase<Lens, F>
{
    fn step(&mut self, object: &mut Lens::Object, delta_ms: f64) -> AnimateResult {
        self.progress = (self.progress + (self.progress_per_ms * delta_ms)).min(1.0);

        let t = (self.func)(self.progress);
        let new_value = self.from.mix(&self.to, t);

        Lens::update(object, |v| *v = new_value);

        if self.progress >= 1.0 {
            AnimateResult::FinishStep
        } else {
            AnimateResult::Continue
        }
    }

    fn is_finished(&self, _target: &Lens::Object) -> bool {
        self.progress >= 1.0
    }

    fn box_clone(&self) -> Box<dyn Animation<Lens::Object>> {
        let Self {
            from,
            to,
            progress,
            progress_per_ms,
            func,
        } = self;
        Box::new(Self {
            from: from.clone(),
            to: to.clone(),
            progress: *progress,
            progress_per_ms: *progress_per_ms,
            func: func.clone(),
        })
    }

    fn target_value_if_lens(&self, lens_type_id: TypeId) -> Option<&dyn Any> {
        if lens_type_id == Lens::default().type_id() {
            Some(&self.to)
        } else {
            None
        }
    }
}

pub trait EaseValue: Send + Sync + 'static + Clone + PartialEq {
    fn mix(&self, other: &Self, factor: f64) -> Self;
}

impl EaseValue for Srgba {
    fn mix(&self, other: &Self, factor: f64) -> Self {
        let factor = factor as f32;
        let n_factor = 1.0 - factor;
        Self {
            red: self.red * n_factor + other.red * factor,
            green: self.green * n_factor + other.green * factor,
            blue: self.blue * n_factor + other.blue * factor,
            alpha: self.alpha * n_factor + other.alpha * factor,
        }
    }
}

impl EaseValue for f32 {
    fn mix(&self, other: &Self, factor: f64) -> Self {
        let factor = factor as f32;
        let n_factor = 1.0 - factor;
        self * n_factor + other * factor
    }
}

impl EaseValue for f64 {
    fn mix(&self, other: &Self, factor: f64) -> Self {
        let n_factor = 1.0 - factor;
        self * n_factor + other * factor
    }
}

impl EaseValue for Vec2 {
    fn mix(&self, other: &Self, factor: f64) -> Self {
        let factor = factor as f32;
        let n_factor = 1.0 - factor;
        Self {
            x: self.x * n_factor + other.x * factor,
            y: self.y * n_factor + other.y * factor,
        }
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

impl ApproachValue for simple_color::Srgba {
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: f64,
        delta_ms: f64,
    ) -> bool {
        let v4 = value.to_vec4();
        let v4 = v4.move_towards(
            target_value.to_vec4(),
            (velocity_units_per_ms * delta_ms) as f32,
        );
        *value = simple_color::Srgba::from_vec4(v4);
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

impl<T: ApproachValue + Default> ApproachValue for Option<T> {
    fn approach(
        value: &mut Self,
        target_value: &Self,
        velocity_units_per_ms: f64,
        delta_ms: f64,
    ) -> bool {
        match (value.as_mut(), target_value) {
            (None, None) => {
                return true;
            }
            (None, Some(tv)) => {
                let mut v = T::default();
                let r = <T as ApproachValue>::approach(&mut v, tv, velocity_units_per_ms, delta_ms);

                *value = Some(v);
                r
            }
            (Some(_), None) => {
                *value = None;
                return true;
            }
            (Some(v), Some(tv)) => {
                <T as ApproachValue>::approach(v, tv, velocity_units_per_ms, delta_ms)
            }
        }
    }
}
