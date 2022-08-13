use either::Either;
use std::any::TypeId;
use std::cmp::Ordering;
use std::fmt;
use std::mem;

use crate::component::{Component, ComponentStack};
use crate::context::{EffectContext, RenderContext};
use crate::element::{ComponentElement, Element, ViewElement};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::hlist::{HCons, HList, HNil};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

pub trait ElementSeq<S: State> {
    type Store: WidgetNodeSeq<S>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store;

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool;
}

pub trait WidgetNodeSeq<S: State> {
    fn event_mask() -> EventMask;

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>);

    fn event<E: 'static>(
        &self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult;

    fn internal_event(
        &self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult;
}

impl<V, S> ElementSeq<S> for ViewElement<V, S>
where
    V: View<S>,
    S: State,
{
    type Store = WidgetNodeStore<<Self as Element<S>>::View, <Self as Element<S>>::Components, S>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let has_changed = Element::update(self, store.node.scope(), state, context);
        store.dirty = has_changed;
        has_changed
    }
}

impl<C, S> ElementSeq<S> for ComponentElement<C, S>
where
    C: Component<S>,
    S: State,
{
    type Store = WidgetNodeStore<<Self as Element<S>>::View, <Self as Element<S>>::Components, S>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        WidgetNodeStore::new(Element::render(self, state, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let has_changed = Element::update(self, store.node.scope(), state, context);
        store.dirty = has_changed;
        has_changed
    }
}

pub struct WidgetNodeStore<V: View<S>, CS, S: State> {
    node: WidgetNode<V, CS, S>,
    dirty: bool,
}

impl<V, CS, S> WidgetNodeStore<V, CS, S>
where
    V: View<S>,
    S: State,
{
    fn new(node: WidgetNode<V, CS, S>) -> Self {
        Self { node, dirty: true }
    }
}

impl<V, CS, S> WidgetNodeSeq<S> for WidgetNodeStore<V, CS, S>
where
    V: View<S>,
    CS: ComponentStack<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        let mut event_mask = <V::Widget as Widget<S>>::Children::event_mask();
        event_mask.add(TypeId::of::<<V::Widget as Widget<S>>::Event>());
        event_mask
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            self.dirty = false;
            self.node.commit(mode, state, context);
        }
    }

    fn event<E: 'static>(
        &self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        self.node.event(event, state, context)
    }

    fn internal_event(
        &self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        self.node.internal_event(event, state, context)
    }
}

impl<V, CS, S> fmt::Debug for WidgetNodeStore<V, CS, S>
where
    V: View<S> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S>>::Children: fmt::Debug,
    CS: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WidgetNodeStore")
            .field("node", &self.node)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<S: State> ElementSeq<S> for HNil {
    type Store = HNil;

    fn render(self, _state: &S, _context: &mut RenderContext) -> Self::Store {
        HNil
    }

    fn update(self, _nodes: &mut Self::Store, _state: &S, _context: &mut RenderContext) -> bool {
        false
    }
}

impl<S: State> WidgetNodeSeq<S> for HNil {
    fn event_mask() -> EventMask {
        EventMask::new()
    }

    fn commit(&mut self, _mode: CommitMode, _state: &S, _context: &mut EffectContext<S>) {}

    fn event<E: 'static>(
        &self,
        _event: &E,
        _state: &S,
        _context: &mut EffectContext<S>,
    ) -> EventResult {
        EventResult::Ignored
    }

    fn internal_event(
        &self,
        _event: &InternalEvent,
        _state: &S,
        _context: &mut EffectContext<S>,
    ) -> EventResult {
        EventResult::Ignored
    }
}

impl<H, T, S> ElementSeq<S> for HCons<H, T>
where
    H: ElementSeq<S>,
    T: ElementSeq<S> + HList,
    T::Store: HList,
    S: State,
{
    type Store = HCons<H::Store, T::Store>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        HCons(self.0.render(state, context), self.1.render(state, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.update(&mut store.0, state, context);
        has_changed |= self.1.update(&mut store.1, state, context);
        has_changed
    }
}

impl<H, T, S> WidgetNodeSeq<S> for HCons<H, T>
where
    H: WidgetNodeSeq<S>,
    T: WidgetNodeSeq<S> + HList,
    S: State,
{
    fn event_mask() -> EventMask {
        H::event_mask().merge(T::event_mask())
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        self.0.commit(mode, state, context);
        self.1.commit(mode, state, context);
    }

    fn event<E: 'static>(
        &self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        self.0
            .event(event, state, context)
            .merge(self.1.event(event, state, context))
    }

    fn internal_event(
        &self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if self.0.internal_event(event, state, context) == EventResult::Captured {
            EventResult::Captured
        } else {
            self.1.internal_event(event, state, context)
        }
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
    S: State,
{
    type Store = VecStore<T::Store>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        VecStore::new(
            self.into_iter()
                .map(|element| element.render(state, context))
                .collect(),
        )
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let mut has_changed = false;

        store
            .staging
            .reserve_exact(self.len().saturating_sub(store.active.len()));
        store.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < store.active.len() {
                let node = &mut store.active[i];
                has_changed |= element.update(node, state, context);
            } else {
                let j = i - store.active.len();
                if j < store.staging.len() {
                    let node = &mut store.staging[j];
                    has_changed |= element.update(node, state, context);
                } else {
                    let node = element.render(state, context);
                    store.staging.push(node);
                    has_changed = true;
                }
            }
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T, S> WidgetNodeSeq<S> for VecStore<T>
where
    T: WidgetNodeSeq<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
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

    fn event<E: 'static>(
        &self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let mut result = EventResult::Ignored;
        for node in &self.active {
            result = result.merge(node.event(event, state, context));
        }
        result
    }

    fn internal_event(
        &self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        for node in &self.active {
            if node.internal_event(event, state, context) == EventResult::Captured {
                return EventResult::Captured;
            }
        }
        EventResult::Ignored
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

impl<T, S, const N: usize> ElementSeq<S> for [T; N]
where
    T: ElementSeq<S>,
    S: State,
{
    type Store = ArrayStore<T::Store, N>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        ArrayStore::new(self.map(|element| element.render(state, context)))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut store.nodes[i];
            has_changed |= element.update(node, state, context);
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T, S, const N: usize> WidgetNodeSeq<S> for ArrayStore<T, N>
where
    T: WidgetNodeSeq<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                node.commit(mode, state, context);
            }
            self.dirty = false;
        }
    }

    fn event<E: 'static>(
        &self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let mut result = EventResult::Ignored;
        for node in &self.nodes {
            result = node.event(event, state, context);
        }
        result
    }

    fn internal_event(
        &self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        for node in &self.nodes {
            if node.internal_event(event, state, context) == EventResult::Captured {
                return EventResult::Captured;
            }
        }
        EventResult::Ignored
    }
}

#[derive(Debug)]
pub struct OptionStore<T> {
    active: Option<T>,
    staging: Option<T>,
    status: RenderStatus,
}

impl<T> OptionStore<T> {
    fn new(active: Option<T>) -> Self {
        Self {
            active,
            staging: None,
            status: RenderStatus::Unchanged,
        }
    }
}

impl<T, S> ElementSeq<S> for Option<T>
where
    T: ElementSeq<S>,
    S: State,
{
    type Store = OptionStore<T::Store>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        OptionStore::new(self.map(|element| element.render(state, context)))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        match (store.active.as_mut(), self) {
            (Some(node), Some(element)) => {
                if element.update(node, state, context) {
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = store.staging.as_mut() {
                    element.update(node, state, context);
                } else {
                    store.staging = Some(element.render(state, context));
                }
                store.status = RenderStatus::Swapped;
                true
            }
            (Some(_), None) => {
                assert!(store.staging.is_none());
                store.status = RenderStatus::Swapped;
                true
            }
            (None, None) => false,
        }
    }
}

impl<T, S> WidgetNodeSeq<S> for OptionStore<T>
where
    T: WidgetNodeSeq<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.status == RenderStatus::Swapped {
            if let Some(node) = self.active.as_mut() {
                node.commit(CommitMode::Unmount, state, context);
            }
            mem::swap(&mut self.active, &mut self.staging);
            if !mode.is_unmount() {
                if let Some(node) = self.active.as_mut() {
                    node.commit(CommitMode::Mount, state, context);
                }
            }
            self.status = RenderStatus::Unchanged;
        } else if self.status == RenderStatus::Changed || mode.is_propagatable() {
            if let Some(node) = self.active.as_mut() {
                node.commit(mode, state, context);
            }
            self.status = RenderStatus::Unchanged;
        }
    }

    fn event<E: 'static>(
        &self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if let Some(node) = self.active.as_ref() {
            node.event(event, state, context)
        } else {
            EventResult::Ignored
        }
    }

    fn internal_event(
        &self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if let Some(node) = self.active.as_ref() {
            node.internal_event(event, state, context)
        } else {
            EventResult::Ignored
        }
    }
}

#[derive(Debug)]
pub struct EitherStore<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    status: RenderStatus,
}

impl<L, R> EitherStore<L, R> {
    fn new(active: Either<L, R>) -> Self {
        Self {
            active,
            staging: None,
            status: RenderStatus::Unchanged,
        }
    }
}

impl<L, R, S> ElementSeq<S> for Either<L, R>
where
    L: ElementSeq<S>,
    R: ElementSeq<S>,
    S: State,
{
    type Store = EitherStore<L::Store, R::Store>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        match self {
            Either::Left(element) => EitherStore::new(Either::Left(element.render(state, context))),
            Either::Right(element) => {
                EitherStore::new(Either::Right(element.render(state, context)))
            }
        }
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        match (store.active.as_mut(), self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update(node, state, context) {
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update(node, state, context) {
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Right(node)) => {
                        element.update(node, state, context);
                    }
                    None => {
                        store.staging = Some(Either::Right(element.render(state, context)));
                    }
                    _ => unreachable!(),
                };
                store.status = RenderStatus::Swapped;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Left(node)) => {
                        element.update(node, state, context);
                    }
                    None => {
                        store.staging = Some(Either::Left(element.render(state, context)));
                    }
                    _ => unreachable!(),
                }
                store.status = RenderStatus::Swapped;
                true
            }
        }
    }
}

impl<L, R, S> WidgetNodeSeq<S> for EitherStore<L, R>
where
    L: WidgetNodeSeq<S>,
    R: WidgetNodeSeq<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        L::event_mask().merge(R::event_mask())
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.status == RenderStatus::Swapped {
            match self.active.as_mut() {
                Either::Left(node) => node.commit(CommitMode::Unmount, state, context),
                Either::Right(node) => node.commit(CommitMode::Unmount, state, context),
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if !mode.is_unmount() {
                match self.active.as_mut() {
                    Either::Left(node) => node.commit(CommitMode::Mount, state, context),
                    Either::Right(node) => node.commit(CommitMode::Mount, state, context),
                }
            }
            self.status = RenderStatus::Unchanged;
        } else if self.status == RenderStatus::Changed || mode.is_propagatable() {
            match self.active.as_mut() {
                Either::Left(node) => node.commit(mode, state, context),
                Either::Right(node) => node.commit(mode, state, context),
            }
            self.status = RenderStatus::Unchanged;
        }
    }

    fn event<E: 'static>(
        &self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        match self.active.as_ref() {
            Either::Left(node) => node.event(event, state, context),
            Either::Right(node) => node.event(event, state, context),
        }
    }

    fn internal_event(
        &self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        match self.active.as_ref() {
            Either::Left(node) => node.internal_event(event, state, context),
            Either::Right(node) => node.internal_event(event, state, context),
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
enum RenderStatus {
    Unchanged,
    Changed,
    Swapped,
}
