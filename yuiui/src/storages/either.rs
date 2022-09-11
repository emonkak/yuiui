use either::Either;
use std::mem;
use std::sync::Once;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::Store;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

use super::RenderFlags;

#[derive(Debug)]
pub struct EitherStorage<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    flags: RenderFlags,
}

impl<L, R> EitherStorage<L, R> {
    fn new(active: Either<L, R>) -> Self {
        Self {
            active,
            staging: None,
            flags: RenderFlags::NONE,
        }
    }
}

impl<L, R, S, M, B> ElementSeq<S, M, B> for Either<L, R>
where
    L: ElementSeq<S, M, B>,
    R: ElementSeq<S, M, B>,
{
    type Storage = EitherStorage<L::Storage, R::Storage>;

    const DEPTH: usize = [L::DEPTH, R::DEPTH][(L::DEPTH < R::DEPTH) as usize];

    fn render_children(self, context: &mut RenderContext, store: &mut Store<S>) -> Self::Storage {
        match self {
            Either::Left(element) => {
                EitherStorage::new(Either::Left(element.render_children(context, store)))
            }
            Either::Right(element) => {
                EitherStorage::new(Either::Right(element.render_children(context, store)))
            }
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool {
        match (&mut storage.active, self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update_children(node, context, store) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update_children(node, context, store) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match &mut storage.staging {
                    Some(Either::Right(node)) => {
                        element.update_children(node, context, store);
                    }
                    None => {
                        storage.staging =
                            Some(Either::Right(element.render_children(context, store)));
                    }
                    _ => unreachable!(),
                };
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match &mut storage.staging {
                    Some(Either::Left(node)) => {
                        element.update_children(node, context, store);
                    }
                    None => {
                        storage.staging =
                            Some(Either::Left(element.render_children(context, store)));
                    }
                    _ => unreachable!(),
                }
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
        }
    }
}

impl<L, R, S, M, B> ViewNodeSeq<S, M, B> for EitherStorage<L, R>
where
    L: ViewNodeSeq<S, M, B>,
    R: ViewNodeSeq<S, M, B>,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let left_mask = L::event_mask();
            let right_mask = R::event_mask();

            INIT.call_once(|| unsafe {
                EVENT_MASK.append(left_mask);
                EVENT_MASK.append(right_mask);
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        match &self.active {
            Either::Left(node) => node.len(),
            Either::Right(node) => node.len(),
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                result |= match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Unmount, context, store, backend),
                    Either::Right(node) => {
                        node.commit(CommitMode::Unmount, context, store, backend)
                    }
                };
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                result |= match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Mount, context, store, backend),
                    Either::Right(node) => node.commit(CommitMode::Mount, context, store, backend),
                };
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            result |= match &mut self.active {
                Either::Left(node) => node.commit(mode, context, store, backend),
                Either::Right(node) => node.commit(mode, context, store, backend),
            };
        }
        self.flags = RenderFlags::COMMITED;
        result
    }
}

impl<L, R, Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B>
    for EitherStorage<L, R>
where
    L: Traversable<Visitor, Context, Output, S, B>,
    R: Traversable<Visitor, Context, Output, S, B>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Output {
        match &mut self.active {
            Either::Left(node) => node.for_each(visitor, context, store, backend),
            Either::Right(node) => node.for_each(visitor, context, store, backend),
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Option<Output> {
        match &mut self.active {
            Either::Left(node) => node.search(id_path, visitor, context, store, backend),
            Either::Right(node) => node.search(id_path, visitor, context, store, backend),
        }
    }
}
