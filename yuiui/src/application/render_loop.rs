use std::collections::VecDeque;
use std::mem;
use yuiui_support::slot_tree::NodeId;

use crate::geometrics::{Rectangle, Viewport};
use crate::graphics::Primitive;
use crate::widget::{
    Command, ComponentIndex, Element, ElementTree, Event, UnitOfWork, Widget, WidgetTree,
};
use crate::widget_impl::null::Null;

#[derive(Debug)]
pub struct RenderLoop<State, Message> {
    element_tree: ElementTree<State, Message>,
    widget_tree: WidgetTree<State, Message>,
    work_in_progress: Option<RenderNode>,
    progress_roots: Vec<NodeId>,
    pending_nodes: VecDeque<RenderNode>,
    pending_works: Vec<UnitOfWork<State, Message>>,
}

impl<State: 'static, Message: 'static> RenderLoop<State, Message> {
    pub fn new(element: Element<State, Message>) -> Self {
        let root_widget = Null.into_rc();
        let element_tree = ElementTree::new(root_widget.clone(), element);
        let widget_tree = WidgetTree::new(root_widget);

        let initial_node = RenderNode {
            id: NodeId::ROOT,
            component_index: 0,
            root: NodeId::ROOT,
        };

        Self {
            element_tree,
            widget_tree,
            work_in_progress: Some(initial_node),
            progress_roots: vec![NodeId::ROOT],
            pending_nodes: VecDeque::new(),
            pending_works: Vec::new(),
        }
    }

    pub fn schedule_update(&mut self, id: NodeId, component_index: usize) -> bool {
        if self.work_in_progress.is_none() {
            self.progress_roots.push(id);
            self.work_in_progress = Some(RenderNode {
                id,
                component_index,
                root: id,
            });
            true
        } else {
            if self
                .pending_nodes
                .iter()
                .position(|node| node.root == id)
                .is_none()
            {
                self.pending_nodes.push_back(RenderNode {
                    id,
                    component_index,
                    root: id,
                });
                true
            } else {
                false
            }
        }
    }

    pub fn render<Handler>(&mut self, viewport: &Viewport, handler: &Handler) -> RenderFlow
    where
        Handler: Fn(Command<Message>, NodeId, ComponentIndex),
    {
        if let Some(node) = self.work_in_progress.take() {
            self.process_node(node, handler);
            RenderFlow::Continue
        } else if let Some(node) = self.pending_nodes.pop_front() {
            self.progress_roots.push(node.root);
            self.process_node(node, handler);
            RenderFlow::Continue
        } else if !self.progress_roots.is_empty() {
            if !self.pending_works.is_empty() {
                for unit_of_work in mem::take(&mut self.pending_works) {
                    self.widget_tree.commit(unit_of_work, &|command, id| {
                        handler(command, id, ComponentIndex::MAX)
                    });
                }
            }

            let mut effective_bounds = None;
            for root in mem::take(&mut self.progress_roots) {
                let layout_root = self.widget_tree.layout(root, viewport);
                if !layout_root.is_root() {
                    let (_, draw_bounds) = self.widget_tree.draw(layout_root);

                    effective_bounds = match effective_bounds {
                        None => Some(draw_bounds),
                        Some(effective_bounds) => Some(effective_bounds.union(draw_bounds)),
                    };
                }
            }

            let (primitive, _) = self.widget_tree.draw(NodeId::ROOT);
            RenderFlow::Paint(primitive, effective_bounds)
        } else {
            RenderFlow::Idle
        }
    }

    pub fn dispatch<Handler>(&mut self, event: Event<State>, handler: &Handler)
    where
        Handler: Fn(Command<Message>, NodeId, ComponentIndex),
    {
        self.element_tree.dispatch(event, handler);
        self.widget_tree.dispatch(event, |command, id| {
            handler(command, id, ComponentIndex::MAX)
        })
    }

    fn process_node<Handler>(&mut self, node: RenderNode, handler: &Handler)
    where
        Handler: Fn(Command<Message>, NodeId, ComponentIndex),
    {
        let next = self.element_tree.render(
            node.id,
            node.component_index,
            node.root,
            handler,
            &mut self.pending_works,
        );
        if let Some((id, component_index)) = next {
            self.work_in_progress = Some(RenderNode {
                id,
                component_index,
                root: node.root,
            });
        }
    }
}

#[derive(Debug)]
pub enum RenderFlow {
    Continue,
    Paint(Primitive, Option<Rectangle>),
    Idle,
}

#[derive(Debug)]
struct RenderNode {
    id: NodeId,
    component_index: usize,
    root: NodeId,
}
