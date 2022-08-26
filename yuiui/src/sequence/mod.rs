mod array;
mod either;
mod hlist;
mod option;
mod vec;
mod widget_node;

use crate::effect::EffectContext;
use crate::event::EventMask;
use crate::id::{IdContext, IdPath};
use crate::state::State;
use crate::widget_node::CommitMode;

pub trait ElementSeq<S: State, E> {
    type Store: WidgetNodeSeq<S, E>;

    fn render(self, state: &S, env: &E, context: &mut IdContext) -> Self::Store;

    fn update(self, store: &mut Self::Store, state: &S, env: &E, context: &mut IdContext) -> bool;
}

pub trait WidgetNodeSeq<S: State, E> {
    fn event_mask() -> EventMask;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);
}

pub trait TraversableSeq<V, S: State, E> {
    fn for_each(&mut self, visitor: &mut V, state: &S, env: &E, context: &mut EffectContext<S>);

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool;
}

pub trait NodeVisitor<T, S: State, E> {
    fn visit(&mut self, node: &mut T, state: &S, env: &E, context: &mut EffectContext<S>);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RenderStatus {
    Unchanged,
    Changed,
    Swapped,
}
