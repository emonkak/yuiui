use std::any::TypeId;
use std::fmt;

use crate::component::{Component, ComponentStack};
use crate::context::{EffectContext, RenderContext};
use crate::element::{ComponentElement, Element, ViewElement};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

use super::{CommitMode, ElementSeq, WidgetNodeSeq};

impl<V, S> ElementSeq<S> for ViewElement<V, S>
where
    V: View<S>,
    S: State,
{
    type Store = WidgetNodeStore<<Self as Element<S>>::View, <Self as Element<S>>::Components, S>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let has_changed = Element::update(self, store.node.scope(), state, context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<C, S> ElementSeq<S> for ComponentElement<C, S>
where
    C: Component<S>,
    S: State,
{
    type Store = WidgetNodeStore<<Self as Element<S>>::View, <Self as Element<S>>::Components, S>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let has_changed = Element::update(self, store.node.scope(), state, context);
        store.dirty = has_changed;
        has_changed
    }
}

pub struct WidgetNodeStore<V: View<S>, CS, S: State> {
    node: WidgetNode<V, CS, S>,
    dirty: bool,
}

impl<V, CS, S> WidgetNodeStore<V, CS, S>
where
    V: View<S>,
    S: State,
{
    fn new(node: WidgetNode<V, CS, S>) -> Self {
        Self { node, dirty: true }
    }
}

impl<V, CS, S> WidgetNodeSeq<S> for WidgetNodeStore<V, CS, S>
where
    V: View<S>,
    CS: ComponentStack<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        let mut event_mask = <V::Widget as Widget<S>>::Children::event_mask();
        event_mask.add(TypeId::of::<<V::Widget as Widget<S>>::Event>());
        event_mask
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            self.dirty = false;
            self.node.commit(mode, state, context);
        }
    }

    fn event<E: 'static>(
        &mut self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        self.node.event(event, state, context)
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        self.node.internal_event(event, state, context)
    }
}

impl<V, CS, S> fmt::Debug for WidgetNodeStore<V, CS, S>
where
    V: View<S> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S>>::Children: fmt::Debug,
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
