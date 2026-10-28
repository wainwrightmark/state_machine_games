pub use paste::paste;

#[macro_export]
macro_rules! define_animation_collection {
	(
	$artifact: ty,
	$($field:ident),+

	) => {
		paste!{
			#[derive(Default)]
			pub struct [<$artifact AnimationCollection>]{
				$(pub $field : Option<$crate::animation::AnimationStage<$artifact>>),+
			}


			impl $crate::animation::AnimationCollection<$artifact> for [<$artifact AnimationCollection>] {
				fn is_empty(&self) -> bool {
					$(self.$field.is_none() &&)+ true
				}

				fn retain_mut(&mut self, mut f: impl FnMut(&mut $crate::animation::AnimationStage<$artifact>) -> bool) {
					$($crate::tuples_animation_collection::option_retain_mut(&mut self.$field, &mut f);)+
				}

				fn clear(&mut self) {
					$(self.$field  = None;)+
				}
			}
		}


	};
}

// #[derive(Clone)]
// pub struct Foo {
//     x: f32,
//     y: f32,
// }

// impl GameArtifact for Foo {}

// define_animation_collection!(Foo, x, y);

// #[derive(Default)]
// pub struct FooAnimations {
//     pub x: Option<AnimationStage<Foo>>,
//     pub y: Option<AnimationStage<Foo>>,
// }

// impl AnimationCollection<Foo> for FooAnimations {
//     fn is_empty(&self) -> bool {
//         self.x.is_none() && self.y.is_none()
//     }

//     fn retain_mut(&mut self, mut f: impl FnMut(&mut AnimationStage<Foo>) -> bool) {
//         option_retain_mut(&mut self.x, &mut f);
//         option_retain_mut(&mut self.y, &mut f);
//     }

//     fn clear(&mut self) {
//         self.x = None;
//         self.y = None;
//     }
// }

pub fn option_retain_mut<T>(option: &mut Option<T>, f: &mut impl FnMut(&mut T) -> bool) {
    match option {
        Some(t) => {
            if !f(t) {
                *option = None;
            }
        }
        None => (),
    }
}
