use std::any::Any;
use std::fmt;
use std::mem;

use geometrics::Size;
use layout::{BoxConstraints, LayoutContext, LayoutResult};
use tree::{NodeId, Tree};
use widget::null::Null;
use widget::widget::{FiberTree, Fiber, Element};

#[derive(Debug)]
pub struct UIUpdater<Window> {
    layout_context: LayoutContext,
    fiber_tree: FiberTree<Window>,
    root_id: NodeId,
}

impl<Window> UIUpdater<Window> {
    pub fn new(element: Element<Window>) -> UIUpdater<Window> {
        let mut fiber_tree = Tree::new();
        let mut layout_context = LayoutContext::new();

        let root_id = fiber_tree.attach(Fiber::new(element));
        layout_context.insert_at(root_id, Default::default());

        UIUpdater {
            layout_context,
            fiber_tree,
            root_id,
        }
    }

    pub fn render(&mut self) {
        let mut current = self.root_id;
        while let Some(next) = self.render_step(current) {
            current = next;
        }
    }

    fn render_step(&mut self, node_id: NodeId) -> Option<NodeId> {
        let target = &mut self.fiber_tree[node_id];
        if let Some(rendered_children) = target.rendered_children.take() {
            self.reconcile_children(node_id, rendered_children);
        }
        self.next_render_node(self.root_id, node_id)
    }

    pub fn update(&mut self, element: Element<Window>) {
        self.fiber_tree[self.root_id].update(element);
    }

    pub fn layout(&mut self, box_constraints: BoxConstraints) -> Size {
        let mut requests = vec![(self.root_id, box_constraints)];
        let mut response = None;

        while let Some(&(request_id, box_constraints)) = requests.last() {
            let mut widget = mem::replace(&mut self.fiber_tree[request_id].widget, Box::new(Null));
            let result = widget.layout(
                request_id,
                box_constraints,
                response,
                &self.fiber_tree,
                &mut self.layout_context
            );
            self.fiber_tree[request_id].widget = widget;

            match result {
                LayoutResult::Size(size) => {
                    self.layout_context.resize(request_id, size);
                    if requests.len() == 1 {
                        return size;
                    }
                    requests.pop();
                    response = Some((request_id, size));
                }
                LayoutResult::RequestChild(child_id, child_box_constraints) => {
                    let child = &self.fiber_tree[child_id];
                    if child.dirty {
                        requests.push((child_id, child_box_constraints));
                        response = None;
                    } else {
                        response = Some((child_id, *self.layout_context.get_size(child_id)));
                    }
                }
            }
        }

        unreachable!();
    }

    fn reconcile_children(
        &mut self,
        node_id: NodeId,
        child_elements: Box<[Element<Window>]>
    ) {
        let mut child_elements = child_elements.into_vec().into_iter();
        let mut current_element = child_elements.next();
        let mut current_child = self.fiber_tree[node_id].first_child();

        loop {
            match (current_child.take(), current_element.take()) {
                (Some(child_id), Some(element)) if same_type(self.fiber_tree[child_id].widget.as_any(), element.widget.as_any()) => {
                    // Update
                    let child_node = &mut self.fiber_tree[child_id];
                    if child_node.should_update(&element) {
                        println!("Update: <{} id=\"{}\">", element.widget.name(), child_id);
                        child_node.update(element);
                    } else {
                        println!("Skip: <{} id=\"{}\">", element.widget.name(), child_id);
                    }
                    current_element = child_elements.next();
                    current_child = child_node.next_sibling();
                }
                (Some(child_id), element) => {
                    // Delete
                    for (node_id, mut detached_node) in self.fiber_tree.detach_subtree(child_id) {
                        println!("Delete: <{} id=\"{}\">", detached_node.widget.name(), node_id);
                        detached_node.unmount();
                        self.layout_context.remove(node_id);
                        if node_id == child_id {
                            current_child = detached_node.next_sibling();
                        }
                    }
                    current_element = element;
                }
                (_, Some(element)) => {
                    // New
                    let fiber = Fiber::new(element);
                    let new_child_id = self.fiber_tree.append_child(node_id, fiber);
                    self.layout_context.insert_at(new_child_id, Default::default());
                    current_element = child_elements.next();
                    println!("New: <{} id=\"{}\">", self.fiber_tree[new_child_id].widget.name(), new_child_id);
                }
                (_, _) => break,
            }
        }
    }

    fn next_render_node(&self, root_id: NodeId, node_id: NodeId) -> Option<NodeId> {
        if let Some(first_child) = self.fiber_tree[node_id].first_child() {
            return Some(first_child);
        }

        let mut currnet_node_id = node_id;

        loop {
            let current_node = &self.fiber_tree[currnet_node_id];
            if let Some(sibling_id) = current_node.next_sibling() {
                return Some(sibling_id);
            }

            if let Some(parent_id) = current_node
                .parent()
                .filter(|&parent_id| parent_id != root_id) {
                currnet_node_id = parent_id;
            } else {
                break;
            }
        }

        None
    }
}

impl<Window: fmt::Debug> fmt::Display for UIUpdater<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fiber_tree.fmt(f, self.root_id)
    }
}

fn same_type(first: &dyn Any, second: &dyn Any) -> bool {
    first.type_id() == second.type_id()
}
