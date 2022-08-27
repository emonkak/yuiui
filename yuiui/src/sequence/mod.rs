mod array;
mod either;
mod hlist;
mod option;
mod vec;
mod widget_node;

use crate::component_node::ComponentStack;
use crate::effect::EffectContext;
use crate::event::EventMask;
use crate::render::{IdPath, RenderContext};
use crate::state::State;
use crate::view::View;
use crate::widget_node::WidgetNode;

pub trait ElementSeq<S: State, E> {
    type Store: WidgetNodeSeq<S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store;

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;
}

pub trait WidgetNodeSeq<S: State, E>: RenderContextSeq<S, E> + EffectContextSeq<S, E> {
    fn event_mask() -> EventMask;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);
}

pub trait RenderContextSeq<S: State, E> {
    fn for_each<V: RenderContextVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    );

    fn search<V: RenderContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;
}

pub trait RenderContextVisitor {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E>,
        S: State;
}

pub trait EffectContextSeq<S: State, E> {
    fn for_each<V: EffectContextVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    );

    fn search<V: EffectContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool;
}

pub trait EffectContextVisitor {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E>,
        S: State;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitMode {
    Mount,
    Unmount,
    Update,
}

impl CommitMode {
    pub fn is_propagatable(&self) -> bool {
        match self {
            Self::Mount | Self::Unmount => true,
            Self::Update => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RenderStatus {
    Unchanged,
    Changed,
    Swapped,
}
