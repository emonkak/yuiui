use std::cmp::Ordering;
use std::mem;

use crate::context::Context;
use crate::element::Element;
use crate::hlist::{HCons, HList, HNil};
use crate::view_node::ViewNode;

pub trait ElementSeq: 'static {
    type Nodes;

    fn commit(nodes: &mut Self::Nodes, context: &mut Context);

    fn build(self, context: &mut Context) -> Self::Nodes;

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool;
}

impl<H, T> ElementSeq for HCons<H, T>
where
    H: Element,
    T: ElementSeq + HList,
    T::Nodes: HList,
{
    type Nodes = HCons<ViewNode<H::View, H::Components>, T::Nodes>;

    fn commit(nodes: &mut Self::Nodes, context: &mut Context) {
        nodes.0.commit(context);
        <T as ElementSeq>::commit(&mut nodes.1, context);
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        HCons(self.0.build(context), self.1.build(context))
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(nodes.0.scope(), context);
        has_changed |= self.1.rebuild(&mut nodes.1, context);
        has_changed
    }
}

impl ElementSeq for HNil {
    type Nodes = HNil;

    fn commit(_nodes: &mut Self::Nodes, _context: &mut Context) {}

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
    type Nodes = VecStore<ViewNode<T::View, T::Components>>;

    fn commit(nodes: &mut Self::Nodes, context: &mut Context) {
        if nodes.dirty {
            match nodes.new_len.cmp(&nodes.active.len()) {
                Ordering::Equal => {
                    for node in &mut nodes.active {
                        node.commit(context);
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut nodes.active[..nodes.new_len] {
                        node.commit(context);
                    }
                    for mut node in nodes.active.drain(nodes.new_len..) {
                        node.invalidate(context);
                        nodes.staging.push(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut nodes.active {
                        node.commit(context);
                    }
                    for i in 0..nodes.active.len() - nodes.new_len {
                        let mut node = nodes.staging.swap_remove(i);
                        node.commit(context);
                        nodes.active.push(node);
                    }
                }
            }
            nodes.dirty = false;
        }
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        VecStore::new(
            self.into_iter()
                .map(|element| element.build(context))
                .collect(),
        )
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        let mut has_changed = false;

        nodes
            .staging
            .reserve_exact(self.len().saturating_sub(nodes.active.len()));
        nodes.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < nodes.active.len() {
                let node = &mut nodes.active[i];
                has_changed |= element.rebuild(node.scope(), context);
            } else {
                let j = i - nodes.active.len();
                if j < nodes.staging.len() {
                    let node = &mut nodes.staging[j];
                    has_changed |= element.rebuild(node.scope(), context);
                } else {
                    let node = element.build(context);
                    nodes.staging.push(node);
                    has_changed = true;
                }
            }
        }

        nodes.dirty |= has_changed;

        has_changed
    }
}

#[derive(Debug)]
pub struct VecStore<T> {
    active: Vec<T>,
    staging: Vec<T>,
    new_len: usize,
    dirty: bool,
}

impl<T> VecStore<T> {
    fn new(active: Vec<T>) -> Self {
        Self {
            staging: Vec::with_capacity(active.len()),
            new_len: active.len(),
            active,
            dirty: false,
        }
    }
}

impl<T> ElementSeq for Option<T>
where
    T: Element,
{
    type Nodes = OptionStore<ViewNode<T::View, T::Components>>;

    fn commit(nodes: &mut Self::Nodes, context: &mut Context) {
        if nodes.swap {
            if let Some(node) = nodes.active.as_mut() {
                node.invalidate(context);
            }
            mem::swap(&mut nodes.active, &mut nodes.staging);
            nodes.swap = false;
        }
        if nodes.dirty {
            if let Some(node) = nodes.active.as_mut() {
                node.commit(context);
            }
            nodes.dirty = false;
        }
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        OptionStore::new(self.map(|element| element.build(context)))
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        match (self, nodes.active.as_mut()) {
            (Some(element), Some(node)) => {
                let has_changed = element.rebuild(node.scope(), context);
                nodes.dirty |= has_changed;
                has_changed
            }
            (Some(element), None) => {
                let has_changed = if let Some(node) = nodes.staging.as_mut() {
                    element.rebuild(node.scope(), context)
                } else {
                    nodes.staging = Some(element.build(context));
                    true
                };
                nodes.swap = true;
                nodes.dirty = true;
                has_changed
            }
            (None, Some(_)) => {
                nodes.staging = None;
                nodes.swap = true;
                nodes.dirty = true;
                true
            }
            (None, None) => false,
        }
    }
}

#[derive(Debug)]
pub struct OptionStore<T> {
    active: Option<T>,
    staging: Option<T>,
    swap: bool,
    dirty: bool,
}

impl<T> OptionStore<T> {
    fn new(active: Option<T>) -> Self {
        Self {
            active,
            staging: None,
            swap: false,
            dirty: false,
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
    type Nodes = EitherStore<ViewNode<L::View, L::Components>, ViewNode<R::View, R::Components>>;

    fn commit(nodes: &mut Self::Nodes, context: &mut Context) {
        if nodes.swap {
            match nodes.active.as_mut() {
                Either::Left(node) => node.invalidate(context),
                Either::Right(node) => node.invalidate(context),
            }
            mem::swap(&mut nodes.active, nodes.staging.as_mut().unwrap());
            nodes.swap = false;
        }
        if nodes.dirty {
            match nodes.active.as_mut() {
                Either::Left(node) => node.commit(context),
                Either::Right(node) => node.commit(context),
            }
            nodes.dirty = false;
        }
    }

    fn build(self, context: &mut Context) -> Self::Nodes {
        match self {
            Either::Left(element) => EitherStore::new(Either::Left(element.build(context))),
            Either::Right(element) => EitherStore::new(Either::Right(element.build(context))),
        }
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        match (self, nodes.active.as_mut()) {
            (Either::Left(element), Either::Left(node)) => {
                let has_changed = element.rebuild(node.scope(), context);
                nodes.dirty |= true;
                has_changed
            }
            (Either::Right(element), Either::Right(node)) => {
                let has_changed = element.rebuild(node.scope(), context);
                nodes.dirty |= true;
                has_changed
            }
            (Either::Left(element), Either::Right(_)) => {
                let has_changed = match nodes.staging.take() {
                    Some(Either::Left(mut node)) => {
                        let has_changed = element.rebuild(node.scope(), context);
                        nodes.staging = Some(Either::Left(node));
                        has_changed
                    }
                    None => {
                        nodes.staging = Some(Either::Left(element.build(context)));
                        true
                    }
                    _ => {
                        unreachable!();
                    }
                };
                nodes.swap = true;
                nodes.dirty = true;
                has_changed
            }
            (Either::Right(element), Either::Left(_)) => {
                let has_changed = match nodes.staging.take() {
                    Some(Either::Right(mut node)) => {
                        let has_changed = element.rebuild(node.scope(), context);
                        nodes.staging = Some(Either::Right(node));
                        has_changed
                    }
                    None => {
                        nodes.staging = Some(Either::Right(element.build(context)));
                        true
                    }
                    _ => {
                        unreachable!();
                    }
                };
                nodes.swap = true;
                nodes.dirty = true;
                has_changed
            }
        }
    }
}

#[derive(Debug)]
pub struct EitherStore<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    swap: bool,
    dirty: bool,
}

impl<L, R> EitherStore<L, R> {
    fn new(active: Either<L, R>) -> Self {
        Self {
            active,
            staging: None,
            swap: false,
            dirty: false,
        }
    }
}
