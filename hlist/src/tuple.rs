use super::{hlist, HList, HNil};

pub trait IntoTuple {
    type IntoTuple: IntoHList<IntoHList = Self>;

    fn into_tuple(self) -> Self::IntoTuple;
}

pub trait IntoHList {
    type IntoHList: IntoTuple<IntoTuple = Self>;

    fn into_hlist(self) -> Self::IntoHList;
}

impl IntoTuple for HNil {
    type IntoTuple = ();

    #[inline]
    fn into_tuple(self) -> Self::IntoTuple {
        ()
    }
}

impl IntoHList for () {
    type IntoHList = HNil;

    #[inline]
    fn into_hlist(self) -> Self::IntoHList {
        HNil
    }
}

macro_rules! define_tuple_impls {
    ([$($T:ident),*] $head:ident) => {
        define_tuple_impl!($($T,)* $head);
    };
    ([$($T:ident),*] $head:ident, $($tail:tt),*) => {
        define_tuple_impl!($($T,)* $head);
        define_tuple_impls!([$($T,)* $head] $($tail),*);
    };
    ($($T:tt),*) => {
        define_tuple_impls!([] $($T),*);
    };
}

macro_rules! define_tuple_impl {
    ($($T:ident),*) => {
        impl<$($T),*> IntoTuple for HList!($($T),*) {
            type IntoTuple = ($($T,)*);

            #[inline]
            fn into_tuple(self) -> Self::IntoTuple {
                #[allow(non_snake_case)]
                let hlist![$($T),*] = self;
                ($($T,)*)
            }
        }

        impl<$($T),*> IntoHList for ($($T,)*) {
            type IntoHList = HList!($($T),*);

            #[inline]
            fn into_hlist(self) -> Self::IntoHList {
                #[allow(non_snake_case)]
                let ($($T,)*) = self;
                hlist![$($T),*]
            }
        }
    };
}

define_tuple_impls! {
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
    T11,
    T12,
    T13,
    T14,
    T15
}
