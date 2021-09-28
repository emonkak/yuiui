use std::any::{Any, TypeId};
use std::fmt;
use std::mem;
use std::rc::Rc;
use yuiui_support::bit_flags::BitFlags;
use yuiui_support::slot_tree::{CursorMut, NodeId, SlotTree};

use super::event_manager::EventManager;
use super::reconciler::{Patch, Reconciler};
use super::root::Root;
use super::{
    Attributes, BoxedComponent, Children, Command, ComponentElement, Effect, Element, Key, Lifecycle,
    RcWidget, Widget, WidgetElement,
};
use crate::event::{WindowEvent, WindowEventMask};
use crate::geometrics::{BoxConstraints, Point, Rectangle, Size, Viewport};
use crate::graphics::Primitive;

type ComponentIndex = usize;

#[derive(Debug)]
pub struct WidgetStorage<Message> {
    element_tree: SlotTree<ElementNode<Message>>,
    widget_tree: SlotTree<Option<WidgetPod<Message>>>,
    event_manager: EventManager,
    uncommited_changes: Vec<WidgetTreePatch<Message>>,
}

impl<Message> WidgetStorage<Message> {
    pub fn new(element: Element<Message>, viewport: Viewport) -> Self
    where
        Message: 'static,
    {
        let widget = Root::new(viewport).into_rc();
        let element_tree = {
            let element = WidgetElement {
                widget: widget.clone(),
                attributes: Default::default(),
                key: None,
                children: Rc::new(vec![element]),
            };
            let element_node = ElementNode {
                element: Some(element),
                component_stack: Vec::new(),
            };
            SlotTree::new(element_node)
        };
        let widget_tree = {
            let widget = WidgetPod::new(widget);
            SlotTree::new(Some(widget))
        };
        Self {
            element_tree,
            widget_tree,
            event_manager: EventManager::new(),
            uncommited_changes: Vec::new(),
        }
    }

    pub fn render(
        &mut self,
        id: NodeId,
        component_index: ComponentIndex,
        root: NodeId,
    ) -> Option<(NodeId, ComponentIndex)> {
        let mut cursor = self.element_tree.cursor_mut(id);
        let component_stack = &mut cursor.current().data_mut().component_stack;

        if component_index < component_stack.len() {
            let component = &mut component_stack[component_index];
            let is_updated = if let Some(pending_element) = component.pending_element.take() {
                component.update(pending_element)
            } else {
                true
            };

            if is_updated {
                let children = vec![component.render()];
                let reconciler = create_reconciler(&mut cursor, children, component_index);
                for patch in reconciler {
                    self.commit_element_tree(id, patch, true)
                }
            }

            Some((id, component_index + 1))
        } else {
            let element = cursor.current().data_mut().element.as_ref().unwrap();
            let reconciler =
                create_reconciler(&mut cursor, (*element.children).clone(), component_index);
            for patch in reconciler {
                self.commit_element_tree(id, patch, false)
            }

            self.element_tree
                .cursor(id)
                .descendants_from(root)
                .next()
                .map(|(next_id, _)| (next_id, 0))
        }
    }

    pub fn commit(&mut self) -> impl Iterator<Item = Command<Message>> + '_ {
        mem::take(&mut self.uncommited_changes)
            .into_iter()
            .flat_map(move |patch| self.commit_widget_tree(patch))
    }

    pub fn has_uncommited_changes(&self) -> bool {
        !self.uncommited_changes.is_empty()
    }

    pub fn layout(&mut self, id: NodeId) -> NodeId {
        let mut current = id;

        loop {
            let mut cursor = self.widget_tree.cursor_mut(current);
            let mut widget = cursor
                .current()
                .data_mut()
                .take()
                .expect("widget is currently in use elsewhere");

            let box_constraints = widget.box_constraints;
            let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
            let mut context = LayoutContext { storage: self };
            let has_changed = widget.layout(box_constraints, &children, &mut context);

            let mut cursor = self.widget_tree.cursor_mut(current);
            *cursor.current().data_mut() = Some(widget);

            match (has_changed, cursor.current().parent()) {
                (true, Some(parent)) => current = parent,
                _ => break current
            }
        }
    }

    pub fn draw(&mut self, id: NodeId) -> (Primitive, Rectangle) {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let mut widget = cursor
            .current()
            .data_mut()
            .take()
            .expect("widget is currently in use elsewhere");

        let origin = cursor.ancestors().fold(Point::ZERO, |origin, (_, node)| {
            let mut parent = node.data_mut().as_mut().unwrap();
            parent.needs_draw = true;
            origin + parent.position
        });
        let bounds = Rectangle::new(origin + widget.position, widget.size);
        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();

        let mut context = DrawContext {
            storage: self,
            origin: widget.position,
        };
        let primitive = widget.draw(bounds, &children, &mut context);

        let mut cursor = self.widget_tree.cursor_mut(id);
        *cursor.current().data_mut() = Some(widget);

        (primitive, bounds)
    }

    pub fn dispatch(&mut self, event: WindowEvent) {
        let event_mask = event.event_mask();

        if let Some(listeners) = self.event_manager.get_listerners(event_mask) {
            for id in listeners {
                let widget = self.widget_tree
                    .cursor_mut(id)
                    .current()
                    .data_mut()
                    .as_mut()
                    .expect("widget is currently in use elsewhere");
                widget.on_event(&event);
            }
        }
    }

    fn commit_element_tree(
        &mut self,
        parent: NodeId,
        patch: Patch<ElementId, Element<Message>>,
        in_component_rendering: bool,
    ) {
        match patch {
            Patch::Append(Element::WidgetElement(element)) => {
                let mut cursor = self.element_tree.cursor_mut(parent);
                if in_component_rendering {
                    let widget_node = cursor.current().data_mut();
                    widget_node.element = Some(element.clone());
                    self.uncommited_changes.push(WidgetTreePatch::Append(
                        cursor.current().parent().unwrap(),
                        element,
                    ))
                } else {
                    cursor.append_child(ElementNode {
                        element: Some(element.clone()),
                        component_stack: Vec::new(),
                    });
                    self.uncommited_changes
                        .push(WidgetTreePatch::Append(parent, element))
                }
            }
            Patch::Append(Element::ComponentElement(element)) => {
                let mut cursor = self.element_tree.cursor_mut(parent);
                let component = ComponentPod::from_element(element);
                if in_component_rendering {
                    cursor.current().data_mut().component_stack.push(component);
                } else {
                    cursor.append_child(ElementNode {
                        element: None,
                        component_stack: vec![component],
                    });
                }
            }
            Patch::Insert(reference, Element::WidgetElement(element)) => {
                let mut cursor = self.element_tree.cursor_mut(reference.id());
                cursor.insert_before(ElementNode {
                    element: Some(element.clone()),
                    component_stack: Vec::new(),
                });
                self.uncommited_changes
                    .push(WidgetTreePatch::Insert(reference.id(), element))
            }
            Patch::Insert(reference, Element::ComponentElement(element)) => {
                let mut cursor = self.element_tree.cursor_mut(reference.id());
                let component = ComponentPod::from_element(element);
                cursor.insert_before(ElementNode {
                    element: None,
                    component_stack: vec![component],
                });
            }
            Patch::Update(ElementId::Widget(id), Element::WidgetElement(element)) => {
                let mut cursor = self.element_tree.cursor_mut(id);
                cursor.current().data_mut().element = Some(element.clone());
                self.uncommited_changes
                    .push(WidgetTreePatch::Update(id, element))
            }
            Patch::Update(
                ElementId::Component(id, component_index),
                Element::ComponentElement(element),
            ) => {
                let mut cursor = self.element_tree.cursor_mut(id);
                let component = &mut cursor.current().data_mut().component_stack[component_index];
                component.pending_element = Some(element);
            }
            Patch::UpdateAndMove(
                ElementId::Widget(id),
                reference,
                Element::WidgetElement(element),
            ) => {
                let mut cursor = self.element_tree.cursor_mut(id);
                cursor.current().data_mut().element = Some(element.clone());
                cursor.move_before(reference.id());
                self.uncommited_changes.push(WidgetTreePatch::UpdateAndMove(
                    id,
                    reference.id(),
                    element,
                ))
            }
            Patch::UpdateAndMove(
                ElementId::Component(id, component_index),
                reference,
                Element::ComponentElement(element),
            ) => {
                let mut cursor = self.element_tree.cursor_mut(id);
                let component = &mut cursor.current().data_mut().component_stack[component_index];
                component.pending_element = Some(element);
                cursor.move_before(reference.id());
            }
            Patch::Remove(ElementId::Widget(id)) => {
                let cursor = self.element_tree.cursor_mut(id);
                let _ = cursor.drain_subtree();
                self.uncommited_changes.push(WidgetTreePatch::Remove(id));
            }
            Patch::Remove(ElementId::Component(id, component_index)) => {
                let mut cursor = self.element_tree.cursor_mut(id);
                let mut widget_node = cursor.current().data_mut();
                let _ = widget_node.component_stack.drain(component_index..);
                if component_index > 0 {
                    widget_node.element = None;
                    let _ = cursor.drain_descendants();
                    self.uncommited_changes
                        .push(WidgetTreePatch::RemoveChildren(id));
                } else {
                    let _ = cursor.drain_subtree();
                    self.uncommited_changes.push(WidgetTreePatch::Remove(id));
                }
            }
            _ => unreachable!("element kind mismatch"),
        }
    }

    fn commit_widget_tree(&mut self, patch: WidgetTreePatch<Message>) -> Vec<Command<Message>> {
        match patch {
            WidgetTreePatch::Append(parent, element) => {
                let id = self.widget_tree.next_node_id();
                let mut cursor = self.widget_tree.cursor_mut(parent);
                let mut widget = WidgetPod::from_element(element);
                let event_manager = &mut self.event_manager;
                let commands = widget
                    .on_lifecycle(Lifecycle::OnMount)
                    .and_then(|effect| process_effect(&mut widget, id, effect, event_manager))
                    .into_iter()
                    .collect();
                cursor.append_child(Some(widget));
                commands
            }
            WidgetTreePatch::Insert(reference, element) => {
                let id = self.widget_tree.next_node_id();
                let mut cursor = self.widget_tree.cursor_mut(reference);
                let mut widget = WidgetPod::from_element(element);
                let event_manager = &mut self.event_manager;
                let commands = widget
                    .on_lifecycle(Lifecycle::OnMount)
                    .and_then(|effect| process_effect(&mut widget, id, effect, event_manager))
                    .into_iter()
                    .collect();
                cursor.insert_before(Some(widget));
                commands
            }
            WidgetTreePatch::Update(id, element) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let widget = cursor.current().data_mut().as_mut().unwrap();
                let event_manager = &mut self.event_manager;
                let commands = widget
                    .update(element)
                    .and_then(|effect| process_effect(widget, id, effect, event_manager))
                    .into_iter()
                    .collect();
                commands
            }
            WidgetTreePatch::UpdateAndMove(id, reference, element) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let widget = cursor.current().data_mut().as_mut().unwrap();
                let event_manager = &mut self.event_manager;
                let commands = widget
                    .update(element)
                    .and_then(|effect| process_effect(widget, id, effect, event_manager))
                    .into_iter()
                    .collect();
                cursor.move_before(reference);
                commands
            }
            WidgetTreePatch::Remove(id) => {
                let cursor = self.widget_tree.cursor_mut(id);
                let event_manager = &mut self.event_manager;
                let commands = cursor
                    .drain_subtree()
                    .flat_map(|(id, node)| {
                        let mut widget = node.into_data().unwrap();
                        event_manager.remove_listener(id, widget.event_mask);
                        widget
                            .on_lifecycle(Lifecycle::OnUnmount)
                            .and_then(|effect| process_effect(&mut widget, id, effect, event_manager))
                    })
                    .collect();
                commands
            }
            WidgetTreePatch::RemoveChildren(id) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let event_manager = &mut self.event_manager;
                let commands = cursor
                    .drain_descendants()
                    .flat_map(|(id, node)| {
                        let mut widget = node.into_data().unwrap();
                        event_manager.remove_listener(id, widget.event_mask);
                        widget
                            .on_lifecycle(Lifecycle::OnUnmount)
                            .and_then(|effect| process_effect(&mut widget, id, effect, event_manager))
                    })
                    .collect();
                commands
            }
        }
    }

    fn layout_child(&mut self, id: NodeId, box_constraints: BoxConstraints) -> Size {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let mut widget = cursor
            .current()
            .data_mut()
            .take()
            .expect("widget is currently in use elsewhere");

        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
        let mut context = LayoutContext { storage: self };

        widget.layout(box_constraints, &children, &mut context);
        widget.box_constraints = box_constraints;
        let size = widget.size;

        let mut cursor = self.widget_tree.cursor_mut(id);
        *cursor.current().data_mut() = Some(widget);

        size
    }

    fn draw_child(&mut self, id: NodeId, origin: Point) -> Primitive {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let mut widget = cursor
            .current()
            .data_mut()
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
        *cursor.current().data_mut() = Some(widget);

        primitive
    }

    fn get_widget(&self, id: NodeId) -> &WidgetPod<Message> {
        self.widget_tree
            .cursor(id)
            .current()
            .data()
            .as_ref()
            .expect("widget is currently in use elsewhere")
    }

    fn get_widget_mut(&mut self, id: NodeId) -> &mut WidgetPod<Message> {
        self.widget_tree
            .cursor_mut(id)
            .current()
            .data_mut()
            .as_mut()
            .expect("widget is currently in use elsewhere")
    }
}

impl<Message: fmt::Debug> fmt::Display for WidgetStorage<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.element_tree.fmt(f)
    }
}

#[derive(Debug)]
enum WidgetTreePatch<Message> {
    Append(NodeId, WidgetElement<Message>),
    Insert(NodeId, WidgetElement<Message>),
    Update(NodeId, WidgetElement<Message>),
    UpdateAndMove(NodeId, NodeId, WidgetElement<Message>),
    Remove(NodeId),
    RemoveChildren(NodeId),
}

#[derive(Debug)]
struct ElementNode<Message> {
    element: Option<WidgetElement<Message>>,
    component_stack: Vec<ComponentPod<Message>>,
}

#[derive(Debug)]
struct WidgetPod<Message> {
    widget: RcWidget<Message>,
    attributes: Rc<Attributes>,
    state: Box<dyn Any>,
    event_mask: BitFlags<WindowEventMask>,
    box_constraints: BoxConstraints,
    position: Point,
    size: Size,
    draw_cache: Option<Primitive>,
    needs_layout: bool,
    needs_draw: bool,
}

impl<Message> WidgetPod<Message> {
    fn new(widget: RcWidget<Message>) -> Self {
        let state = widget.initial_state();
        Self {
            widget,
            attributes: Default::default(),
            state,
            event_mask: BitFlags::empty(),
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
            state,
            event_mask: BitFlags::empty(),
            box_constraints: BoxConstraints::LOOSE,
            position: Point::ZERO,
            size: Size::ZERO,
            draw_cache: None,
            needs_layout: true,
            needs_draw: true,
        }
    }

    fn update(&mut self, element: WidgetElement<Message>) -> Option<Effect<Message>> {
        let should_update = !element.children.is_empty()
            || &*self.attributes != &*element.attributes
            || self
                .widget
                .should_update(element.widget.as_any(), &self.state);
        let old_widget = mem::replace(&mut self.widget, element.widget);

        self.attributes = element.attributes;
        self.needs_layout = should_update;
        self.needs_draw = should_update;

        if should_update {
            self.on_lifecycle(Lifecycle::OnUpdate(old_widget.as_any()))
        } else {
            None
        }
    }

    fn on_event(&mut self, event: &WindowEvent) -> Option<Effect<Message>> {
        self.widget.on_event(event, &mut self.state)
    }

    fn on_lifecycle(&mut self, lifecycle: Lifecycle<&dyn Any>) -> Option<Effect<Message>> {
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
}

#[derive(Debug)]
struct ComponentPod<Message> {
    component: BoxedComponent<Message>,
    attributes: Rc<Attributes>,
    children: Children<Message>,
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
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl TypedKey {
    fn new(type_id: TypeId, key: Option<Key>, index: usize) -> Self {
        match key {
            Some(key) => Self::Keyed(type_id, key),
            None => Self::Indexed(type_id, index),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ElementId {
    Widget(NodeId),
    Component(NodeId, usize),
}

impl ElementId {
    fn id(&self) -> NodeId {
        match self {
            Self::Widget(id) => *id,
            Self::Component(id, _) => *id,
        }
    }
}

fn create_reconciler<Message>(
    cursor: &mut CursorMut<ElementNode<Message>>,
    children: Vec<Element<Message>>,
    component_index: ComponentIndex,
) -> Reconciler<TypedKey, ElementId, Element<Message>> {
    let mut old_keys: Vec<TypedKey> = Vec::new();
    let mut old_ids: Vec<Option<ElementId>> = Vec::new();

    for (index, (child_id, child)) in cursor.children().enumerate() {
        let child_node = child.data();
        let (key, id) = if component_index < child_node.component_stack.len() {
            let component = &child_node.component_stack[component_index];
            let type_id = component.as_any().type_id();
            let key = TypedKey::new(type_id, component.key, index);
            let id = ElementId::Component(child_id, component_index);
            (key, id)
        } else {
            let element = child_node.element.as_ref().unwrap();
            let type_id = element.widget.as_any().type_id();
            let key = TypedKey::new(type_id, element.key, index);
            let id = ElementId::Widget(child_id);
            (key, id)
        };
        old_keys.push(key);
        old_ids.push(Some(id));
    }

    let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
    let mut new_elements: Vec<Option<Element<Message>>> = Vec::with_capacity(children.len());

    for (index, element) in children.into_iter().enumerate() {
        let key = match &element {
            Element::WidgetElement(element) => {
                TypedKey::new(element.widget.as_any().type_id(), element.key, index)
            }
            Element::ComponentElement(element) => {
                TypedKey::new(element.component.as_any().type_id(), element.key, index)
            }
        };
        new_keys.push(key);
        new_elements.push(Some(element));
    }

    Reconciler::new(old_keys, old_ids, new_keys, new_elements)
}

fn process_effect<Message>(
    widget: &mut WidgetPod<Message>,
    id: NodeId,
    effect: Effect<Message>,
    event_manager: &mut EventManager,
) -> Option<Command<Message>> {
    match effect {
        Effect::AddListener(event_mask) => {
            let new_events = event_mask & (event_mask ^ widget.event_mask);
            event_manager.add_listener(id, new_events);
            widget.event_mask |= event_mask;
            None
        }
        Effect::RemoveListener(event_mask) => {
            let removed_events = event_mask & widget.event_mask;
            event_manager.remove_listener(id, removed_events);
            widget.event_mask ^= event_mask;
            None
        }
        Effect::Command(command) => Some(command)
    }
}
