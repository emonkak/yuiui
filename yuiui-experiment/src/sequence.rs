use either::Either;
use std::cmp::Ordering;
use std::fmt;
use std::mem;

use crate::context::Context;
use crate::element::Element;
use crate::hlist::{HCons, HList, HNil};
use crate::view::View;
use crate::widget::{CommitMode, Widget, WidgetNode};

pub trait ElementSeq {
    type Store: WidgetNodeSeq;

    fn build(self, context: &mut Context) -> Self::Store;

    fn rebuild(self, store: &mut Self::Store, context: &mut Context) -> bool;
}

pub trait WidgetNodeSeq {
    fn commit(&mut self, mode: CommitMode, context: &mut Context);
}

pub struct WidgetNodeStore<V: View, CS> {
    node: WidgetNode<V, CS>,
    dirty: bool,
}

impl<V: View, CS> WidgetNodeStore<V, CS> {
    fn new(node: WidgetNode<V, CS>) -> Self {
        Self {
            node,
            dirty: false,
        }
    }
}

impl<E: Element> ElementSeq for E {
    type Store = WidgetNodeStore<E::View, E::Components>;

    fn build(self, context: &mut Context) -> Self::Store {
        WidgetNodeStore::new(self.build(context))
    }

    fn rebuild(self, store: &mut Self::Store, context: &mut Context) -> bool {
        let has_changed = self.rebuild(store.node.scope(), context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<V: View, CS> WidgetNodeSeq for WidgetNodeStore<V, CS> {
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        if self.dirty || mode.is_propagatable() {
            self.dirty = false;
            self.node.commit(mode, context);
        }
    }
}

impl<V: View + fmt::Debug, CS> fmt::Debug for WidgetNodeStore<V, CS>
where
    V: View + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget>::Children: fmt::Debug,
    CS: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WidgetNodeStore")
            .field("node", &self.node)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl ElementSeq for HNil {
    type Store = HNil;

    fn build(self, _context: &mut Context) -> Self::Store {
        HNil
    }

    fn rebuild(self, _nodes: &mut Self::Store, _context: &mut Context) -> bool {
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
    T::Store: HList,
{
    type Store = HCons<H::Store, T::Store>;

    fn build(self, context: &mut Context) -> Self::Store {
        HCons(self.0.build(context), self.1.build(context))
    }

    fn rebuild(self, store: &mut Self::Store, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(&mut store.0, context);
        has_changed |= self.1.rebuild(&mut store.1, context);
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
    type Store = VecStore<T::Store>;

    fn build(self, context: &mut Context) -> Self::Store {
        VecStore::new(
            self.into_iter()
                .map(|element| element.build(context))
                .collect(),
        )
    }

    fn rebuild(self, store: &mut Self::Store, context: &mut Context) -> bool {
        let mut has_changed = false;

        store
            .staging
            .reserve_exact(self.len().saturating_sub(store.active.len()));
        store.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < store.active.len() {
                let node = &mut store.active[i];
                has_changed |= element.rebuild(node, context);
            } else {
                let j = i - store.active.len();
                if j < store.staging.len() {
                    let node = &mut store.staging[j];
                    has_changed |= element.rebuild(node, context);
                } else {
                    let node = element.build(context);
                    store.staging.push(node);
                    has_changed = true;
                }
            }
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T: WidgetNodeSeq> WidgetNodeSeq for VecStore<T> {
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        if self.dirty || mode.is_propagatable() {
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
                    if !mode.is_unmount() {
                        for i in 0..self.active.len() - self.new_len {
                            let mut node = self.staging.swap_remove(i);
                            node.commit(CommitMode::Mount, context);
                            self.active.push(node);
                        }
                    }
                }
            }
            self.dirty = false;
        }
    }
}

#[derive(Debug)]
pub struct ArrayStore<T, const N: usize> {
    nodes: [T; N],
    dirty: bool,
}

impl<T, const N: usize> ArrayStore<T, N> {
    fn new(nodes: [T; N]) -> Self {
        Self {
            nodes,
            dirty: false,
        }
    }
}

impl<T, const N: usize> ElementSeq for [T; N]
where
    T: ElementSeq,
{
    type Store = ArrayStore<T::Store, N>;

    fn build(self, context: &mut Context) -> Self::Store {
        ArrayStore::new(
            self.map(|element| element.build(context))
        )
    }

    fn rebuild(self, store: &mut Self::Store, context: &mut Context) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut store.nodes[i];
            has_changed |= element.rebuild(node, context);
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T: WidgetNodeSeq, const N: usize> WidgetNodeSeq for ArrayStore<T, N> {
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                node.commit(mode, context);
            }
            self.dirty = false;
        }
    }
}

#[derive(Debug)]
pub struct OptionStore<T> {
    active: Option<T>,
    staging: Option<T>,
    status: BuildStatus,
}

impl<T> OptionStore<T> {
    fn new(active: Option<T>) -> Self {
        Self {
            active,
            staging: None,
            status: BuildStatus::Unchanged,
        }
    }
}

impl<T> ElementSeq for Option<T>
where
    T: ElementSeq,
{
    type Store = OptionStore<T::Store>;

    fn build(self, context: &mut Context) -> Self::Store {
        OptionStore::new(self.map(|element| element.build(context)))
    }

    fn rebuild(self, store: &mut Self::Store, context: &mut Context) -> bool {
        match (store.active.as_mut(), self) {
            (Some(node), Some(element)) => {
                if element.rebuild(node, context) {
                    store.status = BuildStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = store.staging.as_mut() {
                    element.rebuild(node, context);
                } else {
                    store.staging = Some(element.build(context));
                }
                store.status = BuildStatus::Swapped;
                true
            }
            (Some(_), None) => {
                assert!(store.staging.is_none());
                store.status = BuildStatus::Swapped;
                true
            }
            (None, None) => false,
        }
    }
}

impl<T: WidgetNodeSeq> WidgetNodeSeq for OptionStore<T> {
    fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        if self.status == BuildStatus::Swapped {
            if let Some(nodes) = self.active.as_mut() {
                nodes.commit(CommitMode::Unmount, context);
            }
            mem::swap(&mut self.active, &mut self.staging);
            if !mode.is_unmount() {
                if let Some(nodes) = self.active.as_mut() {
                    nodes.commit(CommitMode::Mount, context);
                }
            }
            self.status = BuildStatus::Unchanged;
        } else if self.status == BuildStatus::Changed || mode.is_propagatable() {
            if let Some(nodes) = self.active.as_mut() {
                nodes.commit(mode, context);
            }
            self.status = BuildStatus::Unchanged;
        }
    }
}

#[derive(Debug)]
pub struct EitherStore<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    status: BuildStatus,
}

impl<L, R> EitherStore<L, R> {
    fn new(active: Either<L, R>) -> Self {
        Self {
            active,
            staging: None,
            status: BuildStatus::Unchanged,
        }
    }
}

impl<L: ElementSeq, R: ElementSeq> ElementSeq for Either<L, R> {
    type Store = EitherStore<L::Store, R::Store>;

    fn build(self, context: &mut Context) -> Self::Store {
        match self {
            Either::Left(element) => EitherStore::new(Either::Left(element.build(context))),
            Either::Right(element) => EitherStore::new(Either::Right(element.build(context))),
        }
    }

    fn rebuild(self, store: &mut Self::Store, context: &mut Context) -> bool {
        match (store.active.as_mut(), self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.rebuild(node, context) {
                    store.status = BuildStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.rebuild(node, context) {
                    store.status = BuildStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Right(stagin_nodes)) => {
                        element.rebuild(stagin_nodes, context);
                    }
                    None => {
                        store.staging = Some(Either::Right(element.build(context)));
                    }
                    _ => unreachable!()
                };
                store.status = BuildStatus::Swapped;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Left(node)) => {
                        element.rebuild(node, context);
                    }
                    None => {
                        store.staging = Some(Either::Left(element.build(context)));
                    }
                    _ => unreachable!()
                }
                store.status = BuildStatus::Swapped;
                true
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
        if self.status == BuildStatus::Swapped {
            match self.active.as_mut() {
                Either::Left(nodes) => nodes.commit(CommitMode::Unmount, context),
                Either::Right(nodes) => nodes.commit(CommitMode::Unmount, context),
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if !mode.is_unmount() {
                match self.active.as_mut() {
                    Either::Left(nodes) => nodes.commit(CommitMode::Mount, context),
                    Either::Right(nodes) => nodes.commit(CommitMode::Mount, context),
                }
            }
            self.status = BuildStatus::Unchanged;
        } else if self.status == BuildStatus::Changed || mode.is_propagatable() {
            match self.active.as_mut() {
                Either::Left(nodes) => nodes.commit(mode, context),
                Either::Right(nodes) => nodes.commit(mode, context),
            }
            self.status = BuildStatus::Unchanged;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BuildStatus {
    Unchanged,
    Changed,
    Swapped,
}
