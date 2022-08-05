use crate::component::Component;
use crate::context::Context;
use crate::element_seq::ElementSeq;
use crate::view::{View, ViewPod};
use crate::widget::{Widget, WidgetPod};

pub trait Element: 'static {
    type View: View;

    type Components;

    fn depth() -> usize;

    fn compile(
        view_pod: &ViewPod<Self::View, Self::Components>,
    ) -> WidgetPod<<Self::View as View>::Widget> {
        WidgetPod {
            id: view_pod.id,
            widget: Self::View::build(&view_pod.view, &view_pod.children),
            children: <Self::View as View>::Children::compile(&view_pod.children),
        }
    }

    fn recompile(
        view_pod: &ViewPod<Self::View, Self::Components>,
        widget: &mut <Self::View as View>::Widget,
        children: &mut <<Self::View as View>::Widget as Widget>::Children,
    ) -> bool {
        let mut has_changed = view_pod.view.rebuild(&view_pod.children, widget);
        has_changed |= <Self::View as View>::Children::recompile(&view_pod.children, children);
        has_changed
    }

    fn invalidate(view_pod: &ViewPod<Self::View, Self::Components>, context: &mut Context) {
        context.invalidate(view_pod.id);
        <Self::View as View>::Children::invalidate(&view_pod.children, context);
    }

    fn build(self, context: &mut Context) -> ViewPod<Self::View, Self::Components>;

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::Views,
        components: &mut Self::Components,
        context: &mut Context,
    ) -> bool;
}

#[derive(Debug)]
pub struct ViewElement<V: View> {
    view: V,
    children: V::Children,
}

impl<V: View> Element for ViewElement<V> {
    type View = V;

    type Components = ();

    fn depth() -> usize {
        Self::View::depth()
    }

    fn build(self, context: &mut Context) -> ViewPod<Self::View, Self::Components> {
        let id = context.next_identity();
        context.push(id);
        let children = ElementSeq::build(self.children, context);
        context.pop();
        ViewPod {
            id,
            view: self.view,
            children,
            components: (),
        }
    }

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::Views,
        _components: &mut Self::Components,
        context: &mut Context,
    ) -> bool {
        *view = self.view;
        *children = self.children.build(context);
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component> {
    component: C,
}

impl<C: Component> Element for ComponentElement<C> {
    type View = <C::Element as Element>::View;

    type Components = (C, <C::Element as Element>::Components);

    fn depth() -> usize {
        Self::View::depth()
    }

    fn build(self, context: &mut Context) -> ViewPod<Self::View, Self::Components> {
        let view_pod = Component::render(&self.component).build(context);
        ViewPod {
            id: view_pod.id,
            view: view_pod.view,
            children: view_pod.children,
            components: (self.component, view_pod.components),
        }
    }

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::Views,
        components: &mut Self::Components,
        context: &mut Context,
    ) -> bool {
        let (ref old_component, rest_components) = components;
        if self.component.should_update(old_component) {
            Component::render(&self.component).rebuild(view, children, rest_components, context)
        } else {
            false
        }
    }
}

pub fn view<V: View>(view: V, children: V::Children) -> ViewElement<V> {
    ViewElement { view, children }
}

pub fn component<C: Component>(component: C) -> ComponentElement<C> {
    ComponentElement { component }
}