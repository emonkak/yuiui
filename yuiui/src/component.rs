use std::fmt;
use std::marker::PhantomData;

use crate::element::{ComponentElement, Element};
use crate::event::Lifecycle;
use crate::id::IdContext;
use crate::view_node::ViewNodeMut;

pub trait Component<S, M, E>: Sized {
    type Element: Element<S, M, E>;

    #[inline]
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_node: ViewNodeMut<
            '_,
            <Self::Element as Element<S, M, E>>::View,
            <Self::Element as Element<S, M, E>>::Components,
            S,
            M,
            E,
        >,
        _state: &S,
        _messages: &mut Vec<M>,
        _entry_point: &E,
        _id_context: &mut IdContext,
    ) {
    }

    fn render(&self, state: &S, id_context: &mut IdContext) -> Self::Element;

    #[inline]
    fn el(self) -> ComponentElement<Self> {
        ComponentElement::new(self)
    }
}

pub trait HigherOrderComponent<Props, S, M, E> {
    type Component: Component<S, M, E>;

    fn build(self, props: Props) -> Self::Component;

    #[inline]
    fn el(self, props: Props) -> ComponentElement<Self::Component>
    where
        Self: Sized,
    {
        self.build(props).el()
    }
}

impl<Props, Element, S, M, E, RenderFn> HigherOrderComponent<Props, S, M, E> for RenderFn
where
    Element: self::Element<S, M, E>,
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
{
    type Component = FunctionComponentInstance<Props, Element, S, M, E, RenderFn>;

    #[inline]
    fn build(self, props: Props) -> Self::Component {
        FunctionComponentInstance::new(
            props,
            self,
            |_props, _lifecycle, _store, _messages, _entry_point, _id_context| {},
        )
    }
}

pub struct FunctionComponent<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    render_fn: RenderFn,
    lifecycle_fn: LifeCycleFn,
    _phantom: PhantomData<(Props, E, S, M, E)>,
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn>
    FunctionComponent<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    #[inline]
    pub const fn new(render_fn: RenderFn, lifecycle_fn: LifeCycleFn) -> Self {
        Self {
            render_fn,
            lifecycle_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn> HigherOrderComponent<Props, S, M, E>
    for FunctionComponent<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    Element: self::Element<S, M, E>,
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    type Component = FunctionComponentInstance<Props, Element, S, M, E, RenderFn, LifeCycleFn>;

    #[inline]
    fn build(self, props: Props) -> Self::Component {
        FunctionComponentInstance::new(props, self.render_fn, self.lifecycle_fn)
    }
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn> fmt::Debug
    for FunctionComponent<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    Props: fmt::Debug,
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionComponent").finish_non_exhaustive()
    }
}

pub struct FunctionComponentInstance<
    Props,
    Element,
    S,
    M,
    E,
    RenderFn = fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn = fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
> where
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    props: Props,
    render_fn: RenderFn,
    lifecycle_fn: LifeCycleFn,
    _phantom: PhantomData<(Element, S, M, E)>,
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn>
    FunctionComponentInstance<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    #[inline]
    pub const fn new(props: Props, render_fn: RenderFn, lifecycle_fn: LifeCycleFn) -> Self {
        Self {
            props,
            render_fn,
            lifecycle_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn> Component<S, M, E>
    for FunctionComponentInstance<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    Element: self::Element<S, M, E>,
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    type Element = Element;

    #[inline]
    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        _view_node: ViewNodeMut<'_, Element::View, Element::Components, S, M, E>,
        state: &S,
        messages: &mut Vec<M>,
        entry_point: &E,
        id_context: &mut IdContext,
    ) {
        let lifecycle = lifecycle.map(|component| component.props);
        (self.lifecycle_fn)(
            &self.props,
            lifecycle,
            state,
            messages,
            entry_point,
            id_context,
        )
    }

    #[inline]
    fn render(&self, state: &S, id_context: &mut IdContext) -> Self::Element {
        (self.render_fn)(&self.props, state, id_context)
    }
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn> Clone
    for FunctionComponentInstance<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    Props: Clone,
    RenderFn: Clone + Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Clone + Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            props: self.props.clone(),
            render_fn: self.render_fn.clone(),
            lifecycle_fn: self.lifecycle_fn.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn> AsRef<Props>
    for FunctionComponentInstance<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    #[inline]
    fn as_ref(&self) -> &Props {
        &self.props
    }
}

impl<Props, Element, S, M, E, RenderFn, LifeCycleFn> fmt::Debug
    for FunctionComponentInstance<Props, Element, S, M, E, RenderFn, LifeCycleFn>
where
    Props: fmt::Debug,
    RenderFn: Fn(&Props, &S, &mut IdContext) -> Element,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &S, &mut Vec<M>, &E, &mut IdContext),
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionComponentInstance")
            .field("props", &self.props)
            .finish_non_exhaustive()
    }
}
