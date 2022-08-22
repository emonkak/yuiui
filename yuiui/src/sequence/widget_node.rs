use std::any::TypeId;
use std::fmt;
use std::ops::ControlFlow;

use crate::component::{Component, ComponentStack};
use crate::context::{EffectContext, RenderContext};
use crate::element::{ComponentElement, Element, ViewElement};
use crate::event::{CaptureState, EventMask, InternalEvent};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

use super::{CallbackMut, CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};

pub struct WidgetNodeStore<V: View<S, E>, CS, S: State, E> {
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

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, env, context))
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
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

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, env, context))
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let has_changed = Element::update(self, store.node.scope(), state, env, context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<V, CS, S, E> WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E>,
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
        event_mask.add(TypeId::of::<<V::Widget as Widget<S, E>>::Event>());
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
    ) -> CaptureState {
        self.node.event(event, state, env, context);
        CaptureState::Captured
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState {
        if event.id_path.top_id() == self.node.id {
            self.node.internal_event(event, state, env, context)
        } else {
            CaptureState::Ignored
        }
    }
}

impl<V, CS, S, E> fmt::Debug for WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S, E>>::Children: fmt::Debug,
    CS: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WidgetNodeStore")
            .field("node", &self.node)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<'a, V, CS, S, E, C> TraversableSeq<C> for &'a WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
    C: CallbackMut<&'a WidgetNode<V, CS, S, E>>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        callback.call(&self.node)
    }
}

impl<'a, V, CS, S, E, C> TraversableSeq<C> for &'a mut WidgetNodeStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
    C: CallbackMut<&'a mut WidgetNode<V, CS, S, E>>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        callback.call(&mut self.node)
    }
}
