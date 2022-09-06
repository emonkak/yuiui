use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::element::{ComponentElement, Element, ElementSeq};
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeScope};

pub struct Memoize<El, Deps> {
    render: fn(&Deps) -> El,
    deps: Deps,
}

impl<El, Deps> Memoize<El, Deps> {
    pub const fn new(render: fn(&Deps) -> El, deps: Deps) -> Self {
        Self { render, deps }
    }
}

impl<El, Deps, S, E> Element<S, E> for Memoize<El, Deps>
where
    El: Element<S, E>,
    Deps: PartialEq,
    S: State,
{
    type View = El::View;

    type Components = (ComponentNode<AsComponent<Self>, S, E>, El::Components);

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, E> {
        let element = ComponentElement::new(AsComponent { inner: self });
        Element::render(element, state, env, context)
    }

    fn update(
        self,
        scope: ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let (head_node, _) = scope.components;
        if head_node.component.inner.deps != self.deps {
            let element = ComponentElement::new(AsComponent { inner: self });
            Element::update(element, scope, state, env, context)
        } else {
            head_node.pending_component = Some(AsComponent { inner: self });
            false
        }
    }
}

impl<El, Deps, S, E> ElementSeq<S, E> for Memoize<El, Deps>
where
    El: Element<S, E>,
    Deps: PartialEq,
    S: State,
{
    type Storage =
        ViewNode<El::View, (ComponentNode<AsComponent<Self>, S, E>, El::Components), S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        Element::render(self, state, env, context)
    }

    fn update(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        Element::update(self, storage.scope(), state, env, context)
    }
}

pub struct AsComponent<T> {
    inner: T,
}

impl<El, Deps, S, E> Component<S, E> for AsComponent<Memoize<El, Deps>>
where
    El: Element<S, E>,
    Deps: PartialEq,
    S: State,
{
    type Element = El;

    fn render(&self) -> Self::Element {
        (self.inner.render)(&self.inner.deps)
    }
}
