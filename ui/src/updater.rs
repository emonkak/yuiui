use std::any::TypeId;
use std::fmt;
use std::mem;

use geometrics::{Point, Rectangle, Size};
use layout::{BoxConstraints, LayoutState, LayoutContext, LayoutResult};
use paint::PaintContext;
use reconciler::{Reconciler, ReconcileResult};
use slot_vec::SlotVec;
use tree::{NodeId, Tree, WalkDirection};
use widget::null::Null;
use widget::widget::{DefaultLayout, Element, Key, Layout, PaintState, RenderState, WidgetDyn, WidgetTree};

#[derive(Debug)]
pub struct Updater<Handle> {
    tree: WidgetTree<Handle>,
    root_id: NodeId,
    render_states: SlotVec<RenderState<Handle>>,
    layout_states: SlotVec<LayoutState>,
    paint_states: SlotVec<PaintState<Handle>>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

// Render:
//  - &mut WidgetTree
//  - &mut RenderStates
// Layout:
//  - &WidgetTree
//  - &mut RenderStates
//  - &mut LayoutStates
// Paint:
//  - &WidgetTree
//  - &RenderStates
//  - &LayoutStates
//  - &mut PaintStates
impl<Handle> Updater<Handle> {
    pub fn new() -> Self {
        let mut tree = Tree::new();
        let mut render_states = SlotVec::new();
        let mut layout_states = SlotVec::new();
        let mut paint_states = SlotVec::new();

        let root_id = tree.attach(Box::new(Null) as Box<dyn WidgetDyn<Handle>>);

        render_states.insert_at(root_id, RenderState::new(&Null, Box::new([])));
        layout_states.insert_at(root_id, Default::default());
        paint_states.insert_at(root_id, Default::default());

        Self {
            tree,
            root_id,
            render_states,
            layout_states,
            paint_states,
        }
    }

    pub fn update(&mut self, element: Element<Handle>) {
        self.update_state(self.root_id, Element::new(Null, [element]));
    }

    pub fn render(&mut self) {
        let mut current = self.root_id;
        while let Some(next) = self.render_step(current) {
            current = next;
        }
    }

    pub fn layout(&mut self, viewport_size: Size, force_layout: bool) -> Size {
        let mut requests: Vec<(NodeId, BoxConstraints, Box<dyn Layout<Handle>>)> = vec![(self.root_id, BoxConstraints::tight(viewport_size), Box::new(DefaultLayout))];
        let mut response = None;
        let mut should_layout_child = force_layout;

        loop {
            let (request_id, result) = if let Some((request_id, box_constraints, layout)) = requests.last_mut() {
                let result = layout.measure(
                    *request_id,
                    *box_constraints,
                    response,
                    &self.tree,
                    &mut LayoutContext::new(&mut self.layout_states)
                );
                (*request_id, result)
            } else {
                break;
            };

            match result {
                LayoutResult::Size(size) => {
                    let render_state = &mut self.render_states[request_id];
                    let mut deleted_children = mem::take(&mut render_state.deleted_children);

                    for &child_id in deleted_children.iter() {
                        self.layout_states.remove(child_id);
                    }

                    let layout_state = self.layout_states.get_or_insert_default(request_id);
                    if layout_state.deleted_children.len() > 0 {
                        layout_state.deleted_children.append(&mut deleted_children);
                    } else {
                        layout_state.deleted_children = deleted_children;
                    }

                    if layout_state.rectangle.size != size {
                        layout_state.rectangle.size = size;
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
                        let child_layout_state = &self.layout_states[child_id];
                        response = Some((child_id, child_layout_state.rectangle.size));
                    }
                }
            }
        }

        unreachable!();
    }

    pub fn paint(&mut self, parent_handle: &Handle, paint_context: &mut PaintContext<Handle>) where Handle: Clone {
        let mut handle = parent_handle.clone();
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };
        let render_states = &self.render_states;

        for (node_id, node, direction) in self.tree.walk_filter(
            self.root_id,
            |node_id, _| render_states[node_id].dirty,
        ) {
            let widget = &**node;
            let layout_state = &mut self.layout_states[node_id];
            let rectangle = &layout_state.rectangle;

            for child_id in mem::take(&mut layout_state.deleted_children) {
                let paint_state = self.paint_states.remove(child_id);
                if let Some(handle) = paint_state.handle {
                    widget.unmount(handle);
                }
            }

            match direction {
                WalkDirection::Downward => {
                    absolute_point += latest_point;
                    let paint_rectangle = Rectangle {
                        point: absolute_point + rectangle.point,
                        size: rectangle.size
                    };
                    let paint_state = self.paint_states.get_or_insert_default(node_id);
                    if !paint_state.mounted {
                        paint_state.handle = widget.mount(&handle, rectangle);
                    }
                    widget.paint(&paint_rectangle, &handle, paint_context);
                    if let Some(next_handle) = paint_state.handle.as_ref() {
                        handle = next_handle.clone();
                    }
                }
                WalkDirection::Sideward => {
                    let paint_rectangle = Rectangle {
                        point: absolute_point + rectangle.point,
                        size: rectangle.size,
                    };
                    let paint_state = self.paint_states.get_or_insert_default(node_id);
                    if !paint_state.mounted {
                        paint_state.handle = widget.mount(&handle, rectangle);
                    }
                    widget.paint(&paint_rectangle, &handle, paint_context);
                }
                WalkDirection::Upward => {
                    absolute_point -= rectangle.point;
                }
            }
            latest_point = rectangle.point;
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

    fn reconcile_children(
        &mut self,
        target_id: NodeId,
        children: Box<[Element<Handle>]>,
    ) {
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
                self.init_state(self.tree.next_node_id(), &*new_element.widget, new_element.children);
                self.tree.append_child(target_id, new_element.widget);
            }
            ReconcileResult::NewPlacement(ref_id, new_element) => {
                self.init_state(self.tree.next_node_id(), &*new_element.widget, new_element.children);
                self.tree.insert_before(ref_id, new_element.widget);
            }
            ReconcileResult::Update(target_id, new_element) => {
                self.update_state(target_id, new_element);
            }
            ReconcileResult::UpdatePlacement(target_id, ref_id, new_element) => {
                self.update_state(target_id, new_element);
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

    fn init_state(&mut self, node_id: NodeId, widget: &dyn WidgetDyn<Handle>, children: Box<[Element<Handle>]>) {
        self.render_states.insert_at(node_id, RenderState::new(widget, children));
        self.layout_states.insert_at(node_id, Default::default());
        self.paint_states.insert_at(node_id, Default::default());
    }

    fn update_state(&mut self, node_id: NodeId, element: Element<Handle>) {
        let current_widget = &mut *self.tree[node_id];
        let state = &mut self.render_states[node_id];

        if element.widget.should_update(&**current_widget, &element.children) {
            let prev_widget = mem::replace(current_widget, element.widget);

            current_widget.will_update(&*prev_widget, &element.children);

            state.update(&**current_widget, element.children);

            current_widget.did_update(&**current_widget);
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

impl<Handle: fmt::Debug> fmt::Display for Updater<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.tree.format(
                self.root_id,
                |f, node_id, node| {
                    let render_state = &self.render_states[node_id];
                    let layout_state = &self.layout_states[node_id];
                    write!(f, "<{}", node.name())?;
                    write!(f, " id=\"{}\"", node_id)?;
                    write!(f, " x=\"{}\"", layout_state.rectangle.point.x)?;
                    write!(f, " y=\"{}\"", layout_state.rectangle.point.y)?;
                    write!(f, " width=\"{}\"", layout_state.rectangle.size.width)?;
                    write!(f, " height=\"{}\"", layout_state.rectangle.size.height)?;
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

fn key_of<Handle>(widget: &dyn WidgetDyn<Handle>, index: usize) -> TypedKey {
    match widget.key() {
        Some(key) => TypedKey::Keyed(widget.as_any().type_id(), key),
        None => TypedKey::Indexed(widget.as_any().type_id(), index),
    }
}
