use crate::context::Context;
use crate::element::Element;
use crate::node::UINode;
use crate::hlist::{HList, HCons, HNil};

pub trait ElementSeq: 'static {
    type Nodes;

    fn invalidate(nodes: &mut Self::Nodes, context: &mut Context);

    fn build(self, context: &mut Context) -> Self::Nodes;

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool;
}

impl<H, T> ElementSeq for HCons<H, T>
where
    H: Element,
    T: ElementSeq + HList,
    T::Nodes: HList,
{
    type Nodes = HCons<UINode<H::View, H::Components>, T::Nodes>;

    fn invalidate(nodes: &mut Self::Nodes, context: &mut Context) {
        nodes.head.invalidate(context);
        <T as ElementSeq>::invalidate(&mut nodes.tail, context);
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        HCons {
            head: self.head.build(context),
            tail: self.tail.build(context),
        }
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.rebuild(
            &mut nodes.head.view,
            &mut nodes.head.children,
            &mut nodes.head.components,
            context,
        );
        has_changed |= self.tail.rebuild(&mut nodes.tail, context);
        has_changed
    }
}

impl ElementSeq for HNil {
    type Nodes = HNil;

    fn invalidate(_nodes: &mut Self::Nodes, _context: &mut Context) {}

    fn build(self, _context: &mut Context) -> Self::Nodes {
        HNil
    }

    fn rebuild(self, _nodes: &mut Self::Nodes, _context: &mut Context) -> bool {
        false
    }
}

impl<T> ElementSeq for Vec<T>
where
    T: Element,
{
    type Nodes = Vec<UINode<T::View, T::Components>>;

    fn invalidate(nodes: &mut Self::Nodes, context: &mut Context) {
        for node in nodes {
            node.invalidate(context);
        }
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        self.into_iter()
            .map(|element| element.build(context))
            .collect()
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        if self.len() < nodes.len() {
            for mut node in nodes.drain(nodes.len() - self.len() - 1..) {
                node.invalidate(context);
            }
        } else {
            nodes.reserve_exact(self.len());
        }

        let reuse_len = self.len().min(nodes.len());
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            if i < reuse_len {
                let node = &mut nodes[i];
                if element.rebuild(
                    &mut node.view,
                    &mut node.children,
                    &mut node.components,
                    context,
                ) {
                    has_changed = true;
                }
            } else {
                let node = element.build(context);
                nodes.push(node);
                has_changed = true;
            }
        }

        has_changed
    }
}

impl<T> ElementSeq for Option<T>
where
    T: Element,
{
    type Nodes = Option<UINode<T::View, T::Components>>;

    fn invalidate(nodes: &mut Self::Nodes, context: &mut Context) {
        if let Some(node) = nodes {
            node.invalidate(context);
        }
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        self.map(|element| element.build(context))
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        match (self, nodes.as_mut()) {
            (Some(element), Some(node)) => element.rebuild(
                &mut node.view,
                &mut node.children,
                &mut node.components,
                context,
            ),
            (Some(element), None) => {
                *nodes = Some(element.build(context));
                true
            }
            (None, Some(node)) => {
                node.invalidate(context);
                *nodes = None;
                true
            }
            (None, None) => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Either::Left(value) => Either::Left(value),
            Either::Right(value) => Either::Right(value),
        }
    }

    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match self {
            Either::Left(value) => Either::Left(value),
            Either::Right(value) => Either::Right(value),
        }
    }
}

impl<L, R> ElementSeq for Either<L, R>
where
    L: Element,
    R: Element,
{
    type Nodes = Either<UINode<L::View, L::Components>, UINode<R::View, R::Components>>;

    fn invalidate(nodes: &mut Self::Nodes, context: &mut Context) {
        match nodes {
            Either::Left(node) => node.invalidate(context),
            Either::Right(node) => node.invalidate(context),
        }
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        match self {
            Either::Left(element) => Either::Left(element.build(context)),
            Either::Right(element) => Either::Right(element.build(context)),
        }
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        match (self, nodes.as_mut()) {
            (Either::Left(element), Either::Left(node)) => element.rebuild(
                &mut node.view,
                &mut node.children,
                &mut node.components,
                context,
            ),
            (Either::Right(element), Either::Right(node)) => element.rebuild(
                &mut node.view,
                &mut node.children,
                &mut node.components,
                context,
            ),
            (Either::Left(element), Either::Right(node)) => {
                node.invalidate(context);
                *nodes = Either::Left(element.build(context));
                true
            }
            (Either::Right(element), Either::Left(node)) => {
                node.invalidate(context);
                *nodes = Either::Right(element.build(context));
                true
            }
        }
    }
}
