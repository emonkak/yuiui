use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, RenderContext};
use crate::element::{Element, ElementSeq};
use crate::id::Id;
use crate::view::View;
use crate::view_node::{CommitMode, Traversable, ViewNode, ViewNodeSeq};

#[derive(Debug)]
pub struct ArrayStorage<T, const N: usize> {
    nodes: [T; N],
    dirty: bool,
}

impl<T, const N: usize> ArrayStorage<T, N> {
    fn new(nodes: [T; N]) -> Self {
        Self { nodes, dirty: true }
    }
}

impl<Element, S, M, E, const N: usize> ElementSeq<S, M, E> for [Element; N]
where
    Element: self::Element<S, M, E>,
{
    type Storage = ArrayStorage<ViewNode<Element::View, Element::Components, S, M, E>, N>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        ArrayStorage::new(self.map(|element| element.render(context)))
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut storage.nodes[i];
            has_changed |= element.update(node.into(), context);
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<'a, V, CS, S, M, E, const N: usize> ViewNodeSeq<S, M, E>
    for ArrayStorage<ViewNode<V, CS, S, M, E>, N>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    const SIZE_HINT: (usize, Option<usize>) = (N, Some(N));

    fn len(&self) -> usize {
        N
    }

    fn commit(&mut self, mode: CommitMode, context: &mut CommitContext<S, M, E>) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagable() {
            for node in &mut self.nodes {
                result |= node.commit(mode, context);
            }
            self.dirty = false;
        }
        result
    }

    fn gc(&mut self) {
        for node in &mut self.nodes {
            node.gc();
        }
    }
}

impl<Visitor, Context, V, CS, S, M, E, const N: usize> Traversable<Visitor, Context>
    for ArrayStorage<ViewNode<V, CS, S, M, E>, N>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
    ViewNode<V, CS, S, M, E>: Traversable<Visitor, Context>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut Context) {
        for node in &mut self.nodes {
            node.for_each(visitor, context);
        }
    }

    fn for_id(&mut self, id: Id, visitor: &mut Visitor, context: &mut Context) -> bool {
        if let Ok(index) = self.nodes.binary_search_by_key(&id, |node| node.id) {
            let node = &mut self.nodes[index];
            return node.for_id(id, visitor, context);
        }
        false
    }
}
