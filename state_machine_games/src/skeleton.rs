use impl_trait_for_tuples::impl_for_tuples;
use strum::EnumIs;

pub trait Skeleton<T> {
    fn state(&self) -> AnimationState;
    fn progress(&mut self, target: &mut T, delta_ms: f64) -> AnimationState;
}

#[derive(Debug, PartialEq, PartialOrd, EnumIs)]
pub enum AnimationState {
    Finished,
    InProgress,
}

impl std::ops::BitOr for AnimationState{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self.is_in_progress() || rhs.is_in_progress(){
            Self::InProgress
        }else{
            Self::Finished
        }
    }
}

impl<T> Skeleton<T> for () {
    fn state(&self) -> AnimationState {
        AnimationState::Finished
    }

    fn progress(&mut self, _target: &mut T, _delta_ms: f64) -> AnimationState {
        AnimationState::Finished
    }
}


#[impl_for_tuples(1,8)]
impl<T> Skeleton<T> for Tuple {

    fn state(&self) -> AnimationState {
        for_tuples!( ( #( Tuple::state(&self.Tuple) )| * ) )
    }

    fn progress(&mut self, target: &mut T, delta_ms: f64) -> AnimationState {
        for_tuples!( ( #( self.Tuple.progress(target, delta_ms) )| * ) )
    }
}