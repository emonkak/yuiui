use either::Either;
use std::cmp::Ordering;
use std::mem;

use crate::context::Context;
use crate::element::Element;
use crate::hlist::{HCons, HList, HNil};
use crate::view::View;
use crate::widget::{CommitMode, WidgetNode};

pub trait ElementSeq {
    type Nodes: WidgetNodeSeq;

    fn build(self, context: &mut Context) -> Self::Nodes;

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool;
}

pub trait WidgetNodeSeq {
    fn commit(&mut self, mode: CommitMode, context: &mut Context);
}

impl<E: Element> ElementSeq for E {
    type Nodes = WidgetNode<E::View, E::Components>;

    fn build(self, context: &mut Context) -> Self::Nodes {
        self.build(context)
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        self.rebuild(nodes.scope(), context)
    }
}

impl<V: View, CS> WidgetNodeSeq for WidgetNode<V, CS> {
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        self.commit(mode, context)
    }
}

impl ElementSeq for HNil {
    type Nodes = HNil;

    fn build(self, _context: &mut Context) -> Self::Nodes {
        HNil
    }

    fn rebuild(self, _nodes: &mut Self::Nodes, _context: &mut Context) -> bool {
        false
    }
}

impl WidgetNodeSeq for HNil {
    fn commit(&mut self, _mode: CommitMode, _context: &mut Context) {}
}

impl<H, T> ElementSeq for HCons<H, T>
where
    H: ElementSeq,
    T: ElementSeq + HList,
    T::Nodes: HList,
{
    type Nodes = HCons<H::Nodes, T::Nodes>;

    fn build(self, context: &mut Context) -> Self::Nodes {
        HCons(self.0.build(context), self.1.build(context))
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(&mut nodes.0, context);
        has_changed |= self.1.rebuild(&mut nodes.1, context);
        has_changed
    }
}

impl<H, T> WidgetNodeSeq for HCons<H, T>
where
    H: WidgetNodeSeq,
    T: WidgetNodeSeq + HList,
{
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        self.0.commit(mode, context);
        self.1.commit(mode, context);
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

impl<T> ElementSeq for Vec<T>
where
    T: ElementSeq,
{
    type Nodes = VecStore<T::Nodes>;

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
                has_changed |= element.rebuild(node, context);
            } else {
                let j = i - nodes.active.len();
                if j < nodes.staging.len() {
                    let node = &mut nodes.staging[j];
                    has_changed |= element.rebuild(node, context);
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

impl<T: WidgetNodeSeq> WidgetNodeSeq for VecStore<T> {
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        if self.dirty {
            match self.new_len.cmp(&self.active.len()) {
                Ordering::Equal => {
                    for node in &mut self.active {
                        node.commit(mode, context);
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut self.active[..self.new_len] {
                        node.commit(mode, context);
                    }
                    for mut node in self.active.drain(self.new_len..) {
                        node.commit(CommitMode::Unmount, context);
                        self.staging.push(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut self.active {
                        node.commit(mode, context);
                    }
                    for i in 0..self.active.len() - self.new_len {
                        let mut node = self.staging.swap_remove(i);
                        node.commit(mode, context);
                        self.active.push(node);
                    }
                }
            }
            self.dirty = false;
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

impl<T> ElementSeq for Option<T>
where
    T: ElementSeq,
{
    type Nodes = OptionStore<T::Nodes>;

    fn build(self, context: &mut Context) -> Self::Nodes {
        OptionStore::new(self.map(|element| element.build(context)))
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        match (nodes.active.as_mut(), self) {
            (Some(node), Some(element)) => {
                let has_changed = element.rebuild(node, context);
                nodes.swap = false;
                nodes.dirty |= has_changed;
                has_changed
            }
            (None, Some(element)) => {
                let has_changed = if let Some(node) = nodes.staging.as_mut() {
                    element.rebuild(node, context)
                } else {
                    nodes.staging = Some(element.build(context));
                    true
                };
                nodes.swap = true;
                nodes.dirty |= has_changed;
                has_changed
            }
            (Some(_), None) => {
                assert!(nodes.staging.is_none());
                nodes.swap = true;
                nodes.dirty = true;
                true
            }
            (None, None) => false,
        }
    }
}

impl<T: WidgetNodeSeq> WidgetNodeSeq for OptionStore<T> {
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        if self.swap {
            if let Some(nodes) = self.active.as_mut() {
                nodes.commit(CommitMode::Unmount, context);
            }
            mem::swap(&mut self.active, &mut self.staging);
            self.swap = false;
        }
        if self.dirty {
            if let Some(nodes) = self.active.as_mut() {
                nodes.commit(mode, context);
            }
            self.dirty = false;
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

impl<L: ElementSeq, R: ElementSeq> ElementSeq for Either<L, R> {
    type Nodes = EitherStore<L::Nodes, R::Nodes>;

    fn build(self, context: &mut Context) -> Self::Nodes {
        match self {
            Either::Left(element) => EitherStore::new(Either::Left(element.build(context))),
            Either::Right(element) => EitherStore::new(Either::Right(element.build(context))),
        }
    }

    fn rebuild(self, nodes: &mut Self::Nodes, context: &mut Context) -> bool {
        match (nodes.active.as_mut(), self) {
            (Either::Left(node), Either::Left(element)) => {
                let has_changed = element.rebuild(node, context);
                nodes.dirty |= has_changed;
                has_changed
            }
            (Either::Right(node), Either::Right(element)) => {
                let has_changed = element.rebuild(node, context);
                nodes.dirty |= has_changed;
                has_changed
            }
            (Either::Left(_), Either::Right(element)) => {
                let has_changed = match nodes.staging.as_mut() {
                    Some(Either::Right(stagin_nodes)) => element.rebuild(stagin_nodes, context),
                    None => {
                        nodes.staging = Some(Either::Right(element.build(context)));
                        true
                    }
                    _ => {
                        unreachable!();
                    }
                };
                nodes.swap = true;
                nodes.dirty |= has_changed;
                has_changed
            }
            (Either::Right(_), Either::Left(element)) => {
                let has_changed = match nodes.staging.as_mut() {
                    Some(Either::Left(node)) => element.rebuild(node, context),
                    None => {
                        nodes.staging = Some(Either::Left(element.build(context)));
                        true
                    }
                    _ => {
                        unreachable!();
                    }
                };
                nodes.swap = true;
                nodes.dirty |= has_changed;
                has_changed
            }
        }
    }
}

impl<L, R> WidgetNodeSeq for EitherStore<L, R>
where
    L: WidgetNodeSeq,
    R: WidgetNodeSeq,
{
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        if self.swap {
            match self.active.as_mut() {
                Either::Left(nodes) => nodes.commit(CommitMode::Unmount, context),
                Either::Right(nodes) => nodes.commit(CommitMode::Unmount, context),
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            self.swap = false;
        }
        if self.dirty {
            match self.active.as_mut() {
                Either::Left(nodes) => nodes.commit(mode, context),
                Either::Right(nodes) => nodes.commit(mode, context),
            }
            self.dirty = false;
        }
    }
}
