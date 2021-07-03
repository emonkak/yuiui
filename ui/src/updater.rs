use std::any::TypeId;
use std::collections::VecDeque;
use std::fmt;
use std::mem;

use geometrics::{Point, Rectangle, Size};
use layout::{BoxConstraints, LayoutContext, LayoutResult};
use paint::PaintContext;
use reconciler::{Reconciler, ReconcileResult};
use tree::{NodeId, Tree, WalkDirection};
use widget::null::Null;
use widget::widget::{Element, Fiber, FiberTree, Key, WidgetDyn};

#[derive(Debug)]
pub struct Updater<Window> {
    tree: FiberTree<Window>,
    root_id: NodeId,
    layout_context: LayoutContext,
    update_queue: VecDeque<NodeId>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl<Window> Updater<Window> {
    pub fn new() -> Self {
        let mut tree = Tree::new();
        let root_id = tree.attach(Fiber::new(Box::new(Null), Box::new([])));
        let layout_context = LayoutContext::new();

        Self {
            tree,
            root_id,
            layout_context,
            update_queue: VecDeque::new(),
        }
    }

    pub fn update(&mut self, element: Element<Window>) {
        self.tree[self.root_id].update(Element::new(Null, [element]));
    }

    pub fn render(&mut self) {
        let mut current = self.root_id;
        while let Some(next) = self.render_step(current) {
            current = next;
        }
    }

    pub fn layout(&mut self, box_constraints: BoxConstraints) -> Size {
        let mut requests = vec![(self.root_id, box_constraints)];
        let mut response = None;

        while let Some(&(request_id, box_constraints)) = requests.last() {
            let node = &mut self.tree[request_id];
            let widget = mem::replace(&mut node.widget, Box::new(Null));
            let mut state = node.state.take().unwrap_or_else(|| widget.initial_state());

            let result = widget.layout(
                request_id,
                box_constraints,
                response,
                &self.tree,
                &mut self.layout_context,
                &mut *state
            );

            let node = &mut self.tree[request_id];
            node.widget = widget;
            node.state = Some(state);

            match result {
                LayoutResult::Size(size) => {
                    self.layout_context.resize(request_id, size);
                    if size == Size::ZERO {
                        node.dirty = false;
                    }
                    if requests.len() == 1 {
                        return size;
                    }
                    requests.pop();
                    response = Some((request_id, size));
                }
                LayoutResult::RequestChild(child_id, child_box_constraints) => {
                    let child = &self.tree[child_id];
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

    pub fn paint(&mut self, parent_handle: &Window, paint_context: &mut PaintContext<Window>) {
        let mut handle = parent_handle;
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };

        for (node_id, node, direction) in self.tree.walk_filter_mut(self.root_id, |_, node| node.dirty) {
            let rectangle = self.layout_context.get_rectangle(node_id).unwrap();
            match direction {
                WalkDirection::Downward => {
                    absolute_point += latest_point;
                    let paint_rectangle = Rectangle {
                        point: absolute_point + rectangle.point,
                        size: rectangle.size
                    };
                    handle = node.paint(&paint_rectangle, handle, paint_context);
                }
                WalkDirection::Sideward => {
                    let paint_rectangle = Rectangle {
                        point: absolute_point + rectangle.point,
                        size: rectangle.size,
                    };
                    handle = node.paint(&paint_rectangle, handle, paint_context);
                }
                WalkDirection::Upward => {
                    absolute_point -= rectangle.point;
                }
            }
            latest_point = rectangle.point;
        }
    }

    fn render_step(&mut self, node_id: NodeId) -> Option<NodeId> {
        let target = &mut self.tree[node_id];
        if let Some(rendered_children) = target.rendered_children.take() {
            self.reconcile_children(node_id, rendered_children);
        }
        self.next_render_step(node_id)
    }

    fn next_render_step(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(first_child) = self.tree[node_id].first_child() {
            return Some(first_child);
        }

        let mut currnet_node_id = node_id;

        loop {
            let current_node = &self.tree[currnet_node_id];
            if let Some(sibling_id) = current_node.next_sibling() {
                return Some(sibling_id);
            }

            if let Some(parent_id) = current_node
                .parent()
                .filter(|&parent_id| parent_id != self.root_id) {
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

        for (index, (child_id, child)) in self.tree.children(target_id).enumerate() {
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

        let reconciler = Reconciler::new(
            &old_keys,
            &mut old_node_ids,
            &new_keys,
            &mut new_elements
        );

        for result in reconciler {
            self.handle_reconcile_result(target_id, result);
        }
    }

    fn handle_reconcile_result(
        &mut self,
        target_id: NodeId,
        result: ReconcileResult<NodeId, Element<Window>>
    ) {
        println!("{:?}", result);
        match result {
            ReconcileResult::New(new_element) => {
                let new_fiber = Fiber::from(new_element);
                self.tree.append_child(target_id, new_fiber);
            }
            ReconcileResult::NewPlacement(ref_id, new_element) => {
                let new_fiber = Fiber::from(new_element);
                self.tree.insert_before(ref_id, new_fiber);
            }
            ReconcileResult::Update(target_id, new_element) => {
                let target_node = &mut self.tree[target_id];
                target_node.update(new_element);
            }
            ReconcileResult::UpdatePlacement(target_id, ref_id, new_element) => {
                let target_node = &mut self.tree[target_id];
                target_node.update(new_element);
                self.tree.move_position(target_id).insert_before(ref_id);
            }
            ReconcileResult::Deletion(target_id) => {
                for (node_id, mut detached_node) in self.tree.detach_subtree(target_id) {
                    detached_node.unmount();
                    self.layout_context.remove(node_id);
                }
            }
        }
    }
}

impl<Window: fmt::Debug> fmt::Display for Updater<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.tree.format(
                self.root_id,
                |f, node_id, fiber| {
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
                |f, _, fiber| write!(f, "</{}>", fiber.widget.name())
            )
        )
    }
}

fn key_of<Window>(widget: &dyn WidgetDyn<Window>, index: usize) -> TypedKey {
    match widget.key() {
        Some(key) => TypedKey::Keyed(widget.as_any().type_id(), key),
        None => TypedKey::Indexed(widget.as_any().type_id(), index),
    }
}
