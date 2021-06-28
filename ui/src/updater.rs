use std::any::TypeId;
use std::fmt;
use std::mem;

use geometrics::Size;
use layout::{BoxConstraints, LayoutContext, LayoutResult};
use reconciler::{Commiter, reconcile};
use tree::{NodeId, Tree};
use widget::null::Null;
use widget::widget::{Element, Fiber, FiberTree, Key, Widget};

#[derive(Debug)]
pub struct UIUpdater<Window> {
    layout_context: LayoutContext,
    fiber_tree: FiberTree<Window>,
    root_id: NodeId,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl<Window> UIUpdater<Window> {
    pub fn new(element: Element<Window>) -> UIUpdater<Window> {
        let mut fiber_tree = Tree::new();
        let layout_context = LayoutContext::new();
        let root_id = fiber_tree.attach(Fiber::new(element));

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

    pub fn force_update(&mut self, element: Element<Window>) {
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
                        response = Some((child_id, *self.layout_context.get_size(child_id).unwrap()));
                    }
                }
            }
        }

        unreachable!();
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

    fn reconcile_children(
        &mut self,
        target_id: NodeId,
        children: Box<[Element<Window>]>,
    ) {
        let mut old_keys: Vec<TypedKey> = Vec::new();
        let mut old_node_ids: Vec<Option<NodeId>> = Vec::new();

        for (index, (child_id, child)) in self.fiber_tree.children(target_id).enumerate() {
            let key = key_of(&*child.widget, index);
            old_keys.push(key);
            old_node_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Window>>> = Vec::with_capacity(children.len());

        for (index, element) in children.into_vec().into_iter().enumerate() {
            let key = key_of(&*element.widget, index);
            new_keys.push(key);
            new_elements.push(Some(element));
        }

        reconcile(self, target_id, &old_keys, &mut old_node_ids, &new_keys, &mut new_elements);
    }
}

impl<Window> Commiter<NodeId, Element<Window>> for UIUpdater<Window> {
    fn commit_place(&mut self, ref_id: NodeId, new_element: Element<Window>) {
        println!("Place: [ref_id: {}] {}", ref_id, new_element.widget.name());
        let new_fiber = Fiber::new(new_element);
        self.fiber_tree.append_child(ref_id, new_fiber);
    }

    fn commit_place_at(&mut self, ref_id: NodeId, new_element: Element<Window>) {
        println!("PlaceAt: [ref_id: {}] {}", ref_id, new_element.widget.name());
        let new_fiber = Fiber::new(new_element);
        self.fiber_tree.insert_before(ref_id, new_fiber);
    }

    fn commit_update(&mut self, target_id: NodeId, new_element: Element<Window>) {
        let target_node = &mut self.fiber_tree[target_id];
        if target_node.update(new_element) {
            println!("Update: [target_id: {}] {}", target_id, target_node.widget.name());
        } else {
            println!("NoChanges: [target_id: {}] {}", target_id, target_node.widget.name());
        }
    }

    fn commit_update_and_move(&mut self, target_id: NodeId, ref_id: NodeId, new_element: Element<Window>) {
        println!("UpdateAndMove: [target_id: {}] [ref_id: {}] {}", target_id, ref_id, new_element.widget.name());
        let mut target_node = self.fiber_tree.detach(target_id);
        if target_node.update(new_element) {
            println!("Update: [target_id: {}] {}", target_id, target_node.widget.name());
        } else {
            println!("NoChanges: [target_id: {}] {}", target_id, target_node.widget.name());
        }
        self.fiber_tree.insert_before(ref_id, target_node);
    }

    fn commit_delete(&mut self, target_id: NodeId) {
        println!("Delete: [target_id: {}]", target_id);
        for (node_id, mut detached_node) in self.fiber_tree.detach_subtree(target_id) {
            detached_node.unmount();
            self.layout_context.remove(node_id);
        }
    }
}

impl<Window: fmt::Debug> fmt::Display for UIUpdater<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fiber_tree.format(
            f,
            self.root_id,
            &|f, node_id, fiber| {
                write!(f, "<{}", fiber.widget.name())?;
                write!(f, " id=\"{}\"", node_id)?;
                if let Some(rectangle) = self.layout_context.get_rectangle(node_id) {
                    write!(f, " x=\"{}\"", rectangle.point.x)?;
                    write!(f, " y=\"{}\"", rectangle.point.y)?;
                    write!(f, " width=\"{}\"", rectangle.size.width)?;
                    write!(f, " height=\"{}\"", rectangle.size.height)?;
                }
                if fiber.dirty {
                    write!(f, " dirty")?;
                }
                write!(f, ">")?;
                Ok(())
            },
            &|f, _, fiber| write!(f, "</{}>", fiber.widget.name())
        )
    }
}

fn key_of<Window>(widget: &dyn Widget<Window>, index: usize) -> TypedKey {
    match widget.key() {
        Some(key) => TypedKey::Keyed(widget.as_any().type_id(), key),
        None => TypedKey::Indexed(widget.as_any().type_id(), index),
    }
}