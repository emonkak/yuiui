use std::any::{Any, TypeId};
use std::fmt;
use std::mem;
use std::ptr;

use geometrics::{Point, Rectangle, Size};
use layout::{DefaultLayout, BoxConstraints, LayoutResult, Layouter};
use paint::PaintContext;
use reconciler::{Reconciler, ReconcileResult};
use slot_vec::SlotVec;
use tree::walk::{WalkDirection, walk_next_node};
use tree::{NodeId, Tree};
use widget::null::Null;
use widget::{Element, Key, WidgetDyn, WidgetLayout, WidgetTree};

#[derive(Debug)]
pub struct Updater<Handle> {
    tree: WidgetTree<Handle>,
    root_id: NodeId,
    render_states: SlotVec<RenderState<Handle>>,
}

#[derive(Debug)]
pub struct RenderState<Handle> {
    pub(crate) rendered_children: Option<Box<[Element<Handle>]>>,
    pub(crate) deleted_children: Vec<NodeId>,
    pub(crate) state: Box<dyn Any>,
    pub(crate) dirty: bool,
    pub(crate) rectangle: Rectangle,
    pub(crate) handle: Option<Handle>,
    pub(crate) mounted: bool,
}

#[derive(Debug)]
pub struct LayoutContext<'a, Handle> {
    render_states: &'a mut SlotVec<RenderState<Handle>>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl<Handle> Updater<Handle> {
    pub fn new() -> Self {
        let mut tree = Tree::new();
        let mut render_states = SlotVec::new();

        let root_id = tree.attach(Box::new(Null) as Box<dyn WidgetDyn<Handle>>);

        render_states.insert_at(root_id, RenderState::new(&Null, Box::new([])));

        Self {
            tree,
            root_id,
            render_states,
        }
    }

    pub fn update(&mut self, element: Element<Handle>) {
        self.update_render_state(self.root_id, Box::new(Null), Box::new([element]));
    }

    pub fn render(&mut self) {
        let mut current = self.root_id;
        while let Some(next) = self.render_step(current) {
            current = next;
        }
    }

    pub fn layout(&mut self, viewport_size: Size, force_layout: bool) -> Size {
        let mut requests: Vec<(NodeId, BoxConstraints, WidgetLayout<Handle>)> = vec![
            (self.root_id, BoxConstraints::tight(viewport_size), Box::new(DefaultLayout))
        ];
        let mut response = None;
        let mut should_layout_child = force_layout;

        loop {
            let (request_id, result) = if let Some((request_id, box_constraints, layout)) = requests.last_mut() {
                let result = layout.measure(
                    *request_id,
                    *box_constraints,
                    response,
                    &self.tree,
                    &mut LayoutContext {
                        render_states: &mut self.render_states
                    }
                );
                (*request_id, result)
            } else {
                break;
            };

            match result {
                LayoutResult::Size(size) => {
                    let render_state = &mut self.render_states[request_id];

                    if render_state.rectangle.size != size {
                        render_state.rectangle.size = size;
                        render_state.dirty = size != Size::ZERO;
                        should_layout_child = true;
                    }

                    if requests.len() == 1 {
                        return size;
                    }

                    requests.pop();
                    response = Some((request_id, size));
                }
                LayoutResult::RequestChild(child_id, child_box_constraints) => {
                    let child_layout = self.tree[child_id].layout();
                    if should_layout_child || self.render_states[child_id].dirty {
                        requests.push((child_id, child_box_constraints, child_layout));
                        response = None;
                    } else {
                        let child_render_state = &self.render_states[child_id];
                        response = Some((child_id, child_render_state.rectangle.size));
                    }
                }
            }
        }

        unreachable!();
    }

    pub fn paint(&mut self, root_handle: &Handle, paint_context: &mut PaintContext<Handle>) where Handle: Clone {
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };
        let mut handle_stack: Vec<Option<NodeId>> = vec![];

        let mut node_id = self.root_id;
        let mut direction = WalkDirection::Downward;

        loop {
            let mut node = &self.tree[node_id];
            let mut render_state;

            loop {
                render_state = &self.render_states[node_id];

                match direction {
                    WalkDirection::Downward | WalkDirection::Sideward => {
                        if render_state.dirty {
                            break;
                        }
                    }
                    WalkDirection::Upward => break,
                }

                if let Some((next_node_id, next_direction)) = walk_next_node(node_id, self.root_id, node, &WalkDirection::Upward) {
                    node_id = next_node_id;
                    direction = next_direction;
                    node = &self.tree[node_id];
                } else {
                    break;
                }
            }

            let rectangle = render_state.rectangle;

            match direction {
                WalkDirection::Downward => {
                    absolute_point += latest_point;
                }
                WalkDirection::Sideward => {
                    handle_stack.pop();
                }
                WalkDirection::Upward => {
                    absolute_point -= rectangle.point;
                    handle_stack.pop();
                }
            };

            latest_point = rectangle.point;

            if direction == WalkDirection::Downward || direction == WalkDirection::Sideward {
                let widget = &**node;
                let parent_handle = handle_stack.last()
                    .copied()
                    .flatten()
                    .and_then(|node_id| self.render_states[node_id].handle.as_ref())
                    .map_or(ptr::null(), |handle| handle as *const Handle);
                let absolute_rectangle = Rectangle {
                    point: absolute_point + rectangle.point,
                    size: rectangle.size
                };

                let mut render_state = &mut self.render_states[node_id];

                if !render_state.mounted {
                    let handle = unsafe { parent_handle.as_ref().unwrap_or(root_handle) };
                    render_state.handle = widget.mount(handle, &rectangle, &mut *render_state.state);
                    render_state.mounted = true;
                }

                let paint_handle = if let Some(handle) = render_state.handle.as_ref() {
                    handle_stack.push(Some(node_id));
                    handle
                } else if let Some(handle) = unsafe { parent_handle.as_ref() } {
                    handle_stack.push(handle_stack.last().copied().flatten());
                    handle
                } else {
                    handle_stack.push(None);
                    root_handle
                };

                widget.paint(
                    paint_handle,
                    &absolute_rectangle,
                    &mut *render_state.state,
                    paint_context
                );

                for child_id in mem::take(&mut render_state.deleted_children) {
                    let mut deleted_render_state = self.render_states.remove(child_id);
                    if let Some(handle) = deleted_render_state.handle {
                        widget.unmount(
                            handle,
                            &deleted_render_state.rectangle,
                            &mut *deleted_render_state.state
                        );
                    }
                }
            }

            if let Some((next_node_id, next_direction)) = walk_next_node(node_id, self.root_id, node, &direction) {
                node_id = next_node_id;
                direction = next_direction;
            } else {
                break;
            }
        }
    }

    fn render_step(&mut self, node_id: NodeId) -> Option<NodeId> {
        let render_state = &mut self.render_states[node_id];
        if let Some(rendered_children) = render_state.rendered_children.take() {
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

    fn reconcile_children(&mut self, target_id: NodeId, children: Box<[Element<Handle>]>) {
        let mut old_keys: Vec<TypedKey> = Vec::new();
        let mut old_node_ids: Vec<Option<NodeId>> = Vec::new();

        for (index, (child_id, child)) in self.tree.children(target_id).enumerate() {
            let key = key_of(&***child, index);
            old_keys.push(key);
            old_node_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Handle>>> = Vec::with_capacity(children.len());

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
        result: ReconcileResult<NodeId, Element<Handle>>
    ) {
        match result {
            ReconcileResult::New(new_element) => {
                let render_state = RenderState::new(&*new_element.widget, new_element.children);
                let node_id = self.tree.append_child(target_id, new_element.widget);
                self.render_states.insert_at(node_id, render_state);
            }
            ReconcileResult::NewPlacement(ref_id, new_element) => {
                let render_state = RenderState::new(&*new_element.widget, new_element.children);
                let node_id = self.tree.insert_before(ref_id, new_element.widget);
                self.render_states.insert_at(node_id, render_state);
            }
            ReconcileResult::Update(target_id, new_element) => {
                self.update_render_state(target_id, new_element.widget, new_element.children);
            }
            ReconcileResult::UpdatePlacement(target_id, ref_id, new_element) => {
                self.update_render_state(target_id, new_element.widget, new_element.children);
                self.tree.move_position(target_id).insert_before(ref_id);
            }
            ReconcileResult::Deletion(target_id) => {
                let mut deleted_children = Vec::new();
                let parent = self.tree[target_id].parent();

                for (node_id, _) in self.tree.detach_subtree(target_id) {
                    deleted_children.push(node_id);
                    self.render_states.remove(node_id);
                }

                if let Some(parent_id) = parent {
                    self.render_states[parent_id].deleted_children = deleted_children;
                }
            }
        }
    }

    fn update_render_state(&mut self, node_id: NodeId, next_widget: Box<dyn WidgetDyn<Handle>>, children: Box<[Element<Handle>]>) {
        let current_widget = &mut *self.tree[node_id];
        let render_state = &mut self.render_states[node_id];

        if next_widget.should_render(&**current_widget, &children) {
            let prev_widget = mem::replace(current_widget, next_widget);

            current_widget.will_render(&*prev_widget, &children, &mut *render_state.state);

            let rendered_children = current_widget.render(children, &mut *render_state.state);
            render_state.dirty = true;
            render_state.rendered_children = Some(rendered_children);

            current_widget.did_render(&**current_widget, &mut *render_state.state);
        }

        for (parent_id, _) in self.tree.ancestors(node_id) {
            let state = &mut self.render_states[parent_id];
            if state.dirty {
                break;
            }
            state.dirty = true;
        }
    }
}

impl<Handle> fmt::Display for Updater<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.tree.format(
                self.root_id,
                |f, node_id, node| {
                    let render_state = &self.render_states[node_id];
                    write!(f, "<{}", node.name())?;
                    write!(f, " id=\"{}\"", node_id)?;
                    write!(f, " x=\"{}\"", render_state.rectangle.point.x)?;
                    write!(f, " y=\"{}\"", render_state.rectangle.point.y)?;
                    write!(f, " width=\"{}\"", render_state.rectangle.size.width)?;
                    write!(f, " height=\"{}\"", render_state.rectangle.size.height)?;
                    if render_state.dirty {
                        write!(f, " dirty")?;
                    }
                    write!(f, ">")?;
                    Ok(())
                },
                |f, _, node| write!(f, "</{}>", node.name())
            )
        )
    }
}

impl<Handle> RenderState<Handle> {
    pub fn new(widget: &dyn WidgetDyn<Handle>, children: Box<[Element<Handle>]>) -> Self {
        let mut initial_state = widget.initial_state();
        let rendered_children = widget.render(children, &mut *initial_state);
        Self {
            rendered_children: Some(rendered_children),
            deleted_children: Vec::new(),
            state: initial_state,
            dirty: true,
            rectangle: Rectangle::ZERO,
            handle: None,
            mounted: false,
        }
    }

    pub fn update(&mut self, widget: &dyn WidgetDyn<Handle>, children: Box<[Element<Handle>]>) {
        let rendered_children = widget.render(children, &mut *self.state);
        self.dirty = true;
        self.rendered_children = Some(rendered_children);
    }
}

impl<'a, Handle> Layouter for LayoutContext<'a, Handle> {
    fn get_rectangle(&self, node_id: NodeId) -> &Rectangle {
        &self.render_states[node_id].rectangle
    }

    fn get_rectangle_mut(&mut self, node_id: NodeId) -> &mut Rectangle {
        &mut self.render_states[node_id].rectangle
    }
}

fn key_of<Handle>(widget: &dyn WidgetDyn<Handle>, index: usize) -> TypedKey {
    match widget.key() {
        Some(key) => TypedKey::Keyed(widget.as_any().type_id(), key),
        None => TypedKey::Indexed(widget.as_any().type_id(), index),
    }
}
