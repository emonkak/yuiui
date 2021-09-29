use futures::FutureExt;
use std::collections::VecDeque;
use std::mem;
use yuiui_support::slot_tree::NodeId;

use super::message::ApplicationMessage;
use crate::geometrics::{Rectangle, Viewport};
use crate::graphics::Primitive;
use crate::ui::EventLoopContext;
use crate::widget::{Command, Element, ElementTree, Event, UnitOfWork, Widget, WidgetTree};
use crate::widget_impl::root::Root;

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
        let root_widget = Root.into_rc();
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

    pub fn schedule_update(&mut self, id: NodeId, component_index: usize) {
        if self.work_in_progress.is_none() {
            let node = RenderNode {
                id,
                component_index,
                root: id,
            };
            self.progress_roots.push(node.root);
            self.work_in_progress = Some(node);
        } else {
            if self
                .pending_nodes
                .iter()
                .position(|node| node.root == id)
                .is_none()
            {
                let node = RenderNode {
                    id,
                    component_index,
                    root: id,
                };
                self.pending_nodes.push_back(node);
            }
        }
    }

    pub fn render<Context>(&mut self, viewport: &Viewport, context: &Context) -> RenderFlow
    where
        Context: EventLoopContext<ApplicationMessage<Message>>,
    {
        if let Some(node) = self.work_in_progress.take() {
            self.process_node(node);
            RenderFlow::Continue
        } else if let Some(node) = self.pending_nodes.pop_front() {
            self.progress_roots.push(node.root);
            self.process_node(node);
            RenderFlow::Continue
        } else if !self.progress_roots.is_empty() {
            if !self.pending_works.is_empty() {
                for unit_of_work in mem::take(&mut self.pending_works) {
                    self.widget_tree
                        .commit(unit_of_work, |command| run_command(context, command));
                }
            }

            let mut scissor_bounds = None;
            for root in mem::take(&mut self.progress_roots) {
                let layout_root = self.widget_tree.layout(root, viewport);
                if !layout_root.is_root() {
                    let (_, draw_bounds) = self.widget_tree.draw(layout_root);

                    scissor_bounds = match scissor_bounds {
                        None => Some(draw_bounds),
                        Some(bounds) => Some(bounds.union(draw_bounds)),
                    };
                }
            }

            let (primitive, _) = self.widget_tree.draw(NodeId::ROOT);
            RenderFlow::Paint(primitive, scissor_bounds)
        } else {
            RenderFlow::Idle
        }
    }

    pub fn dispatch<Context>(&mut self, event: &Event<State>, context: &Context)
    where
        Context: EventLoopContext<ApplicationMessage<Message>>,
    {
        self.widget_tree
            .dispatch(event, |command| run_command(context, command))
    }

    fn process_node(&mut self, node: RenderNode) {
        let next = self.element_tree.render(
            node.id,
            node.component_index,
            node.root,
            &mut self.pending_works,
        );
        if let Some((id, component_index)) = next {
            let node = RenderNode {
                id,
                component_index,
                root: node.root,
            };
            self.work_in_progress = Some(node);
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

fn run_command<Message, Context>(context: &Context, command: Command<Message>)
where
    Message: 'static,
    Context: EventLoopContext<ApplicationMessage<Message>>,
{
    match command {
        Command::Perform(future) => {
            context.perform(future.map(ApplicationMessage::Broadcast));
        }
        Command::RequestIdle(callback) => {
            context.request_idle(|deadline| ApplicationMessage::Broadcast(callback(deadline)));
        }
        Command::Send(message) => context.send(ApplicationMessage::Broadcast(message)),
        Command::Quit => context.send(ApplicationMessage::Quit),
    }
}
