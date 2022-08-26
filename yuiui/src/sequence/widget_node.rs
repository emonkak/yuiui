use std::fmt;

use crate::component::Component;
use crate::component_node::ComponentStack;
use crate::effect::EffectContext;
use crate::element::{ComponentElement, Element, ViewElement};
use crate::event::{Event, EventMask, InternalEvent};
use crate::id::{IdContext, IdPath};
use crate::sequence::TraverseContext;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent};
use crate::widget_node::{CommitMode, WidgetNode};

use super::{ElementSeq, NodeVisitor, TraversableSeq, WidgetNodeSeq};

pub struct WidgetNodeStore<V: View<S, E>, CS: ComponentStack<S, E>, S: State, E> {
    node: WidgetNode<V, CS, S, E>,
    dirty: bool,
}

impl<V, S, E> ElementSeq<S, E> for ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
{
    type Store =
        WidgetNodeStore<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut IdContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, env, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, env: &E, context: &mut IdContext) -> bool {
        let has_changed = Element::update(self, store.node.scope(), state, env, context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<C, S, E> ElementSeq<S, E> for ComponentElement<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    type Store =
        WidgetNodeStore<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut IdContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, env, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, env: &E, context: &mut IdContext) -> bool {
        let has_changed = Element::update(self, store.node.scope(), state, env, context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<V, CS, S, E> WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
{
    fn new(node: WidgetNode<V, CS, S, E>) -> Self {
        Self { node, dirty: true }
    }
}

impl<V, CS, S, E> WidgetNodeSeq<S, E> for WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
{
    fn event_mask() -> EventMask {
        let mut event_mask = <V::Widget as Widget<S, E>>::Children::event_mask();
        event_mask.extend(<V::Widget as WidgetEvent>::Event::allowed_types());
        event_mask
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            self.dirty = false;
            self.node.commit(mode, state, env, context);
        }
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        self.node.event(event, state, env, context)
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        self.node.internal_event(event, state, env, context)
    }
}

impl<V, CS, S, E> fmt::Debug for WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S, E>>::Children: fmt::Debug,
    CS: ComponentStack<S, E> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WidgetNodeStore")
            .field("node", &self.node)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<V, CS, Visitor, S, E, C> TraversableSeq<Visitor, S, E, C> for WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E>,
    <V::Widget as Widget<S, E>>::Children: TraversableSeq<Visitor, S, E, C>,
    CS: ComponentStack<S, E>,
    Visitor: NodeVisitor<WidgetNode<V, CS, S, E>, S, E, C>,
    S: State,
    C: TraverseContext,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut C) {
        self.node.for_each(visitor, state, env, context);
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut C,
    ) -> bool {
        self.node.search(id_path, visitor, state, env, context)
    }
}
