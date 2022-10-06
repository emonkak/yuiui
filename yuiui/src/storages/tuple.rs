use std::sync::Once;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeSeq};

impl<S, M, R> ElementSeq<S, M, R> for () {
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

impl<S, M, R> ViewNodeSeq<S, M, R> for () {
    const SIZE_HINT: (usize, Option<usize>) = (0, Some(0));

    fn event_mask() -> &'static EventMask {
        static MASK: EventMask = EventMask::new();
        &MASK
    }

    fn len(&self) -> usize {
        0
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        None
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> bool {
        false
    }

    fn gc(&mut self) {}
}

impl<Visitor, Context, Output, S, M, R> Traversable<Visitor, Context, Output, S, M, R> for ()
where
    Output: Default,
{
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Output {
        Output::default()
    }

    fn for_id(
        &mut self,
        _id: Id,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Option<Output> {
        None
    }
}

macro_rules! define_tuple_impls {
    ([$($T:ident),*] [$($n:tt),*] $T_head:ident; $n_head:tt) => {
        define_tuple_impl!($($T,)* $T_head; $($n,)* $n_head; $n_head);
    };
    ([$($T:ident),*] [$($n:tt),*] $T_head:ident, $($T_tail:tt),*; $n_head:tt, $($n_tail:tt),*) => {
        define_tuple_impl!($($T,)* $T_head; $($n,)* $n_head; $n_head);
        define_tuple_impls!([$($T,)* $T_head] [$($n,)* $n_head] $($T_tail),*; $($n_tail),*);
    };
    ($($T:ident),*; $($n:tt),*) => {
        define_tuple_impls!([] [] $($T),*; $($n),*);
    };
}

macro_rules! define_tuple_impl {
    ($($T:ident),*; $($n:tt),*; $last_n:tt) => {
        impl<$($T,)* S, M, R> ElementSeq<S, M, R> for ($($T,)*)
        where
            $($T: ElementSeq<S, M, R>,)*
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

        impl<$($T,)* S, M, R> ViewNodeSeq<S, M, R> for ($($T,)*)
        where
            $($T: ViewNodeSeq<S, M, R>,)*
        {
            const SIZE_HINT: (usize, Option<usize>) = {
                let lower = 0usize $(.saturating_add($T::SIZE_HINT.0))*;
                let upper = Some(0usize);
                $(
                    let upper = match (upper, $T::SIZE_HINT.1) {
                        (Some(x), Some(y)) => x.checked_add(y),
                        _ => None,
                    };
                )*
                (lower, upper)
            };

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
                            EVENT_MASK.extend($T);
                        )*
                    });
                }

                unsafe { &EVENT_MASK }
            }

            fn len(&self) -> usize {
                0 $(+ self.$n.len())*
            }

            fn id_range(&self) -> Option<(Id, Id)> {
                let first = self.0.id_range();
                let last = self.$last_n.id_range();
                match (first, last) {
                    (Some((start, _)), Some((_, end))) => {
                        Some((start, end))
                    },
                    _ => None
                }
            }

            fn commit(
                &mut self,
                mode: CommitMode,
                context: &mut MessageContext<M>,
                store: &Store<S>,
                renderer: &mut R,
            ) -> bool {
                $(self.$n.commit(mode, context, store, renderer))||*
            }

            fn gc(&mut self) {
                $(self.$n.gc();)*
            }
        }

        impl<$($T,)* Visitor, Context, Output, S, M, R> Traversable<Visitor, Context, Output, S, M, R>
            for ($($T,)*)
        where
            $($T: Traversable<Visitor, Context, Output, S, M, R>,)*
            Output: Monoid,
        {
            fn for_each(
                &mut self,
                visitor: &mut Visitor,
                context: &mut Context,
                store: &Store<S>,
                renderer: &mut R,
            ) -> Output {
                let result = Output::default();
                $(
                    let result = result.combine(self.$n.for_each(visitor, context, store, renderer));
                )*
                result
            }

            fn for_id(
                &mut self,
                id: Id,
                visitor: &mut Visitor,
                context: &mut Context,
                store: &Store<S>,
                renderer: &mut R,
            ) -> Option<Output> {
                $(
                    if let Some(result) = self.$n.for_id(id, visitor, context, store, renderer) {
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
