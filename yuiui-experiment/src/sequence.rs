use either::Either;
use std::cmp::Ordering;
use std::fmt;
use std::mem;

use crate::component::Component;
use crate::context::Context;
use crate::element::{ComponentElement, Element, ViewElement};
use crate::hlist::{HCons, HList, HNil};
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

pub trait ElementSeq<S> {
    type Store: WidgetNodeSeq<S>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store;

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool;
}

pub trait WidgetNodeSeq<S> {
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context);
}

pub struct WidgetNodeStore<V: View<S>, CS, S> {
    node: WidgetNode<V, CS, S>,
    dirty: bool,
}

impl<V: View<S>, CS, S> WidgetNodeStore<V, CS, S> {
    fn new(node: WidgetNode<V, CS, S>) -> Self {
        Self { node, dirty: true }
    }
}

impl<V: View<S>, S> ElementSeq<S> for ViewElement<V, S> {
    type Store = WidgetNodeStore<<Self as Element<S>>::View, <Self as Element<S>>::Components, S>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        WidgetNodeStore::new(Element::build(self, state, context))
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        let has_changed = Element::rebuild(self, store.node.scope(), state, context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<C: Component<S>, S> ElementSeq<S> for ComponentElement<C, S> {
    type Store = WidgetNodeStore<<Self as Element<S>>::View, <Self as Element<S>>::Components, S>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        WidgetNodeStore::new(Element::build(self, state, context))
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        let has_changed = Element::rebuild(self, store.node.scope(), state, context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<V: View<S>, CS, S> WidgetNodeSeq<S> for WidgetNodeStore<V, CS, S> {
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        if self.dirty || mode.is_propagatable() {
            self.dirty = false;
            self.node.commit(mode, state, context);
        }
    }
}

impl<V, CS, S> fmt::Debug for WidgetNodeStore<V, CS, S>
where
    V: View<S> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S>>::Children: fmt::Debug,
    CS: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WidgetNodeStore")
            .field("node", &self.node)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<S> ElementSeq<S> for HNil {
    type Store = HNil;

    fn build(self, _state: &S, _context: &mut Context) -> Self::Store {
        HNil
    }

    fn rebuild(self, _nodes: &mut Self::Store, _state: &S, _context: &mut Context) -> bool {
        false
    }
}

impl<S> WidgetNodeSeq<S> for HNil {
    fn commit(&mut self, _mode: CommitMode, _state: &S, _context: &mut Context) {}
}

impl<H, T, S> ElementSeq<S> for HCons<H, T>
where
    H: ElementSeq<S>,
    T: ElementSeq<S> + HList,
    T::Store: HList,
{
    type Store = HCons<H::Store, T::Store>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        HCons(self.0.build(state, context), self.1.build(state, context))
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(&mut store.0, state, context);
        has_changed |= self.1.rebuild(&mut store.1, state, context);
        has_changed
    }
}

impl<H, T, S> WidgetNodeSeq<S> for HCons<H, T>
where
    H: WidgetNodeSeq<S>,
    T: WidgetNodeSeq<S> + HList,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        self.0.commit(mode, state, context);
        self.1.commit(mode, state, context);
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
            dirty: true,
        }
    }
}

impl<T, S> ElementSeq<S> for Vec<T>
where
    T: ElementSeq<S>,
{
    type Store = VecStore<T::Store>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        VecStore::new(
            self.into_iter()
                .map(|element| element.build(state, context))
                .collect(),
        )
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        let mut has_changed = false;

        store
            .staging
            .reserve_exact(self.len().saturating_sub(store.active.len()));
        store.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < store.active.len() {
                let node = &mut store.active[i];
                has_changed |= element.rebuild(node, state, context);
            } else {
                let j = i - store.active.len();
                if j < store.staging.len() {
                    let node = &mut store.staging[j];
                    has_changed |= element.rebuild(node, state, context);
                } else {
                    let node = element.build(state, context);
                    store.staging.push(node);
                    has_changed = true;
                }
            }
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T: WidgetNodeSeq<S>, S> WidgetNodeSeq<S> for VecStore<T> {
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        if self.dirty || mode.is_propagatable() {
            match self.new_len.cmp(&self.active.len()) {
                Ordering::Equal => {
                    for node in &mut self.active {
                        node.commit(mode, state, context);
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut self.active[..self.new_len] {
                        node.commit(mode, state, context);
                    }
                    for mut node in self.active.drain(self.new_len..) {
                        node.commit(CommitMode::Unmount, state, context);
                        self.staging.push(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut self.active {
                        node.commit(mode, state, context);
                    }
                    if !mode.is_unmount() {
                        for i in 0..self.active.len() - self.new_len {
                            let mut node = self.staging.swap_remove(i);
                            node.commit(CommitMode::Mount, state, context);
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
        Self { nodes, dirty: true }
    }
}

impl<T, const N: usize, S> ElementSeq<S> for [T; N]
where
    T: ElementSeq<S>,
{
    type Store = ArrayStore<T::Store, N>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        ArrayStore::new(self.map(|element| element.build(state, context)))
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut store.nodes[i];
            has_changed |= element.rebuild(node, state, context);
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T: WidgetNodeSeq<S>, S, const N: usize> WidgetNodeSeq<S> for ArrayStore<T, N> {
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                node.commit(mode, state, context);
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

impl<T, S> ElementSeq<S> for Option<T>
where
    T: ElementSeq<S>,
{
    type Store = OptionStore<T::Store>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        OptionStore::new(self.map(|element| element.build(state, context)))
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        match (store.active.as_mut(), self) {
            (Some(node), Some(element)) => {
                if element.rebuild(node, state, context) {
                    store.status = BuildStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = store.staging.as_mut() {
                    element.rebuild(node, state, context);
                } else {
                    store.staging = Some(element.build(state, context));
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

impl<T: WidgetNodeSeq<S>, S> WidgetNodeSeq<S> for OptionStore<T> {
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        if self.status == BuildStatus::Swapped {
            if let Some(nodes) = self.active.as_mut() {
                nodes.commit(CommitMode::Unmount, state, context);
            }
            mem::swap(&mut self.active, &mut self.staging);
            if !mode.is_unmount() {
                if let Some(nodes) = self.active.as_mut() {
                    nodes.commit(CommitMode::Mount, state, context);
                }
            }
            self.status = BuildStatus::Unchanged;
        } else if self.status == BuildStatus::Changed || mode.is_propagatable() {
            if let Some(nodes) = self.active.as_mut() {
                nodes.commit(mode, state, context);
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

impl<L: ElementSeq<S>, R: ElementSeq<S>, S> ElementSeq<S> for Either<L, R> {
    type Store = EitherStore<L::Store, R::Store>;

    fn build(self, state: &S, context: &mut Context) -> Self::Store {
        match self {
            Either::Left(element) => EitherStore::new(Either::Left(element.build(state, context))),
            Either::Right(element) => {
                EitherStore::new(Either::Right(element.build(state, context)))
            }
        }
    }

    fn rebuild(self, store: &mut Self::Store, state: &S, context: &mut Context) -> bool {
        match (store.active.as_mut(), self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.rebuild(node, state, context) {
                    store.status = BuildStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.rebuild(node, state, context) {
                    store.status = BuildStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Right(stagin_nodes)) => {
                        element.rebuild(stagin_nodes, state, context);
                    }
                    None => {
                        store.staging = Some(Either::Right(element.build(state, context)));
                    }
                    _ => unreachable!(),
                };
                store.status = BuildStatus::Swapped;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Left(node)) => {
                        element.rebuild(node, state, context);
                    }
                    None => {
                        store.staging = Some(Either::Left(element.build(state, context)));
                    }
                    _ => unreachable!(),
                }
                store.status = BuildStatus::Swapped;
                true
            }
        }
    }
}

impl<L, R, S> WidgetNodeSeq<S> for EitherStore<L, R>
where
    L: WidgetNodeSeq<S>,
    R: WidgetNodeSeq<S>,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        if self.status == BuildStatus::Swapped {
            match self.active.as_mut() {
                Either::Left(nodes) => nodes.commit(CommitMode::Unmount, state, context),
                Either::Right(nodes) => nodes.commit(CommitMode::Unmount, state, context),
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if !mode.is_unmount() {
                match self.active.as_mut() {
                    Either::Left(nodes) => nodes.commit(CommitMode::Mount, state, context),
                    Either::Right(nodes) => nodes.commit(CommitMode::Mount, state, context),
                }
            }
            self.status = BuildStatus::Unchanged;
        } else if self.status == BuildStatus::Changed || mode.is_propagatable() {
            match self.active.as_mut() {
                Either::Left(nodes) => nodes.commit(mode, state, context),
                Either::Right(nodes) => nodes.commit(mode, state, context),
            }
            self.status = BuildStatus::Unchanged;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitMode {
    Mount,
    Unmount,
    Update,
}

impl CommitMode {
    fn is_propagatable(&self) -> bool {
        match self {
            Self::Mount => true,
            Self::Unmount => true,
            Self::Update => false,
        }
    }

    fn is_unmount(&self) -> bool {
        match self {
            Self::Mount => false,
            Self::Unmount => true,
            Self::Update => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BuildStatus {
    Unchanged,
    Changed,
    Swapped,
}
