use std::marker::PhantomData;

use glam::Vec2;

pub trait Lens: std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static {
    type Object;
    type Value: 'static;
}
// todo have both types of lens `GetRefLens` and `GetRefMaybeLens` etc.
// The second type should be automatically inferred from the first
// There need to be two types of prisms to support this
// Transitions should use the maybe type

// TODO add a marker type parameter to get around the orphan rule

pub trait GetRefLens: Lens {
    fn get_ref(object: &Self::Object) -> &Self::Value;
}

pub trait GetMutLens: Lens {
    fn get_mut(object: &mut Self::Object) -> &mut Self::Value;
}

pub trait GetValueLens: Lens {
    fn get_value(object: &<Self as Lens>::Object) -> <Self as Lens>::Value;
}

pub trait SetValueLens: Lens {
    fn set(object: &mut <Self as Lens>::Object, value: <Self as Lens>::Value);
}

impl<L: GetMutLens> SetValueLens for L {
    fn set(object: &mut <Self as Lens>::Object, value: <Self as Lens>::Value) {
        let o = L::get_mut(object);
        *o = value;
    }
}

pub trait UpdateLens: Lens {
    fn update(object: &mut Self::Object, fun: impl FnOnce(&mut Self::Value));
}

// IdentityLens
#[derive(Debug, Copy, Eq)]
pub struct IdentityLens<T: std::fmt::Debug + Send + Sync + 'static> {
    phantom: PhantomData<T>,
}

impl<T: std::fmt::Debug + Send + Sync + 'static> GetRefLens for IdentityLens<T> {
    fn get_ref(object: &Self::Object) -> &Self::Value {
        object
    }
}

impl<T: std::fmt::Debug + Clone + Send + Sync + 'static> GetValueLens for IdentityLens<T> {
    fn get_value(object: &<Self as Lens>::Object) -> <Self as Lens>::Value {
        object.clone()
    }
}

impl<T: std::fmt::Debug + Send + Sync + 'static> GetMutLens for IdentityLens<T> {
    fn get_mut(object: &mut Self::Object) -> &mut Self::Value {
        object
    }
}

impl<T: std::fmt::Debug + Send + Sync + 'static> Clone for IdentityLens<T> {
    fn clone(&self) -> Self {
        Self {
            phantom: self.phantom,
        }
    }
}

impl<T: std::fmt::Debug + Send + Sync + 'static> PartialEq for IdentityLens<T> {
    fn eq(&self, other: &Self) -> bool {
        self.phantom == other.phantom
    }
}

impl<T: std::fmt::Debug + Send + Sync + 'static> Lens for IdentityLens<T> {
    type Object = T;
    type Value = T;
}

// PRISMS

// Prism2

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct Prism2<L1: Lens, L2: Lens<Object = L1::Value>> {
    phantom: PhantomData<(L1, L2)>,
}

impl<L1: Lens, L2: Lens<Object = L1::Value>> Lens for Prism2<L1, L2> {
    type Object = L1::Object;
    type Value = L2::Value;
}

impl<L1: GetRefLens, L2: GetRefLens + Lens<Object = L1::Value>> GetRefLens for Prism2<L1, L2> {
    fn get_ref(object: &Self::Object) -> &Self::Value {
        L2::get_ref(L1::get_ref(object))
    }
}

impl<L1: GetRefLens, L2: GetValueLens + Lens<Object = L1::Value>> GetValueLens for Prism2<L1, L2> {
    fn get_value(object: &<Self as Lens>::Object) -> <Self as Lens>::Value {
        L2::get_value(L1::get_ref(object))
    }
}

impl<L1: GetMutLens, L2: Lens<Object = L1::Value> + GetMutLens> GetMutLens for Prism2<L1, L2> {
    fn get_mut(object: &mut Self::Object) -> &mut Self::Value {
        L2::get_mut(L1::get_mut(object))
    }
}

// Prism3

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct Prism3<L1: Lens, L2: Lens<Object = L1::Value>, L3: Lens<Object = L2::Value>> {
    phantom: PhantomData<(L1, L2, L3)>,
}

impl<L1: Lens, L2: Lens<Object = L1::Value>, L3: Lens<Object = L2::Value>> Lens
    for Prism3<L1, L2, L3>
{
    type Object = L1::Object;
    type Value = L3::Value;
}

impl<
    L1: GetRefLens,
    L2: GetRefLens + Lens<Object = L1::Value>,
    L3: GetRefLens + Lens<Object = L2::Value>,
> GetRefLens for Prism3<L1, L2, L3>
{
    fn get_ref(object: &Self::Object) -> &Self::Value {
        L3::get_ref(L2::get_ref(L1::get_ref(object)))
    }
}

impl<
    L1: GetRefLens,
    L2: GetRefLens + Lens<Object = L1::Value>,
    L3: GetValueLens + Lens<Object = L2::Value>,
> GetValueLens for Prism3<L1, L2, L3>
{
    fn get_value(object: &<Self as Lens>::Object) -> <Self as Lens>::Value {
        L3::get_value(L2::get_ref(L1::get_ref(object)))
    }
}

impl<
    L1: GetMutLens,
    L2: Lens<Object = L1::Value> + GetMutLens,
    L3: Lens<Object = L2::Value> + GetMutLens,
> GetMutLens for Prism3<L1, L2, L3>
{
    fn get_mut(object: &mut Self::Object) -> &mut Self::Value {
        L3::get_mut(L2::get_mut(L1::get_mut(object)))
    }
}

// TUPLES

macro_rules! impl_lens {
    ($L0:ident, $($L:ident),*) => {
        impl<$L0 : Lens, $($L : Lens<Object = $L0::Object>),*> Lens for ($L0, $($L,)*) {
            type Object = $L0::Object;
            type Value = ($L0::Value, $($L::Value,)*);
        }
    };
}

macro_rules! impl_get_value_lens {
    ($L0:ident, $($L:ident),*) => {
        impl<$L0 : GetValueLens, $($L : GetValueLens + Lens<Object = $L0::Object>),*> GetValueLens for ($L0, $($L,)*) {
            fn get_value(object: &<Self as Lens>::Object) -> <Self as Lens>::Value {
                 ($L0::get_value(object), $($L::get_value(object),)*)
            }
        }
    };
}

macro_rules! impl_set_lens {
    (($L0:ident, $l0:ident), $(($L:ident, $l:ident)),*) => {
        impl<$L0 : SetValueLens, $($L : SetValueLens + Lens<Object = $L0::Object>),*> SetValueLens for ($L0, $($L,)*) {
            fn set(object: &mut <Self as Lens>::Object, value: <Self as Lens>::Value) {
                let ($l0, $($l,)*) = value;


                $L0::set(object, $l0);
                $($L::set(object, $l);)*
            }
        }
    };
}

impl_lens!(L0, L1);
impl_lens!(L0, L1, L2);
impl_lens!(L0, L1, L2, L3);

impl_get_value_lens!(L0, L1);
impl_get_value_lens!(L0, L1, L2);
impl_get_value_lens!(L0, L1, L2, L3);

impl_set_lens!((L0, l0), (L1, l1));
impl_set_lens!((L0, l0), (L1, l1), (L2, l2));
impl_set_lens!((L0, l0), (L1, l1), (L2, l2), (L3, l3));

#[macro_export]
macro_rules! define_lens {
    ($L:ident, $O:ident, $V:ident, $p:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $L;

        impl $crate::lens::Lens for $L {
            type Object = $O;
            type Value = $V;
        }

        impl $crate::lens::GetRefLens for $L {
            fn get_ref(object: &Self::Object) -> &Self::Value {
                &object.$p
            }
        }

        impl $crate::lens::GetValueLens for $L {
            fn get_value(object: &Self::Object) -> Self::Value {
                object.$p
            }
        }

        impl $crate::lens::GetMutLens for $L {
            fn get_mut(object: &mut Self::Object) -> &mut Self::Value {
                &mut object.$p
            }
        }
    };
}

#[macro_export]
macro_rules! define_signal_lens {
    ($L:ident, $O:ident, $V:ident, $p:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $L;

        impl $crate::lens::Lens for $L {
            type Object = $O;
            type Value = $V;
        }

        impl $crate::lens::UpdateLens for $L {
            fn update(object: &mut Self::Object, f: impl FnOnce(&mut Self::Value)) {
                object.$p.update(f);
            }
        }

        impl $crate::lens::GetValueLens for $L {
            fn get_value(object: &Self::Object)-> Self::Value{
                object.$p.get_untracked()
            }
        }

        impl SetValueLens for $L {
            fn set(object: &mut <Self as Lens>::Object, value: <Self as Lens>::Value) {
                object.$p.set(value)
            }
        }
    };
}

define_lens!(Vec2XLens, Vec2, f32, x);
define_lens!(Vec2YLens, Vec2, f32, y);
