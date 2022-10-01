use std::ops::RangeInclusive;
use std::sync::Once;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeRange, ViewNodeSeq};

impl<S, M, B> ElementSeq<S, M, B> for () {
    type Storage = ();

    fn render_children(self, _context: &mut RenderContext, _store: &Store<S>) -> Self::Storage {
        ()
    }

    fn update_children(
        self,
        _nodes: &mut Self::Storage,
        _context: &mut RenderContext,
        _store: &Store<S>,
    ) -> bool {
        false
    }
}

impl<S, M, B> ViewNodeSeq<S, M, B> for () {
    const IS_DYNAMIC: bool = false;

    fn event_mask() -> &'static EventMask {
        static MASK: EventMask = EventMask::new();
        &MASK
    }

    fn len(&self) -> usize {
        0
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> bool {
        false
    }
}

impl<Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B> for ()
where
    Output: Default,
{
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Output {
        Output::default()
    }

    fn for_id(
        &mut self,
        _id: Id,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Option<Output> {
        None
    }
}

macro_rules! define_tuple_impls {
    ([$($TSS:tt),*; $($nss:tt),*; $last_n:tt], $t:tt, $($TS:tt),*; $n:tt, $($ns:tt),*) => {
        define_tuple_impl!($($TSS),*; $($nss),*; $last_n);
        define_tuple_impls!([$($TSS),*, $t; $($nss),*, $n; $n], $($TS),*; $($ns),*);
    };
    ([$($TSS:tt),*; $($nss:tt),*; $last_n:tt], $T:tt; $n:tt) => {
        define_tuple_impl!($($TSS),*; $($nss),*; $last_n);
        define_tuple_impl!($($TSS),*, $T; $($nss),*, $n; $n);
    };
    ($T:tt, $($TS:tt),*; $n:tt, $($ns:tt),*) => {
        define_tuple_impls!([$T; $n; $n], $($TS),*; $($ns),*);
    };
}

macro_rules! define_tuple_impl {
    ($($T:tt),*; $($n:tt),*; $last_n:tt) => {
        impl<$($T,)* S, M, B> ElementSeq<S, M, B> for ($($T,)*)
        where
            $($T: ElementSeq<S, M, B>,)*
        {
            type Storage = ($($T::Storage,)*);

            fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
                ($(self.$n.render_children(context, store),)*)
            }

            fn update_children(
                self,
                storage: &mut Self::Storage,
                context: &mut RenderContext,
                store: &Store<S>,
            ) -> bool {
                $(self.$n.update_children(&mut storage.$n, context, store))||*
            }
        }

        impl<$($T,)* S, M, B> ViewNodeSeq<S, M, B> for ($($T,)*)
        where
            $($T: ViewNodeSeq<S, M, B>,)*
        {
            const IS_DYNAMIC: bool = $($T::IS_DYNAMIC)||*;

            fn event_mask() -> &'static EventMask {
                static INIT: Once = Once::new();
                static mut EVENT_MASK: EventMask = EventMask::new();

                if !INIT.is_completed() {
                    $(
                        #[allow(non_snake_case)]
                        let $T = $T::event_mask();
                    )*

                    INIT.call_once(|| unsafe {
                        $(
                            if !$T.is_empty() {
                                EVENT_MASK.extend($T);
                            }
                        )*
                    });
                }

                unsafe { &EVENT_MASK }
            }

            fn len(&self) -> usize {
                0 $(+ self.$n.len())*
            }

            fn commit(
                &mut self,
                mode: CommitMode,
                context: &mut MessageContext<M>,
                store: &Store<S>,
                backend: &mut B,
            ) -> bool {
                $(self.$n.commit(mode, context, store, backend))||*
            }
        }

        impl<$($T),*> ViewNodeRange for ($($T,)*)
        where
            $($T: ViewNodeRange,)*
        {
            fn id_range(&self) -> RangeInclusive<Id> {
                let start = self.0.id_range();
                let end = self.$last_n.id_range();
                *start.start()..=*end.end()
            }
        }

        impl<$($T,)* Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B>
            for ($($T,)*)
        where
            $($T: Traversable<Visitor, Context, Output, S, B>,)*
            Output: Monoid,
        {
            fn for_each(
                &mut self,
                visitor: &mut Visitor,
                context: &mut Context,
                store: &Store<S>,
                backend: &mut B,
            ) -> Output {
                let result = Output::default();
                $(
                    let result = result.combine(self.$n.for_each(visitor, context, store, backend));
                )*
                result
            }

            fn for_id(
                &mut self,
                id: Id,
                visitor: &mut Visitor,
                context: &mut Context,
                store: &Store<S>,
                backend: &mut B,
            ) -> Option<Output> {
                $(
                    if let Some(result) = self.$n.for_id(id, visitor, context, store, backend) {
                        return Some(result);
                    }
                )*
                None
            }
        }
    };
}

define_tuple_impls! {
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15;
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
}
