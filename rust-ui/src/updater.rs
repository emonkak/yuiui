use std::any::{Any, TypeId};
use std::fmt;
use std::mem;

use crate::event::{EventContext, EventManager, EventType};
use crate::generator::GeneratorState;
use crate::geometrics::{Point, Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::lifecycle::{Lifecycle, LifecycleContext};
use crate::paint::{PaintContext, PaintState};
use crate::reconciler::{ReconcileResult, Reconciler};
use crate::render::RenderState;
use crate::slot_vec::SlotVec;
use crate::tree::walk::{walk_next_node, WalkDirection};
use crate::tree::{NodeId, Tree};
use crate::widget::element::{Children, Element, Key};
use crate::widget::null::Null;
use crate::widget::{BoxedWidget, PolymophicWidget, WidgetTree};

#[derive(Debug)]
pub struct Updater<Handle> {
    tree: WidgetTree<Handle>,
    root_id: NodeId,
    render_states: SlotVec<RenderState<Handle>>,
    paint_states: SlotVec<PaintState>,
    event_manager: EventManager<Handle>,
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
        let mut paint_states = SlotVec::new();

        let root_id = tree.attach(Box::new(Null) as BoxedWidget<Handle>);

        render_states.insert_at(root_id, RenderState::new(
            PolymophicWidget::<Handle>::initial_state(&Null),
            Vec::new(),
            None
        ));
        paint_states.insert_at(root_id, PaintState::default());

        Self {
            tree,
            root_id,
            render_states,
            paint_states,
            event_manager: EventManager::new(),
        }
    }

    pub fn update(&mut self, element: Element<Handle>) {
        self.update_render_state(self.root_id, Box::new(Null), vec![element], None);
    }

    pub fn render(&mut self) {
        let mut current = self.root_id;
        while let Some(next) = self.render_step(current) {
            current = next;
        }
    }

    pub fn layout(&mut self, viewport_size: Size, force_layout: bool) -> Size {
        let mut layout_stack = Vec::new();
        let mut current_id = self.root_id;
        let mut current_layout = self.tree[self.root_id].layout(
            self.root_id,
            BoxConstraints::tight(&viewport_size),
            &self.tree,
            &*self.render_states[self.root_id].state,
        );
        let mut calculated_size = Size::ZERO;

        loop {
            match current_layout.resume(calculated_size) {
                GeneratorState::Yielded(LayoutRequest::LayoutChild(
                    child_id,
                    child_box_constraints,
                )) => {
                    let child_render_state = &self.render_states[child_id];
                    if force_layout || child_render_state.dirty {
                        let layout = self.tree[child_id].layout(
                            child_id,
                            child_box_constraints,
                            &self.tree,
                            &*child_render_state.state,
                        );
                        layout_stack.push((child_id, layout));
                    } else {
                        calculated_size = self.paint_states[child_id].rectangle.size;
                    }
                }
                GeneratorState::Yielded(LayoutRequest::ArrangeChild(child_id, point)) => {
                    let mut paint_state = self.paint_states.get_or_insert_default(child_id);
                    paint_state.rectangle.point = point;
                    calculated_size = paint_state.rectangle.size;
                }
                GeneratorState::Complete(size) => {
                    let mut paint_state = self.paint_states.get_or_insert_default(current_id);
                    paint_state.rectangle.size = size;
                    calculated_size = size;

                    if let Some((next_id, next_layout)) = layout_stack.pop() {
                        current_id = next_id;
                        current_layout = next_layout;
                    } else {
                        break;
                    }
                }
            }
        }

        calculated_size
    }

    pub fn paint(&mut self, paint_context: &mut dyn PaintContext<Handle>) {
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };

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

                if let Some((next_node_id, next_direction)) =
                    walk_next_node(node_id, self.root_id, node, &WalkDirection::Upward)
                {
                    node_id = next_node_id;
                    direction = next_direction;
                    node = &self.tree[node_id];
                } else {
                    break;
                }
            }

            let rectangle = self.paint_states[node_id].rectangle;

            if direction == WalkDirection::Downward {
                absolute_point += latest_point;
            } else if direction == WalkDirection::Upward {
                absolute_point -= rectangle.point;
            }

            latest_point = rectangle.point;

            if direction == WalkDirection::Downward || direction == WalkDirection::Sideward {
                let widget = &**node;
                let absolute_rectangle = Rectangle {
                    point: absolute_point + rectangle.point,
                    size: rectangle.size,
                };

                let mut render_state = &mut self.render_states[node_id];
                let mut context = LifecycleContext {
                    event_manager: &mut self.event_manager,
                };

                if !render_state.mounted {
                    widget.lifecycle(Lifecycle::DidMount, &mut *render_state.state, &mut context);
                    render_state.mounted = true;
                }

                widget.paint(&absolute_rectangle, &mut *render_state.state, paint_context);

                for (child_id, child_widget) in mem::take(&mut render_state.deleted_children) {
                    let mut render_state = self.render_states.remove(child_id);
                    self.paint_states.remove(child_id);
                    child_widget.lifecycle(
                        Lifecycle::DidUnmount,
                        &mut *render_state.state,
                        &mut context,
                    );
                }
            }

            if let Some((next_node_id, next_direction)) =
                walk_next_node(node_id, self.root_id, node, &direction)
            {
                node_id = next_node_id;
                direction = next_direction;
            } else {
                break;
            }
        }
    }

    pub fn dispatch_events<EventType>(&mut self, event: EventType::Event)
    where
        EventType: self::EventType + 'static,
    {
        let boxed_event: Box<dyn Any> = Box::new(event);
        let mut context = EventContext {};
        for handler in self.event_manager.get::<EventType>() {
            handler.dispatch(
                &self.tree,
                &mut self.render_states,
                &boxed_event,
                &mut context,
            )
        }
    }

    fn render_step(&mut self, node_id: NodeId) -> Option<NodeId> {
        let widget = &self.tree[node_id];
        let render_state = &mut self.render_states[node_id];
        if let Some(children) = render_state.children.take(){
            let rendered_children = widget.render(children, &mut *render_state.state, node_id);
            self.reconcile_children(node_id, rendered_children.into());
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
                .filter(|&parent_id| parent_id != self.root_id)
            {
                currnet_node_id = parent_id;
            } else {
                break;
            }
        }

        None
    }

    fn reconcile_children(&mut self, target_id: NodeId, children: Children<Handle>) {
        let mut old_keys: Vec<TypedKey> = Vec::new();
        let mut old_node_ids: Vec<Option<NodeId>> = Vec::new();

        for (index, (child_id, child)) in self.tree.children(target_id).enumerate() {
            let child_render_state = &self.render_states[child_id];
            let key = key_of(&***child, index, child_render_state.key);
            old_keys.push(key);
            old_node_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Handle>>> = Vec::with_capacity(children.len());

        for (index, element) in children.into_iter().enumerate() {
            let key = key_of(&*element.widget, index, element.key);
            new_keys.push(key);
            new_elements.push(Some(element));
        }

        let reconciler =
            Reconciler::new(&old_keys, &mut old_node_ids, &new_keys, &mut new_elements);

        for result in reconciler {
            self.handle_reconcile_result(target_id, result);
        }
    }

    fn handle_reconcile_result(
        &mut self,
        target_id: NodeId,
        result: ReconcileResult<NodeId, Element<Handle>>,
    ) {
        match result {
            ReconcileResult::New(new_element) => {
                let node_id = self.tree.next_node_id();
                let render_state = RenderState::new(
                    new_element.widget.initial_state(),
                    new_element.children,
                    new_element.key,
                );
                self.tree.append_child(target_id, new_element.widget);
                self.render_states.insert_at(node_id, render_state);
            }
            ReconcileResult::NewPlacement(ref_id, new_element) => {
                let node_id = self.tree.next_node_id();
                let render_state = RenderState::new(
                    new_element.widget.initial_state(),
                    new_element.children,
                    new_element.key,
                );
                self.tree.insert_before(ref_id, new_element.widget);
                self.render_states.insert_at(node_id, render_state);
            }
            ReconcileResult::Update(target_id, new_element) => {
                self.update_render_state(
                    target_id,
                    new_element.widget,
                    new_element.children,
                    new_element.key,
                );
            }
            ReconcileResult::UpdatePlacement(target_id, ref_id, new_element) => {
                self.update_render_state(
                    target_id,
                    new_element.widget,
                    new_element.children,
                    new_element.key,
                );
                self.tree.move_position(target_id).insert_before(ref_id);
            }
            ReconcileResult::Deletion(target_id) => {
                let mut deleted_children = Vec::new();

                for (node_id, node) in self.tree.detach_subtree(target_id) {
                    let render_state = &mut self.render_states[node_id];
                    let mut context = LifecycleContext {
                        event_manager: &mut self.event_manager,
                    };

                    node.lifecycle(
                        Lifecycle::WillUnmount,
                        &mut *render_state.state,
                        &mut context,
                    );

                    deleted_children.push((node_id, node.into_data()));
                }

                let parent = self.tree[target_id].parent();

                if let Some(parent_id) = parent {
                    self.render_states[parent_id].deleted_children = deleted_children;
                }
            }
        }
    }

    pub fn update_render_state(
        &mut self,
        node_id: NodeId,
        new_widget: BoxedWidget<Handle>,
        children: Children<Handle>,
        key: Option<Key>,
    ) {
        let current_widget = &mut *self.tree[node_id];
        let render_state = &mut self.render_states[node_id];

        if new_widget.should_update(&**current_widget, &*render_state.state) {
            let prev_widget = mem::replace(current_widget, new_widget);
            let mut context = LifecycleContext {
                event_manager: &mut self.event_manager,
            };

            current_widget.lifecycle(
                Lifecycle::WillUpdate(&*prev_widget),
                &mut *render_state.state,
                &mut context,
            );

            render_state.children = Some(children);
            render_state.dirty = true;
            render_state.key = key;
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
            self.tree.to_formatter(
                self.root_id,
                |f, node_id, node| {
                    let render_state = &self.render_states[node_id];
                    let paint_state = &self.paint_states[node_id];
                    write!(f, "<{:?}", node)?;
                    write!(f, " id=\"{}\"", node_id)?;
                    write!(f, " x=\"{}\"", paint_state.rectangle.point.x)?;
                    write!(f, " y=\"{}\"", paint_state.rectangle.point.y)?;
                    write!(f, " width=\"{}\"", paint_state.rectangle.size.width)?;
                    write!(f, " height=\"{}\"", paint_state.rectangle.size.height)?;
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

fn key_of<Handle>(
    widget: &dyn PolymophicWidget<Handle>,
    index: usize,
    key: Option<Key>,
) -> TypedKey {
    match key {
        Some(key) => TypedKey::Keyed(widget.as_any().type_id(), key),
        None => TypedKey::Indexed(widget.as_any().type_id(), index),
    }
}
