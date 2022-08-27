use hlist::{HCons, HList, HNil};

use crate::effect::{EffectContext, EffectContextSeq, EffectContextVisitor};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::render::{IdPath, RenderContext, RenderContextSeq, RenderContextVisitor};
use crate::state::State;
use crate::widget_node::{CommitMode, WidgetNodeSeq};

impl<S, E> ElementSeq<S, E> for HNil
where
    S: State,
{
    type Store = HNil;

    fn render(self, _state: &S, _env: &E, _context: &mut RenderContext) -> Self::Store {
        HNil
    }

    fn update(
        self,
        _nodes: &mut Self::Store,
        _state: &S,
        _env: &E,
        _context: &mut RenderContext,
    ) -> bool {
        false
    }
}

impl<S, E> WidgetNodeSeq<S, E> for HNil
where
    S: State,
{
    fn event_mask() -> EventMask {
        EventMask::new()
    }

    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EffectContext<S>) {
    }
}

impl<S, E> RenderContextSeq<S, E> for HNil
where
    S: State,
{
    fn for_each<V: RenderContextVisitor>(
        &mut self,
        _visitor: &mut V,
        _state: &S,
        _env: &E,
        _context: &mut RenderContext,
    ) {
    }

    fn search<V: RenderContextVisitor>(
        &mut self,
        _id_path: &IdPath,
        _visitor: &mut V,
        _state: &S,
        _env: &E,
        _context: &mut RenderContext,
    ) -> bool {
        false
    }
}

impl<S, E> EffectContextSeq<S, E> for HNil
where
    S: State,
{
    fn for_each<V: EffectContextVisitor>(
        &mut self,
        _visitor: &mut V,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) {
    }

    fn search<V: EffectContextVisitor>(
        &mut self,
        _id_path: &IdPath,
        _visitor: &mut V,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) -> bool {
        false
    }
}

impl<H, T, S, E> ElementSeq<S, E> for HCons<H, T>
where
    H: ElementSeq<S, E>,
    T: ElementSeq<S, E> + HList,
    T::Store: HList,
    S: State,
{
    type Store = HCons<H::Store, T::Store>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        HCons {
            head: self.head.render(state, env, context),
            tail: self.tail.render(state, env, context),
        }
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.update(&mut store.head, state, env, context);
        has_changed |= self.tail.update(&mut store.tail, state, env, context);
        has_changed
    }
}

impl<H, T, S, E> WidgetNodeSeq<S, E> for HCons<H, T>
where
    H: WidgetNodeSeq<S, E>,
    T: WidgetNodeSeq<S, E> + HList,
    S: State,
{
    fn event_mask() -> EventMask {
        H::event_mask().merge(T::event_mask())
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.head.commit(mode, state, env, context);
        self.tail.commit(mode, state, env, context);
    }
}

impl<H, T, S, E> RenderContextSeq<S, E> for HCons<H, T>
where
    H: RenderContextSeq<S, E>,
    T: RenderContextSeq<S, E> + HList,
    S: State,
{
    fn for_each<V: RenderContextVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) {
        self.head.for_each(visitor, state, env, context);
        self.tail.for_each(visitor, state, env, context);
    }

    fn search<V: RenderContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        self.head.search(id_path, visitor, state, env, context)
            || self.tail.search(id_path, visitor, state, env, context)
    }
}

impl<H, T, S, E> EffectContextSeq<S, E> for HCons<H, T>
where
    H: EffectContextSeq<S, E>,
    T: EffectContextSeq<S, E> + HList,
    S: State,
{
    fn for_each<V: EffectContextVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        self.head.for_each(visitor, state, env, context);
        self.tail.for_each(visitor, state, env, context);
    }

    fn search<V: EffectContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        self.head.search(id_path, visitor, state, env, context)
            || self.tail.search(id_path, visitor, state, env, context)
    }
}
