use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::component::{Component, ComponentStack};
use crate::context::Context;
use crate::element::Element;
use crate::sequence::{CommitMode, ElementSeq, WidgetNodeSeq};
use crate::view::View;
use crate::widget::{Widget, WidgetNode, WidgetNodeScope};

pub struct Adapt<T, F, NS> {
    target: T,
    mappter_fn: Rc<F>,
    new_state: PhantomData<NS>,
}

impl<T, F, NS> Adapt<T, F, NS> {
    pub fn new(target: T, mappter_fn: impl Into<Rc<F>>) -> Self {
        Self {
            target,
            mappter_fn: mappter_fn.into(),
            new_state: PhantomData,
        }
    }
}

impl<T: fmt::Debug, F, NS> fmt::Debug for Adapt<T, F, NS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Adapt")
            .field(&self.target)
            .finish()
    }
}

impl<E, F, S, NS> Element<S> for Adapt<E, F, NS>
where
    E: Element<NS>,
    F: Fn(&S) -> &NS,
{
    type View = Adapt<E::View, F, NS>;

    type Components = Adapt<E::Components, F, NS>;

    fn build(
        self,
        state: &S,
        context: &mut Context,
    ) -> WidgetNode<Self::View, Self::Components, S> {
        let mappter_fn = self.mappter_fn;
        let node = self.target.build((mappter_fn)(state), context);
        WidgetNode {
            id: node.id,
            widget: Adapt::new(node.widget, mappter_fn.clone()),
            pending_view: node
                .pending_view
                .map(|view| Adapt::new(view, mappter_fn.clone())),
            children: Adapt::new(node.children, mappter_fn.clone()),
            components: Adapt::new(node.components, mappter_fn.clone()),
        }
    }

    fn rebuild(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut Context,
    ) -> bool {
        let mut pending_view = scope.pending_view.take().map(|view| view.target);
        let new_scope = WidgetNodeScope {
            id: scope.id,
            pending_view: &mut pending_view,
            children: &mut scope.children.target,
            components: &mut scope.components.target,
        };
        let has_changed = self
            .target
            .rebuild(new_scope, (self.mappter_fn)(state), context);
        *scope.pending_view = pending_view.map(|view| Adapt::new(view, self.mappter_fn.clone()));
        has_changed
    }
}

impl<ES, F, S, NS> ElementSeq<S> for Adapt<ES, F, NS>
where
    ES: ElementSeq<NS>,
    F: Fn(&S) -> &NS,
{
    type Store = Adapt<ES::Store, F, NS>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        Adapt::new(
            self.target.build((self.mappter_fn)(state), context),
            self.mappter_fn.clone(),
        )
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        self.target
            .rebuild(&mut store.target, (self.mappter_fn)(state), context)
    }
}

impl<WS, F, S, NS> WidgetNodeSeq<S> for Adapt<WS, F, NS>
where
    WS: WidgetNodeSeq<NS>,
    F: Fn(&S) -> &NS,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        self.target.commit(mode, (self.mappter_fn)(state), context);
    }
}

impl<C, F, S, NS> Component<S> for Adapt<C, F, NS>
where
    C: Component<NS>,
    F: Fn(&S) -> &NS,
{
    type Element = Adapt<C::Element, F, NS>;

    fn render(&self, state: &S) -> Self::Element {
        Adapt::new(
            self.target.render((self.mappter_fn)(state)),
            self.mappter_fn.clone(),
        )
    }

    fn should_update(&self, other: &Self, state: &S) -> bool {
        self.target
            .should_update(&other.target, (self.mappter_fn)(state))
    }
}

impl<CS, F, S, NS> ComponentStack<S> for Adapt<CS, F, NS>
where
    CS: ComponentStack<NS>,
    F: Fn(&S) -> &NS,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        self.target.commit(mode, (self.mappter_fn)(state), context);
    }
}

impl<V, F, S, NS> View<S> for Adapt<V, F, NS>
where
    V: View<NS>,
    F: Fn(&S) -> &NS,
{
    type Widget = Adapt<V::Widget, F, NS>;

    type Children = Adapt<V::Children, F, NS>;

    fn build(self, children: &<Self::Widget as Widget<S>>::Children, state: &S) -> Self::Widget {
        Adapt::new(
            self.target
                .build(&children.target, (self.mappter_fn)(state)),
            self.mappter_fn.clone(),
        )
    }

    fn rebuild(
        self,
        children: &<Self::Widget as Widget<S>>::Children,
        widget: &mut Self::Widget,
        state: &S,
    ) -> bool {
        self.target.rebuild(
            &children.target,
            &mut widget.target,
            (self.mappter_fn)(state),
        )
    }
}

impl<W, F, S, NS> Widget<S> for Adapt<W, F, NS>
where
    W: Widget<NS>,
    F: Fn(&S) -> &NS,
{
    type Children = Adapt<W::Children, F, NS>;
}
