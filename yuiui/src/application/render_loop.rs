use std::collections::VecDeque;
use std::mem;
use yuiui_support::slot_tree::NodeId;

use crate::geometrics::Rectangle;
use crate::graphics::Primitive;
use crate::widget::{Command, WidgetStorage};

#[derive(Debug)]
pub struct RenderLoop<Message> {
    storage: WidgetStorage<Message>,
    work_in_progress: Option<Work>,
    progress_roots: Vec<NodeId>,
    pending_works: VecDeque<Work>,
}

impl<Message: 'static> RenderLoop<Message> {
    pub fn new(storage: WidgetStorage<Message>) -> Self {
        let root_id = NodeId::ROOT;
        let initial_work = Work {
            id: root_id,
            component_index: 0,
            root: root_id,
        };
        Self {
            progress_roots: vec![root_id],
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
                root: id,
            };
            self.progress_roots.push(work.root);
            self.work_in_progress = Some(work);
        } else {
            if self
                .pending_works
                .iter()
                .position(|work| work.root == id)
                .is_none()
            {
                let work = Work {
                    id,
                    component_index,
                    root: id,
                };
                self.pending_works.push_back(work);
            }
        }
    }

    pub fn render(&mut self) -> RenderFlow<impl Iterator<Item = Command<Message>> + '_> {
        if let Some(work) = self.work_in_progress.take() {
            self.process_work(work);
            RenderFlow::Continue
        } else if let Some(work) = self.pending_works.pop_front() {
            self.progress_roots.push(work.root);
            self.process_work(work);
            RenderFlow::Continue
        } else if self.storage.has_uncommited_changes() {
            let commands = self.storage.commit();
            RenderFlow::Commit(commands)
        } else if !self.progress_roots.is_empty() {
            let mut scissor_bounds = None;
            for root in mem::take(&mut self.progress_roots) {
                let layout_root = self.storage.layout(root);
                if !layout_root.is_root() {
                    let (_, draw_bounds) = self.storage.draw(layout_root);
                    scissor_bounds = match scissor_bounds {
                        None => Some(draw_bounds),
                        Some(bounds) => Some(bounds.union(draw_bounds))
                    };
                }
            }
            let (primitive, _) = self.storage.draw(NodeId::ROOT);
            RenderFlow::Paint(primitive, scissor_bounds)
        } else {
            RenderFlow::Idle
        }
    }

    fn process_work(&mut self, work: Work) {
        let next = self
            .storage
            .render(work.id, work.component_index, work.root);
        if let Some((id, component_index)) = next {
            let work = Work {
                id,
                component_index,
                root: work.root,
            };
            self.work_in_progress = Some(work);
        }
    }
}

#[derive(Debug)]
pub enum RenderFlow<Commands> {
    Continue,
    Commit(Commands),
    Paint(Primitive, Option<Rectangle>),
    Idle,
}

#[derive(Debug)]
struct Work {
    id: NodeId,
    component_index: usize,
    root: NodeId,
}
