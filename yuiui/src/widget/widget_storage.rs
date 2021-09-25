use std::any::{Any, TypeId};
use std::fmt;
use std::mem;
use std::rc::Rc;
use yuiui_support::slot_tree::{CursorMut, NodeId, SlotTree};

use super::reconciler::{ReconcileResult, Reconciler};
use super::root::Root;
use super::{
    Attributes, BoxedComponent, BoxedWidget, Command, ComponentElement, Element, Key, Lifecycle, Widget, WidgetElement,
};
use crate::geometrics::{BoxConstraints, Point, Rectangle, Size, Viewport};
use crate::graphics::Primitive;
use crate::event::WindowEvent;

type ComponentIndex = usize;

#[derive(Debug)]
pub struct WidgetStorage<Message> {
    widget_tree: SlotTree<WidgetNode<Message>>,
}

impl<Message> WidgetStorage<Message> {
    pub fn new(element: Element<Message>, viewport: Viewport) -> Self
    where
        Message: 'static,
    {
        let root = {
            let widget = Root::new(viewport).into_boxed();
            let instance = WidgetPod::new(widget, vec![element]);
            WidgetNode {
                instance: Some(instance),
                component_stack: Vec::new(),
            }
        };
        Self {
            widget_tree: SlotTree::new(root),
        }
    }

    pub fn render(
        &mut self,
        id: NodeId,
        component_index: ComponentIndex,
        root: NodeId,
    ) -> RenderResult<Message> {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let component_stack = &mut cursor.current().data_mut().component_stack;

        let mut commands = Vec::new();
        let mut removed_ids = Vec::new();

        let next = if component_index < component_stack.len() {
            let instance = &mut component_stack[component_index];
            let is_updated = if let Some(pending_element) = instance.pending_element.take() {
                instance.update(pending_element)
            } else {
                true
            };

            if is_updated {
                let children = vec![instance.render()];
                let reconciler = create_reconciler(&mut cursor, children, component_index);
                for result in reconciler {
                    self.update_child(id, result, true, &mut commands, &mut removed_ids)
                }
            }

            Some((id, component_index + 1))
        } else {
            let instance = cursor.current().data_mut().borrow_mut();
            let (is_updated, response) = if let Some(pending_element) = instance.pending_element.take() {
                instance.update(pending_element)
            } else {
                let response = instance.on_lifecycle(Lifecycle::OnMount);
                (true, response)
            };

            if let Some(command) = response {
                commands.push((id, command));
            }

            if is_updated {
                if let Some(children) = instance.pending_children.take() {
                    let reconciler = create_reconciler(&mut cursor, children, component_index);
                    for result in reconciler {
                        self.update_child(id, result, false, &mut commands, &mut removed_ids)
                    }
                }
            }

            self.widget_tree
                .cursor(id)
                .descendants_from(root)
                .next()
                .map(|(next_id, _)| (next_id, 0))
        };

        RenderResult {
            next,
            commands,
            removed_ids,
        }
    }

    pub fn layout(&mut self, id: NodeId) -> NodeId {
        let mut root_id = id;

        loop {
            let mut cursor = self.widget_tree.cursor_mut(root_id);
            let mut instance = cursor
                .current()
                .data_mut()
                .instance
                .take()
                .expect("widget is currently in use elsewhere");

            let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
            let parent = cursor.current().parent();

            let mut context = LayoutContext { storage: self };

            let box_constraints = instance.box_constraints;
            let is_changed = instance.layout(box_constraints, &children, &mut context);

            let mut cursor = self.widget_tree.cursor_mut(id);
            cursor.current().data_mut().instance = Some(instance);

            match parent {
                Some(parent) if is_changed => root_id = parent,
                _ => break,
            }
        }

        root_id
    }

    pub fn draw(&mut self, id: NodeId) -> (Primitive, Rectangle) {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let mut widget = cursor
            .current()
            .data_mut()
            .instance
            .take()
            .expect("widget is currently in use elsewhere");

        let origin = cursor.ancestors().fold(Point::ZERO, |origin, (_, node)| {
            let instance = node.data_mut().borrow_mut();
            instance.needs_draw = true;
            origin + instance.position
        });
        let bounds = Rectangle::new(origin + widget.position, widget.size);
        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();

        let mut context = DrawContext {
            storage: self,
            origin: widget.position,
        };
        let primitive = widget.draw(bounds, &children, &mut context);

        let mut cursor = self.widget_tree.cursor_mut(id);
        cursor.current().data_mut().instance = Some(widget);

        (primitive, bounds)
    }

    fn update_child(
        &mut self,
        parent: NodeId,
        result: ReconcileResult<ReconcilementId, Element<Message>>,
        in_component_rendering: bool,
        commands: &mut Vec<(NodeId, Command<Message>)>,
        removed_ids: &mut Vec<NodeId>,
    ) {
        match result {
            ReconcileResult::Append(element) => {
                let mut cursor = self.widget_tree.cursor_mut(parent);
                match element {
                    Element::WidgetElement(element) => {
                        let instance = WidgetPod::from_element(element);
                        if in_component_rendering {
                            let widget_node = cursor.current().data_mut();
                            widget_node.instance = Some(instance);
                        } else {
                            cursor.append_child(WidgetNode {
                                instance: Some(instance),
                                component_stack: Vec::new(),
                            });
                        }
                    }
                    Element::ComponentElement(element) => {
                        let instance = ComponentPod::from_element(element);
                        if in_component_rendering {
                            cursor.current().data_mut().component_stack.push(instance);
                        } else {
                            cursor.append_child(WidgetNode {
                                instance: None,
                                component_stack: vec![instance],
                            });
                        }
                    }
                }
            }
            ReconcileResult::Insert(reference, element) => {
                let mut cursor = self.widget_tree.cursor_mut(reference.node_id());
                match element {
                    Element::WidgetElement(element) => {
                        let instance = WidgetPod::from_element(element);
                        cursor.insert_before(WidgetNode {
                            instance: Some(instance),
                            component_stack: Vec::new(),
                        });
                    }
                    Element::ComponentElement(element) => {
                        let instance = ComponentPod::from_element(element);
                        cursor.insert_before(WidgetNode {
                            instance: None,
                            component_stack: vec![instance],
                        });
                    }
                }
            }
            ReconcileResult::Update(ReconcilementId::Widget(id), element) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let instance = cursor.current().data_mut().borrow_mut();
                match element {
                    Element::WidgetElement(element) => {
                        instance.pending_element = Some(element)
                    }
                    _ => unreachable!("element kind mismatch"),
                };
            }
            ReconcileResult::Update(ReconcilementId::Component(id, component_index), element) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let instance = &mut cursor.current().data_mut().component_stack[component_index];
                match element {
                    Element::ComponentElement(element) => instance.pending_element = Some(element),
                    _ => unreachable!("element kind mismatch"),
                }
            }
            ReconcileResult::UpdateAndMove(
                ReconcilementId::Widget(id),
                ReconcilementId::Widget(reference_id),
                element,
            ) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let instance = cursor.current().data_mut().borrow_mut();
                let _ = match element {
                    Element::WidgetElement(element) => {
                        instance.pending_element = Some(element)
                    }
                    _ => unreachable!("element kind mismatch"),
                };
                cursor.move_before(reference_id);
            }
            ReconcileResult::UpdateAndMove(
                ReconcilementId::Component(id, component_index),
                ReconcilementId::Component(reference_id, _),
                element,
            ) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let instance = &mut cursor.current().data_mut().component_stack[component_index];
                match element {
                    Element::ComponentElement(element) => instance.pending_element = Some(element),
                    _ => unreachable!("element kind mismatch"),
                }
                cursor.move_before(reference_id);
            }
            ReconcileResult::Remove(ReconcilementId::Widget(id)) => {
                let cursor = self.widget_tree.cursor_mut(id);
                for (id, node) in cursor.drain_subtree() {
                    let mut widget_node = node.into_data();
                    let response = widget_node.borrow_mut().on_lifecycle(Lifecycle::OnUnmount);
                    if let Some(command) = response {
                        commands.push((id, command));
                    }
                    removed_ids.push(id);
                }
            }
            ReconcileResult::Remove(ReconcilementId::Component(id, component_index)) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let mut widget_node = cursor.current().data_mut();
                let _ = widget_node.component_stack.drain(component_index..);
                if component_index > 0 {
                    widget_node.instance = None;
                    for (id, node) in cursor.drain_descendants() {
                        let mut widget_node = node.into_data();
                        let response = widget_node.borrow_mut().on_lifecycle(Lifecycle::OnUnmount);
                        if let Some(command) = response {
                            commands.push((id, command));
                        }
                        removed_ids.push(id);
                    }
                } else {
                    for (id, node) in cursor.drain_subtree() {
                        let mut widget_node = node.into_data();
                        let response = widget_node.borrow_mut().on_lifecycle(Lifecycle::OnUnmount);
                        if let Some(command) = response {
                            commands.push((id, command));
                        }
                        removed_ids.push(id);
                    }
                }
            }
            _ => unreachable!("element kind mismatch"),
        }
    }

    fn layout_child(&mut self, id: NodeId, box_constraints: BoxConstraints) -> Size {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let mut instance = cursor
            .current()
            .data_mut()
            .instance
            .take()
            .expect("widget is currently in use elsewhere");

        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
        let mut context = LayoutContext { storage: self };

        instance.layout(box_constraints, &children, &mut context);
        instance.box_constraints = box_constraints;
        let size = instance.size;

        let mut cursor = self.widget_tree.cursor_mut(id);
        cursor.current().data_mut().instance = Some(instance);

        size
    }

    fn draw_child(&mut self, id: NodeId, origin: Point) -> Primitive {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let mut widget = cursor
            .current()
            .data_mut()
            .instance
            .take()
            .expect("widget is currently in use elsewhere");

        let bounds = Rectangle::new(origin + widget.position, widget.size);
        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
        let mut context = DrawContext {
            storage: self,
            origin: widget.position,
        };
        let primitive = widget.draw(bounds, &children, &mut context);

        let mut cursor = self.widget_tree.cursor_mut(id);
        cursor.current().data_mut().instance = Some(widget);

        primitive
    }

    fn get_widget(&self, id: NodeId) -> &WidgetPod<Message> {
        self.widget_tree
            .cursor(id)
            .current()
            .data()
            .instance
            .as_ref()
            .expect("widget is currently in use elsewhere")
    }

    fn get_widget_mut(&mut self, id: NodeId) -> &mut WidgetPod<Message> {
        self.widget_tree
            .cursor_mut(id)
            .current()
            .data_mut()
            .instance
            .as_mut()
            .expect("widget is currently in use elsewhere")
    }
}

impl<Message: fmt::Debug> fmt::Display for WidgetStorage<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.widget_tree.fmt(f)
    }
}

pub struct RenderResult<Message> {
    pub next: Option<(NodeId, ComponentIndex)>,
    pub commands: Vec<(NodeId, Command<Message>)>,
    pub removed_ids: Vec<NodeId>,
}

#[derive(Debug)]
struct WidgetNode<Message> {
    instance: Option<WidgetPod<Message>>,
    component_stack: Vec<ComponentPod<Message>>,
}

impl<Message> WidgetNode<Message> {
    fn borrow(&self) -> &WidgetPod<Message> {
        self.instance
            .as_ref()
            .expect("widget is currently in use elsewhere")
    }

    fn borrow_mut(&mut self) -> &mut WidgetPod<Message> {
        self.instance
            .as_mut()
            .expect("widget is currently in use elsewhere")
    }
}

#[derive(Debug)]
struct WidgetPod<Message> {
    widget: BoxedWidget<Message>,
    attributes: Rc<Attributes>,
    pending_children: Option<Vec<Element<Message>>>,
    key: Option<Key>,
    state: Box<dyn Any>,
    pending_element: Option<WidgetElement<Message>>,
    box_constraints: BoxConstraints,
    position: Point,
    size: Size,
    draw_cache: Option<Primitive>,
    needs_layout: bool,
    needs_draw: bool,
}

impl<Message> WidgetPod<Message> {
    fn new(widget: BoxedWidget<Message>, children: Vec<Element<Message>>) -> Self {
        let state = widget.initial_state();
        Self {
            widget,
            attributes: Rc::new(Attributes::new()),
            pending_children: Some(children),
            key: None,
            state,
            pending_element: None,
            box_constraints: BoxConstraints::LOOSE,
            position: Point::ZERO,
            size: Size::ZERO,
            draw_cache: None,
            needs_layout: true,
            needs_draw: true,
        }
    }

    fn from_element(element: WidgetElement<Message>) -> Self {
        let state = element.widget.initial_state();
        Self {
            widget: element.widget,
            attributes: element.attributes,
            pending_children: Some(element.children),
            key: element.key,
            state,
            pending_element: None,
            box_constraints: BoxConstraints::LOOSE,
            position: Point::ZERO,
            size: Size::ZERO,
            draw_cache: None,
            needs_layout: true,
            needs_draw: true,
        }
    }

    fn update(&mut self, element: WidgetElement<Message>) -> (bool, Option<Command<Message>>) {
        let should_update = !element.children.is_empty()
            || &*self.attributes != &*element.attributes
            || self.widget.should_update(
                element.widget.as_any(),
                &self.state,
            );
        let old_widget = mem::replace(&mut self.widget, element.widget);

        self.attributes = element.attributes;
        self.pending_children = Some(element.children);
        self.needs_layout = should_update;
        self.needs_draw = should_update;

        if should_update {
            let response = self.on_lifecycle(Lifecycle::OnUpdate(old_widget.as_any()));
            (true, response)
        } else {
            (false, None)
        }
    }

    fn on_event(&mut self, event: WindowEvent) -> Option<Command<Message>> {
        self.widget.on_event(event, &mut self.state)
    }

    fn on_lifecycle(&mut self, lifecycle: Lifecycle<&dyn Any>) -> Option<Command<Message>> {
        self.widget.on_lifecycle(lifecycle, &mut self.state)
    }

    fn layout(
        &mut self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<Message>,
    ) -> bool {
        if !self.needs_layout && self.box_constraints == box_constraints {
            return false;
        }
        let size = self
            .widget
            .layout(box_constraints, children, context, &mut self.state);
        self.needs_layout = false;
        if size != self.size {
            self.size = size;
            self.needs_draw = true;
            true
        } else {
            false
        }
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        children: &[NodeId],
        context: &mut DrawContext<Message>,
    ) -> Primitive {
        if !self.needs_draw {
            if let Some(primitive) = &self.draw_cache {
                return primitive.clone();
            }
        }
        let primitive = self.widget.draw(bounds, children, context, &mut self.state);
        self.draw_cache = Some(primitive.clone());
        self.needs_draw = false;
        primitive
    }

    fn as_any(&self) -> &dyn Any {
        self.widget.as_any()
    }
}

#[derive(Debug)]
struct ComponentPod<Message> {
    component: BoxedComponent<Message>,
    attributes: Rc<Attributes>,
    children: Vec<Element<Message>>,
    state: Box<dyn Any>,
    key: Option<Key>,
    pending_element: Option<ComponentElement<Message>>,
}

impl<Message> ComponentPod<Message> {
    fn from_element(element: ComponentElement<Message>) -> Self {
        let state = element.component.initial_state();
        Self {
            component: element.component,
            attributes: element.attributes,
            children: element.children,
            key: element.key,
            state,
            pending_element: None,
        }
    }

    fn update(&mut self, element: ComponentElement<Message>) -> bool {
        let should_update = &*self.attributes != &*element.attributes
            || self.component.should_update(
                element.component.as_any(),
                &self.children,
                &element.children,
                &self.state,
            );

        self.component = element.component;
        self.attributes = element.attributes;
        self.children = element.children;

        should_update
    }

    fn render(&self) -> Element<Message> {
        self.component.render(&self.children, &self.state)
    }

    fn as_any(&self) -> &dyn Any {
        self.component.as_any()
    }
}

#[derive(Debug)]
pub struct LayoutContext<'a, Message> {
    storage: &'a mut WidgetStorage<Message>,
}

impl<'a, Message> LayoutContext<'a, Message> {
    pub fn get_size(&mut self, id: NodeId) -> Size {
        let widget = self.storage.get_widget(id);
        widget.size
    }

    pub fn get_attributes(&self, id: NodeId) -> &Attributes {
        &*self.storage.get_widget(id).attributes
    }

    pub fn set_position(&mut self, id: NodeId, position: Point) {
        let widget = self.storage.get_widget_mut(id);
        widget.position = position;
    }

    pub fn layout_child(&mut self, id: NodeId, box_constraints: BoxConstraints) -> Size {
        self.storage.layout_child(id, box_constraints)
    }
}

#[derive(Debug)]
pub struct DrawContext<'a, Message> {
    storage: &'a mut WidgetStorage<Message>,
    origin: Point,
}

impl<'a, Message> DrawContext<'a, Message> {
    pub fn draw_child(&mut self, id: NodeId) -> Primitive {
        self.storage.draw_child(id, self.origin)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ReconcilementKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl ReconcilementKey {
    fn new(type_id: TypeId, key: Option<Key>, index: usize) -> Self {
        match key {
            Some(key) => Self::Keyed(type_id, key),
            None => Self::Indexed(type_id, index),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReconcilementId {
    Widget(NodeId),
    Component(NodeId, usize),
}

impl ReconcilementId {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Widget(id) => *id,
            Self::Component(id, _) => *id,
        }
    }
}

fn create_reconciler<Message>(
    cursor: &mut CursorMut<WidgetNode<Message>>,
    children: Vec<Element<Message>>,
    component_index: ComponentIndex,
) -> Reconciler<ReconcilementKey, ReconcilementId, Element<Message>> {
    let mut old_keys: Vec<ReconcilementKey> = Vec::new();
    let mut old_ids: Vec<Option<ReconcilementId>> = Vec::new();

    for (index, (child_id, child)) in cursor.children().enumerate() {
        let child_node = child.data();
        let (key, id) = if component_index < child_node.component_stack.len() {
            let instance = &child_node.component_stack[component_index];
            let type_id = instance.as_any().type_id();
            let key = ReconcilementKey::new(type_id, instance.key, index);
            let id = ReconcilementId::Component(child_id, component_index);
            (key, id)
        } else {
            let instance = child_node.borrow();
            let type_id = instance.as_any().type_id();
            let key = ReconcilementKey::new(type_id, instance.key, index);
            let id = ReconcilementId::Widget(child_id);
            (key, id)
        };
        old_keys.push(key);
        old_ids.push(Some(id));
    }

    let mut new_keys: Vec<ReconcilementKey> = Vec::with_capacity(children.len());
    let mut new_elements: Vec<Option<Element<Message>>> = Vec::with_capacity(children.len());

    for (index, element) in children.into_iter().enumerate() {
        let key = match &element {
            Element::WidgetElement(element) => {
                ReconcilementKey::new(element.widget.as_any().type_id(), element.key, index)
            }
            Element::ComponentElement(element) => {
                ReconcilementKey::new(element.component.as_any().type_id(), element.key, index)
            }
        };
        new_keys.push(key);
        new_elements.push(Some(element));
    }

    Reconciler::new(old_keys, old_ids, new_keys, new_elements)
}
