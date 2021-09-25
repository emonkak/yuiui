use std::any::{Any, TypeId};
use std::fmt;
use std::mem;
use std::rc::Rc;
use yuiui_support::slot_tree::{CursorMut, NodeId, SlotTree};

use super::reconciler::{ReconcileResult, Reconciler};
use super::root::Root;
use super::{
    Attributes, BoxedComponent, BoxedWidget, ComponentElement, Element, Key, Widget, WidgetElement,
};
use crate::geometrics::{BoxConstraints, Point, Rectangle, Size, Viewport};
use crate::graphics::Primitive;

#[derive(Debug)]
pub struct WidgetStorage {
    widget_tree: SlotTree<WidgetNode>,
    version: usize,
}

impl WidgetStorage {
    pub fn new(element: Element, viewport: Viewport) -> Self {
        let root = {
            let widget = Root::new(viewport);
            let element = WidgetElement {
                widget: widget.into_boxed(),
                attributes: Rc::new(Attributes::new()),
                children: vec![element],
                key: None,
            };
            let instance = WidgetPod::new(element, 0);
            WidgetNode {
                instance: Some(instance),
                component_stack: Vec::new(),
            }
        };
        Self {
            widget_tree: SlotTree::new(root),
            version: 0,
        }
    }

    pub fn get_widget(&self, id: NodeId) -> &WidgetPod {
        self.widget_tree
            .cursor(id)
            .current()
            .data()
            .instance
            .as_ref()
            .expect("widget is currently in use elsewhere")
    }

    pub fn get_widget_mut(&mut self, id: NodeId) -> &mut WidgetPod {
        self.widget_tree
            .cursor_mut(id)
            .current()
            .data_mut()
            .instance
            .as_mut()
            .expect("widget is currently in use elsewhere")
    }

    pub fn try_get_widget(&self, tag: WidgetTag) -> Option<&WidgetPod> {
        self.widget_tree
            .try_cursor(tag.node_id)
            .map(|cursor| cursor.current().data().borrow())
            .filter(|instance| instance.version == tag.version)
    }

    pub fn try_get_widget_mut(&mut self, tag: WidgetTag) -> Option<&mut WidgetPod> {
        self.widget_tree
            .try_cursor_mut(tag.node_id)
            .map(|mut cursor| cursor.current().data_mut().borrow_mut())
            .filter(|instance| instance.version == tag.version)
    }

    pub fn try_get_component(&self, tag: ComponentTag) -> Option<&ComponentPod> {
        self.widget_tree
            .try_cursor(tag.node_id)
            .and_then(|cursor| {
                cursor
                    .current()
                    .data()
                    .component_stack
                    .get(tag.component_index)
            })
            .filter(|instance| instance.version == tag.component_version)
    }

    pub fn try_get_component_mut(&mut self, tag: ComponentTag) -> Option<&mut ComponentPod> {
        self.widget_tree
            .try_cursor_mut(tag.node_id)
            .and_then(|mut cursor| {
                cursor
                    .current()
                    .data_mut()
                    .component_stack
                    .get_mut(tag.component_index)
            })
            .filter(|instance| instance.version == tag.component_version)
    }

    pub fn render(
        &mut self,
        id: NodeId,
        component_index: usize,
        root: NodeId,
    ) -> Option<(NodeId, usize)> {
        let mut cursor = self.widget_tree.cursor_mut(id);
        let component_stack = &mut cursor.current().data_mut().component_stack;

        if component_index < component_stack.len() {
            let instance = &mut component_stack[component_index];
            let should_update = if let Some(pending_element) = instance.pending_element.take() {
                instance.update(pending_element)
            } else {
                true
            };

            if should_update {
                let children = vec![instance.render()];
                let reconciler = create_reconciler(&mut cursor, children, component_index);
                for result in reconciler {
                    self.update_child(id, result, true)
                }
            }

            Some((id, component_index + 1))
        } else {
            let children =
                mem::take(&mut cursor.current().data_mut().borrow_mut().pending_children);
            if !children.is_empty() {
                let reconciler = create_reconciler(&mut cursor, children, component_index);
                for result in reconciler {
                    self.update_child(id, result, false)
                }
            }

            self.widget_tree
                .cursor(id)
                .descendants_from(root)
                .next()
                .map(|(next_id, _)| (next_id, 0))
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
        result: ReconcileResult<ReconcilementId, Element>,
        in_component_rendering: bool,
    ) {
        match result {
            ReconcileResult::Append(element) => {
                let mut cursor = self.widget_tree.cursor_mut(parent);
                match element {
                    Element::WidgetElement(element) => {
                        let instance = WidgetPod::new(element, self.version);
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
                        let instance = ComponentPod::new(element, self.version);
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
                let mut cursor = self.widget_tree.cursor_mut(reference.id());
                match element {
                    Element::WidgetElement(element) => {
                        let instance = WidgetPod::new(element, self.version);
                        cursor.insert_before(WidgetNode {
                            instance: Some(instance),
                            component_stack: Vec::new(),
                        });
                    }
                    Element::ComponentElement(element) => {
                        let instance = ComponentPod::new(element, self.version);
                        cursor.insert_before(WidgetNode {
                            instance: None,
                            component_stack: vec![instance],
                        });
                    }
                }
            }
            ReconcileResult::Update(ReconcilementId::Widget(id), element) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                match element {
                    Element::WidgetElement(element) => {
                        cursor.current().data_mut().borrow_mut().update(element)
                    }
                    _ => unreachable!("element type mismatch"),
                };
            }
            ReconcileResult::Update(ReconcilementId::Component(id, component_index), element) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let instance = &mut cursor.current().data_mut().component_stack[component_index];
                match element {
                    Element::ComponentElement(element) => instance.pending_element = Some(element),
                    _ => unreachable!("element type mismatch"),
                }
            }
            ReconcileResult::UpdateAndMove(
                ReconcilementId::Widget(id),
                ReconcilementId::Widget(reference_id),
                element,
            ) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let _ = match element {
                    Element::WidgetElement(element) => {
                        cursor.current().data_mut().borrow_mut().update(element)
                    }
                    _ => unreachable!("element type mismatch"),
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
                    _ => unreachable!("element type mismatch"),
                }
                cursor.move_before(reference_id);
            }
            ReconcileResult::Remove(ReconcilementId::Widget(id)) => {
                let _ = self.widget_tree.cursor_mut(id).drain_subtree();
                self.version += 1;
            }
            ReconcileResult::Remove(ReconcilementId::Component(id, component_index)) => {
                let mut cursor = self.widget_tree.cursor_mut(id);
                let mut widget_node = cursor.current().data_mut();
                let _ = widget_node.component_stack.drain(component_index..);
                widget_node.instance = None;
                let _ = self.widget_tree.cursor_mut(id).drain_descendants();
                self.version += 1;
            }
            _ => unreachable!("element type mismatch"),
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
}

impl fmt::Display for WidgetStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.widget_tree.fmt(f)
    }
}

#[derive(Debug)]
struct WidgetNode {
    instance: Option<WidgetPod>,
    component_stack: Vec<ComponentPod>,
}

impl WidgetNode {
    fn borrow(&self) -> &WidgetPod {
        self.instance
            .as_ref()
            .expect("widget is currently in use elsewhere")
    }

    fn borrow_mut(&mut self) -> &mut WidgetPod {
        self.instance
            .as_mut()
            .expect("widget is currently in use elsewhere")
    }
}

#[derive(Debug)]
pub struct WidgetPod {
    widget: BoxedWidget,
    attributes: Rc<Attributes>,
    pending_children: Vec<Element>,
    key: Option<Key>,
    state: Box<dyn Any>,
    box_constraints: BoxConstraints,
    position: Point,
    size: Size,
    draw_cache: Option<Primitive>,
    needs_layout: bool,
    needs_draw: bool,
    version: usize,
}

impl WidgetPod {
    fn new(element: WidgetElement, version: usize) -> Self {
        let state = element.widget.initial_state();
        Self {
            widget: element.widget,
            attributes: element.attributes,
            pending_children: element.children,
            key: element.key,
            state,
            box_constraints: BoxConstraints::LOOSE,
            position: Point::ZERO,
            size: Size::ZERO,
            draw_cache: None,
            needs_layout: true,
            needs_draw: true,
            version,
        }
    }

    fn update(&mut self, element: WidgetElement) -> bool {
        let should_update = !element.children.is_empty()
            || self.widget.should_update(
                element.widget.as_any(),
                &*self.attributes,
                &*element.attributes,
                &self.state,
            );

        self.widget = element.widget;
        self.attributes = element.attributes;
        self.pending_children = element.children;
        self.needs_layout = should_update;
        self.needs_draw = should_update;

        should_update
    }

    fn layout(
        &mut self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext,
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
        context: &mut DrawContext,
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
pub struct ComponentPod {
    component: BoxedComponent,
    attributes: Rc<Attributes>,
    children: Vec<Element>,
    state: Box<dyn Any>,
    key: Option<Key>,
    pending_element: Option<ComponentElement>,
    version: usize,
}

impl ComponentPod {
    fn new(element: ComponentElement, version: usize) -> Self {
        let state = element.component.initial_state();
        Self {
            component: element.component,
            attributes: element.attributes,
            children: element.children,
            key: element.key,
            state,
            pending_element: None,
            version,
        }
    }

    fn update(&mut self, element: ComponentElement) -> bool {
        let should_update = self.component.should_update(
            element.component.as_any(),
            &*self.attributes,
            &*element.attributes,
            &self.children,
            &element.children,
            &self.state,
        );

        self.component = element.component;
        self.attributes = element.attributes;
        self.children = element.children;

        should_update
    }

    fn render(&self) -> Element {
        self.component.render(&self.children, &self.state)
    }

    fn as_any(&self) -> &dyn Any {
        self.component.as_any()
    }
}

#[derive(Debug)]
pub struct LayoutContext<'a> {
    storage: &'a mut WidgetStorage,
}

impl<'a> LayoutContext<'a> {
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
pub struct DrawContext<'a> {
    storage: &'a mut WidgetStorage,
    origin: Point,
}

impl<'a> DrawContext<'a> {
    pub fn draw_child(&mut self, id: NodeId) -> Primitive {
        self.storage.draw_child(id, self.origin)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WidgetTag {
    node_id: NodeId,
    version: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentTag {
    node_id: NodeId,
    component_index: usize,
    component_version: usize,
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
    fn id(&self) -> NodeId {
        match self {
            Self::Widget(id) => *id,
            Self::Component(id, _) => *id,
        }
    }
}

fn create_reconciler(
    cursor: &mut CursorMut<WidgetNode>,
    children: Vec<Element>,
    component_index: usize,
) -> Reconciler<ReconcilementKey, ReconcilementId, Element> {
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
    let mut new_elements: Vec<Option<Element>> = Vec::with_capacity(children.len());

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
