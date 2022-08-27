mod component_element;
mod view_element;

pub use component_element::ComponentElement;
pub use view_element::ViewElement;

use std::fmt;

use crate::adapt::Adapt;
use crate::component_node::ComponentStack;
use crate::render::RenderContext;
use crate::state::State;
use crate::view::View;
use crate::widget::Widget;
use crate::widget_node::{WidgetNode, WidgetNodeScope, WidgetNodeSeq};

pub trait Element<S: State, E> {
    type View: View<S, E>;

    type Components: ComponentStack<S, E, View = Self::View>;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E>;

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;

    fn adapt<F, OriginState>(self, f: F) -> Adapt<Self, F, S>
    where
        Self: Sized,
        F: Fn(&OriginState) -> &S + Sync + Send + 'static,
    {
        Adapt::new(self, f.into())
    }
}

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

pub trait DebuggableElement<S: State, E>:
    Element<
        S,
        E,
        View = <Self as DebuggableElement<S, E>>::View,
        Components = <Self as DebuggableElement<S, E>>::Components,
    > + fmt::Debug
{
    type View: View<S, E, Widget = Self::Widget> + fmt::Debug;

    type Widget: Widget<S, E, Children = Self::Children> + fmt::Debug;

    type Children: WidgetNodeSeq<S, E> + fmt::Debug;

    type Components: ComponentStack<S, E, View = <Self as DebuggableElement<S, E>>::View>
        + fmt::Debug;
}

impl<El, S, E> DebuggableElement<S, E> for El
where
    El: Element<S, E> + fmt::Debug,
    El::View: fmt::Debug,
    <El::View as View<S, E>>::Widget: fmt::Debug,
    <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children: fmt::Debug,
    El::Components: fmt::Debug,
    S: State,
{
    type View = El::View;

    type Widget = <El::View as View<S, E>>::Widget;

    type Children = <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children;

    type Components = El::Components;
}
