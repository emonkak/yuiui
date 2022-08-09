use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::component::{Component, ComponentStack};
use crate::context::Context;
use crate::element::Element;
use crate::sequence::{CommitMode, ElementSeq, WidgetNodeSeq};
use crate::view::View;
use crate::widget::{Widget, WidgetNode, WidgetNodeScope, WidgetStatus};

pub struct Adapt<T, F, SS> {
    target: T,
    selector_fn: Rc<F>,
    sub_state: PhantomData<SS>,
}

impl<T, F, SS> Adapt<T, F, SS> {
    pub fn new(target: T, selector_fn: impl Into<Rc<F>>) -> Self {
        Self {
            target,
            selector_fn: selector_fn.into(),
            sub_state: PhantomData,
        }
    }
}

impl<T: fmt::Debug, F, SS> fmt::Debug for Adapt<T, F, SS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Adapt").field(&self.target).finish()
    }
}

impl<E, F, S, SS> Element<S> for Adapt<E, F, SS>
where
    E: Element<SS>,
    F: Fn(&S) -> &SS,
{
    type View = Adapt<E::View, F, SS>;

    type Components = Adapt<E::Components, F, SS>;

    fn render(
        self,
        state: &S,
        context: &mut Context,
    ) -> WidgetNode<Self::View, Self::Components, S> {
        let selector_fn = self.selector_fn;
        let node = self.target.render((selector_fn)(state), context);
        WidgetNode {
            id: node.id,
            status: node.status.map(|status| match status {
                WidgetStatus::Prepared(widget) => {
                    WidgetStatus::Prepared(Adapt::new(widget, selector_fn.clone()))
                }
                WidgetStatus::Changed(widget, view) => WidgetStatus::Changed(
                    Adapt::new(widget, selector_fn.clone()),
                    Adapt::new(view, selector_fn.clone()),
                ),
                WidgetStatus::Uninitialized(view) => {
                    WidgetStatus::Uninitialized(Adapt::new(view, selector_fn.clone()))
                }
            }),
            children: Adapt::new(node.children, selector_fn.clone()),
            components: Adapt::new(node.components, selector_fn.clone()),
        }
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut Context,
    ) -> bool {
        let mut new_status = scope.status.take().map(|status| match status {
            WidgetStatus::Prepared(widget) => WidgetStatus::Prepared(widget.target),
            WidgetStatus::Changed(widget, view) => {
                WidgetStatus::Changed(widget.target, view.target)
            }
            WidgetStatus::Uninitialized(view) => WidgetStatus::Uninitialized(view.target),
        });
        let new_scope = WidgetNodeScope {
            id: scope.id,
            status: &mut new_status,
            children: &mut scope.children.target,
            components: &mut scope.components.target,
        };
        let selector_fn = self.selector_fn;
        let has_changed = self.target.update(new_scope, selector_fn(state), context);
        *scope.status = new_status.map(|status| match status {
            WidgetStatus::Prepared(widget) => {
                WidgetStatus::Prepared(Adapt::new(widget, selector_fn.clone()))
            }
            WidgetStatus::Changed(widget, view) => WidgetStatus::Changed(
                Adapt::new(widget, selector_fn.clone()),
                Adapt::new(view, selector_fn.clone()),
            ),
            WidgetStatus::Uninitialized(view) => {
                WidgetStatus::Uninitialized(Adapt::new(view, selector_fn.clone()))
            }
        });
        has_changed
    }
}

impl<ES, F, S, SS> ElementSeq<S> for Adapt<ES, F, SS>
where
    ES: ElementSeq<SS>,
    F: Fn(&S) -> &SS,
{
    type Store = Adapt<ES::Store, F, SS>;

    fn render(self, state: &S, context: &mut Context) -> Self::Store {
        Adapt::new(
            self.target.render((self.selector_fn)(state), context),
            self.selector_fn.clone(),
        )
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        self.target
            .update(&mut store.target, (self.selector_fn)(state), context)
    }
}

impl<WS, F, S, SS> WidgetNodeSeq<S> for Adapt<WS, F, SS>
where
    WS: WidgetNodeSeq<SS>,
    F: Fn(&S) -> &SS,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        self.target.commit(mode, (self.selector_fn)(state), context);
    }
}

impl<C, F, S, SS> Component<S> for Adapt<C, F, SS>
where
    C: Component<SS>,
    F: Fn(&S) -> &SS,
{
    type Element = Adapt<C::Element, F, SS>;

    fn render(&self, state: &S) -> Self::Element {
        Adapt::new(
            self.target.render((self.selector_fn)(state)),
            self.selector_fn.clone(),
        )
    }

    fn should_update(&self, other: &Self, state: &S) -> bool {
        self.target
            .should_update(&other.target, (self.selector_fn)(state))
    }
}

impl<CS, F, S, SS> ComponentStack<S> for Adapt<CS, F, SS>
where
    CS: ComponentStack<SS>,
    F: Fn(&S) -> &SS,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        self.target.commit(mode, (self.selector_fn)(state), context);
    }
}

impl<V, F, S, SS> View<S> for Adapt<V, F, SS>
where
    V: View<SS>,
    F: Fn(&S) -> &SS,
{
    type Widget = Adapt<V::Widget, F, SS>;

    type Children = Adapt<V::Children, F, SS>;

    fn build(self, children: &<Self::Widget as Widget<S>>::Children, state: &S) -> Self::Widget {
        Adapt::new(
            self.target
                .build(&children.target, (self.selector_fn)(state)),
            self.selector_fn.clone(),
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
            (self.selector_fn)(state),
        )
    }
}

impl<W, F, S, SS> Widget<S> for Adapt<W, F, SS>
where
    W: Widget<SS>,
    F: Fn(&S) -> &SS,
{
    type Children = Adapt<W::Children, F, SS>;
}