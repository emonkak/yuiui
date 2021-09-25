use std::collections::VecDeque;
use yuiui_support::slot_tree::NodeId;

use crate::geometrics::Rectangle;
use crate::graphics::Primitive;
use crate::widget::WidgetStorage;

#[derive(Debug)]
pub struct RenderLoop {
    storage: WidgetStorage,
    current_root: Option<NodeId>,
    work_in_progress: Option<Work>,
    pending_works: VecDeque<Work>,
}

impl RenderLoop {
    pub fn new(storage: WidgetStorage) -> Self {
        let root_id = NodeId::ROOT;
        let initial_work = Work {
            id: root_id,
            component_index: 0,
            origin: root_id,
        };
        Self {
            current_root: Some(root_id),
            work_in_progress: Some(initial_work),
            pending_works: VecDeque::new(),
            storage,
        }
    }

    pub fn schedule_update(&mut self, id: NodeId, component_index: usize) {
        if self.work_in_progress.is_none() {
            let work = Work {
                id,
                component_index,
                origin: id,
            };
            self.current_root = Some(work.origin);
            self.work_in_progress = Some(work);
        } else {
            if self
                .pending_works
                .iter()
                .position(|work| work.origin == id)
                .is_none()
            {
                let work = Work {
                    id,
                    component_index,
                    origin: id,
                };
                self.pending_works.push_back(work);
            }
        }
    }

    pub fn render(&mut self) -> RenderResult {
        if let Some(work) = self.work_in_progress.take() {
            self.process_work(work);
            RenderResult::Continue
        } else if let Some(render_root) = self.current_root.take() {
            let layout_root = self.storage.layout(render_root);
            let (primitive, bounds) = self.storage.draw(layout_root);
            if layout_root.is_root() {
                RenderResult::Commit(primitive, None)
            } else {
                let (primitive, _) = self.storage.draw(NodeId::ROOT);
                RenderResult::Commit(primitive, Some(bounds))
            }
        } else if let Some(work) = self.pending_works.pop_front() {
            self.process_work(work);
            RenderResult::Continue
        } else {
            RenderResult::Idle
        }
    }

    fn process_work(&mut self, work: Work) {
        let next = self
            .storage
            .render(work.id, work.component_index, work.origin);
        if let Some((id, component_index)) = next {
            let work = Work {
                id,
                component_index,
                origin: work.origin,
            };
            self.work_in_progress = Some(work);
        }
    }
}

#[derive(Debug)]
pub enum RenderResult {
    Continue,
    Commit(Primitive, Option<Rectangle>),
    Idle,
}

#[derive(Debug)]
struct Work {
    id: NodeId,
    component_index: usize,
    origin: NodeId,
}
