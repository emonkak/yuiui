use std::any::Any;
use std::mem;

use geometrics::Size;
use layout::{BoxConstraints, LayoutResult};
use tree::{NodeId, Tree};
use widget::null::Null;
use widget::{Element, RenderingTree, WidgetPod};

pub struct UIEngine<WindowHandle, PaintContext> {
    tree: RenderingTree<WindowHandle, PaintContext>,
    root_id: NodeId,
}

impl<WindowHandle, PaintContext> UIEngine<WindowHandle, PaintContext> {
    pub fn new(element: Element<WindowHandle, PaintContext>) -> UIEngine<WindowHandle, PaintContext> {
        let mut tree = Tree::new();
        let root_id = tree.attach(WidgetPod::new(element));
        UIEngine {
            tree,
            root_id,
        }
    }

    pub fn render(&mut self) {
        let mut current = self.root_id;
        while let Some(next) = self.render_step(current) {
            current = next;
        }
    }

    fn render_step(&mut self, target_id: NodeId) -> Option<NodeId> {
        let target = &mut self.tree[target_id];
        if let Some(rendered_children) = target.rendered_children.take() {
            reconcile_children(&mut self.tree, target_id, rendered_children);
        }
        next_child(&self.tree, self.root_id, target_id)
    }

    pub fn update(&mut self, element: Element<WindowHandle, PaintContext>) {
        self.tree[self.root_id].update(element);
    }

    pub fn layout(&mut self, box_constraints: BoxConstraints) -> Size {
        let mut requests = vec![(self.root_id, box_constraints)];
        let mut response = None;

        while let Some(&(child_id, box_constraints)) = requests.last() {
            let mut widget = mem::replace(&mut self.tree[child_id].widget, Box::new(Null));
            let result = widget.layout(
                child_id,
                response,
                &box_constraints,
                &mut self.tree
            );

            let node = &mut *self.tree[child_id];
            node.widget = widget;

            match result {
                LayoutResult::Size(size) => {
                    node.resize(size);
                    if requests.len() == 1 {
                        return size;
                    }
                    requests.pop();
                    response = Some((child_id, size));
                }
                LayoutResult::RequestChild(child_id, child_box_constraints) => {
                    let child = &self.tree[child_id];
                    if child.dirty {
                        requests.push((child_id, child_box_constraints));
                        response = None;
                    } else {
                        response = Some((child_id, child.rectangle.size));
                    }
                }
            }
        }

        unreachable!();
    }
}

impl<WindowHandle, PaintContext> ToString for UIEngine<WindowHandle, PaintContext> {
    fn to_string(&self) -> String {
        self.tree.to_string(self.root_id)
    }
}

fn reconcile_children<WindowHandle, PaintContext>(
    tree: &mut RenderingTree<WindowHandle, PaintContext>,
    target_id: NodeId,
    child_elements: Box<[Element<WindowHandle, PaintContext>]>
) {
    let mut child_elements = child_elements.into_vec().into_iter();
    let mut current_element = child_elements.next();
    let mut current_child = tree[target_id].first_child();

    loop {
        match (current_child.take(), current_element.take()) {
            (Some(child_id), Some(element)) if same_type(tree[child_id].widget.as_any(), element.widget.as_any()) => {
                // Update
                let child_node = &mut tree[child_id];
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
                for (node_id, mut detached_node) in tree.detach_subtree(child_id) {
                    println!("Delete: <{} id=\"{}\">", detached_node.widget.name(), node_id);
                    detached_node.disconnect();
                    if node_id == child_id {
                        current_child = detached_node.next_sibling();
                    }
                }
                current_element = element;
            }
            (_, Some(element)) => {
                // New
                let widget_pod = WidgetPod::new(element);
                let child_id = tree.append_child(target_id, widget_pod);
                println!("New: <{} id=\"{}\">", tree[child_id].widget.name(), child_id);
                current_element = child_elements.next();
            }
            (_, _) => break,
        }
    }
}

fn next_child<T>(tree: &Tree<T>, root_id: NodeId, target_id: NodeId) -> Option<NodeId> {
    if let Some(first_child) = tree[target_id].first_child() {
        return Some(first_child);
    }

    let mut next_target_id = target_id;

    loop {
        let next_target = &tree[next_target_id];
        if let Some(next_sibling) = next_target.next_sibling() {
            return Some(next_sibling);
        }

        next_target_id = next_target.parent();
        if next_target_id == root_id {
            break;
        }
    }

    None
}

fn same_type(first: &dyn Any, second: &dyn Any) -> bool {
    first.type_id() == second.type_id()
}
