use crate::component::Component;
use crate::component_node::ComponentStack;
use crate::effect::EffectContext;
use crate::element::{ComponentElement, Element, ViewElement};
use crate::event::{Event, EventMask};
use crate::render::{IdPath, RenderContext};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent};
use crate::widget_node::WidgetNode;

use super::{
    CommitMode, EffectContextSeq, EffectContextVisitor, ElementSeq, RenderContextSeq,
    RenderContextVisitor, WidgetNodeSeq,
};

impl<V, S, E> ElementSeq<S, E> for ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
{
    type Store =
        WidgetNode<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        Element::render(self, state, env, context)
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        Element::update(self, store.scope(), state, env, context)
    }
}

impl<C, S, E> ElementSeq<S, E> for ComponentElement<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    type Store =
        WidgetNode<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        Element::render(self, state, env, context)
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        Element::update(self, store.scope(), state, env, context)
    }
}

impl<V, CS, S, E> WidgetNodeSeq<S, E> for WidgetNode<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn event_mask() -> EventMask {
        let mut event_mask = <V::Widget as Widget<S, E>>::Children::event_mask();
        event_mask.extend(<V::Widget as WidgetEvent>::Event::allowed_types());
        event_mask
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.commit(mode, state, env, context);
    }
}

impl<V, CS, S, E> RenderContextSeq<S, E> for WidgetNode<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn for_each<Visitor: RenderContextVisitor>(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) {
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
    }

    fn search<Visitor: RenderContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        context.begin_widget(self.id);
        let result = if self.id == id_path.bottom_id() {
            visitor.visit(self, state, env, context);
            true
        } else if id_path.starts_with(context.id_path()) {
            RenderContextSeq::search(&mut self.children, id_path, visitor, state, env, context)
        } else {
            false
        };
        context.end_widget();
        result
    }
}

impl<V, CS, S, E> EffectContextSeq<S, E> for WidgetNode<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn for_each<Visitor: EffectContextVisitor>(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
    }

    fn search<Visitor: EffectContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        context.begin_widget(self.id);
        let result = if self.id == id_path.bottom_id() {
            visitor.visit(self, state, env, context);
            true
        } else if id_path.starts_with(context.id_path()) {
            EffectContextSeq::search(&mut self.children, id_path, visitor, state, env, context)
        } else {
            false
        };
        context.end_widget();
        result
    }
}
