use crate::context::{CommitContext, RenderContext};
use crate::element::ElementSeq;
use crate::id::Id;
use crate::view_node::{CommitMode, Traversable, ViewNodeSeq};

impl<S, M, E> ElementSeq<S, M, E> for () {
    type Storage = ();

    fn render_children(self, _context: &mut RenderContext<S>) -> Self::Storage {
        ()
    }

    fn update_children(self, _nodes: &mut Self::Storage, _context: &mut RenderContext<S>) -> bool {
        false
    }
}

impl<S, M, E> ViewNodeSeq<S, M, E> for () {
    const SIZE_HINT: (usize, Option<usize>) = (0, Some(0));

    fn len(&self) -> usize {
        0
    }

    fn commit(&mut self, _mode: CommitMode, _context: &mut CommitContext<S, M, E>) -> bool {
        false
    }

    fn gc(&mut self) {}
}

impl<Visitor, Context> Traversable<Visitor, Context> for () {
    fn for_each(&mut self, _visitor: &mut Visitor, _context: &mut Context) {}

    fn for_id(&mut self, _id: Id, _visitor: &mut Visitor, _context: &mut Context) -> bool {
        false
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
        impl<$($T,)* S, M, E> ElementSeq<S, M, E> for ($($T,)*)
        where
            $($T: ElementSeq<S, M, E>,)*
        {
            type Storage = ($($T::Storage,)*);

            fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
                ($(self.$n.render_children(context),)*)
            }

            fn update_children(
                self,
                storage: &mut Self::Storage,
                context: &mut RenderContext<S>,
            ) -> bool {
                $(self.$n.update_children(&mut storage.$n, context))||*
            }
        }

        impl<$($T,)* S, M, E> ViewNodeSeq<S, M, E> for ($($T,)*)
        where
            $($T: ViewNodeSeq<S, M, E>,)*
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

            fn commit(
                &mut self,
                mode: CommitMode,
                context: &mut CommitContext<S, M, E>,
            ) -> bool {
                $(self.$n.commit(mode, context))||*
            }

            fn gc(&mut self) {
                $(self.$n.gc();)*
            }
        }

        impl<$($T,)* Visitor, Context> Traversable<Visitor, Context>
            for ($($T,)*)
        where
            $($T: Traversable<Visitor, Context>,)*
        {
            fn for_each(
                &mut self,
                visitor: &mut Visitor,
                context: &mut Context,
            ) {
                $(
                    self.$n.for_each(visitor, context);
                )*
            }

            fn for_id(
                &mut self,
                id: Id,
                visitor: &mut Visitor,
                context: &mut Context,
            ) -> bool {
                $(
                    if self.$n.for_id(id, visitor, context) {
                        return true;
                    }
                )*
                false
            }
        }
    };
}

define_tuple_impls! {
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11;
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
}
