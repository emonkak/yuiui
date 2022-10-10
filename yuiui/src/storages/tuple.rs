use crate::element::ElementSeq;
use crate::id::{Id, IdContext};
use crate::store::Store;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeSeq};

impl<S, M, R> ElementSeq<S, M, R> for () {
    type Storage = ();

    fn render_children(self, _id_context: &mut IdContext, _state: &S) -> Self::Storage {
        ()
    }

    fn update_children(
        self,
        _nodes: &mut Self::Storage,
        _id_context: &mut IdContext,
        _state: &S,
    ) -> bool {
        false
    }
}

impl<S, M, R> ViewNodeSeq<S, M, R> for () {
    const SIZE_HINT: (usize, Option<usize>) = (0, Some(0));

    fn len(&self) -> usize {
        0
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        None
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _renderer: &mut R,
    ) -> bool {
        false
    }

    fn gc(&mut self) {}
}

impl<Visitor, Output, S, M, R> Traversable<Visitor, Output, S, M, R> for ()
where
    Output: Default,
{
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Output {
        Output::default()
    }

    fn for_id(
        &mut self,
        _id: Id,
        _visitor: &mut Visitor,
        _id_context: &mut IdContext,
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

            fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
                ($(self.$n.render_children(id_context, state),)*)
            }

            fn update_children(
                self,
                storage: &mut Self::Storage,
                id_context: &mut IdContext,
                state: &S,
            ) -> bool {
                $(self.$n.update_children(&mut storage.$n, id_context, state))||*
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
                id_context: &mut IdContext,
                store: &Store<S>,
                messages: &mut Vec<M>,
                renderer: &mut R,
            ) -> bool {
                $(self.$n.commit(mode, id_context, store, messages, renderer))||*
            }

            fn gc(&mut self) {
                $(self.$n.gc();)*
            }
        }

        impl<$($T,)* Visitor, Output, S, M, R> Traversable<Visitor, Output, S, M, R>
            for ($($T,)*)
        where
            $($T: Traversable<Visitor, Output, S, M, R>,)*
            Output: Monoid,
        {
            fn for_each(
                &mut self,
                visitor: &mut Visitor,
                id_context: &mut IdContext,
                store: &Store<S>,
                renderer: &mut R,
            ) -> Output {
                let result = Output::default();
                $(
                    let result = result.combine(self.$n.for_each(visitor, id_context, store, renderer));
                )*
                result
            }

            fn for_id(
                &mut self,
                id: Id,
                visitor: &mut Visitor,
                id_context: &mut IdContext,
                store: &Store<S>,
                renderer: &mut R,
            ) -> Option<Output> {
                $(
                    if let Some(result) = self.$n.for_id(id, visitor, id_context, store, renderer) {
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
