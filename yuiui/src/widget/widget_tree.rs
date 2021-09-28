use std::any::Any;
use std::collections::VecDeque;
use std::fmt;
use std::mem;
use std::rc::Rc;
use yuiui_support::bit_flags::BitFlags;
use yuiui_support::slot_tree::{NodeId, SlotTree};

use super::event_manager::EventManager;
use super::{Attributes, Command, Effect, Lifecycle, RcWidget, UnitOfWork, WidgetElement};
use crate::event::{WindowEvent, WindowEventMask};
use crate::geometrics::{BoxConstraints, Point, Rectangle, Size, Viewport};
use crate::graphics::Primitive;

#[derive(Debug)]
pub struct WidgetTree<Message> {
    tree: SlotTree<Option<WidgetPod<Message>>>,
    event_manager: EventManager,
}

impl<Message> WidgetTree<Message> {
    pub fn new(root_widget: RcWidget<Message>) -> Self {
        Self {
            tree: SlotTree::new(Some(WidgetPod::new(root_widget))),
            event_manager: EventManager::new(),
        }
    }

    pub fn commit<Handler>(&mut self, unit_of_work: UnitOfWork<Message>, command_handler: Handler)
    where
        Handler: Fn(Command<Message>),
    {
        match unit_of_work {
            UnitOfWork::Append(parent, element) => {
                let id = self.tree.next_node_id();
                let mut cursor = self.tree.cursor_mut(parent);
                let mut widget = WidgetPod::from_element(element);
                let effect = widget.on_lifecycle(Lifecycle::OnMount);
                process_effect(
                    effect,
                    id,
                    &mut widget,
                    &command_handler,
                    &mut self.event_manager,
                );
                cursor.append_child(Some(widget));
            }
            UnitOfWork::Insert(reference, element) => {
                let id = self.tree.next_node_id();
                let mut cursor = self.tree.cursor_mut(reference);
                let mut widget = WidgetPod::from_element(element);
                let effect = widget.on_lifecycle(Lifecycle::OnMount);
                process_effect(
                    effect,
                    id,
                    &mut widget,
                    &command_handler,
                    &mut self.event_manager,
                );
                cursor.insert_before(Some(widget));
            }
            UnitOfWork::Update(id, element) => {
                let mut cursor = self.tree.cursor_mut(id);
                let widget = cursor.current().data_mut().as_mut().unwrap();
                let effect = widget.update(element);
                process_effect(
                    effect,
                    id,
                    widget,
                    &command_handler,
                    &mut self.event_manager,
                );
            }
            UnitOfWork::UpdateAndMove(id, reference, element) => {
                let mut cursor = self.tree.cursor_mut(id);
                let widget = cursor.current().data_mut().as_mut().unwrap();
                let effect = widget.update(element);
                process_effect(
                    effect,
                    id,
                    widget,
                    &command_handler,
                    &mut self.event_manager,
                );
                cursor.move_before(reference);
            }
            UnitOfWork::Remove(id) => {
                let cursor = self.tree.cursor_mut(id);
                for (id, node) in cursor.drain_subtree() {
                    let mut widget = node.into_data().unwrap();
                    self.event_manager.remove_listener(id, widget.event_mask);
                    let effect = widget.on_lifecycle(Lifecycle::OnUnmount);
                    process_effect(
                        effect,
                        id,
                        &mut widget,
                        &command_handler,
                        &mut self.event_manager,
                    );
                }
            }
            UnitOfWork::RemoveChildren(id) => {
                let mut cursor = self.tree.cursor_mut(id);
                for (id, node) in cursor.drain_descendants() {
                    let mut widget = node.into_data().unwrap();
                    self.event_manager.remove_listener(id, widget.event_mask);
                    let effect = widget.on_lifecycle(Lifecycle::OnUnmount);
                    process_effect(
                        effect,
                        id,
                        &mut widget,
                        &command_handler,
                        &mut self.event_manager,
                    );
                }
            }
        }
    }

    pub fn dispatch<Handler>(&mut self, event: &WindowEvent, command_handler: Handler)
    where
        Handler: Fn(Command<Message>),
    {
        let event_mask = event.event_mask();

        for id in self.event_manager.get_listerners(event_mask) {
            let widget = self
                .tree
                .cursor_mut(id)
                .current()
                .data_mut()
                .as_mut()
                .expect("widget is currently in use elsewhere");
            let effect = widget.on_event(event);
            process_effect(
                effect,
                id,
                widget,
                &command_handler,
                &mut self.event_manager,
            );
        }
    }

    pub fn layout(&mut self, id: NodeId, viewport: &Viewport) -> NodeId {
        let mut current = id;

        loop {
            let mut cursor = self.tree.cursor_mut(current);
            let mut widget = cursor
                .current()
                .data_mut()
                .take()
                .expect("widget is currently in use elsewhere");

            let box_constraints = if id.is_root() {
                BoxConstraints::tight(viewport.logical_size())
            } else {
                widget.box_constraints
            };
            let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
            let mut context = LayoutContext { storage: self };
            let has_changed = widget.layout(box_constraints, &children, &mut context);

            let mut cursor = self.tree.cursor_mut(current);
            *cursor.current().data_mut() = Some(widget);

            match (has_changed, cursor.current().parent()) {
                (true, Some(parent)) => current = parent,
                _ => break current,
            }
        }
    }

    pub fn draw(&mut self, id: NodeId) -> (Primitive, Rectangle) {
        let mut cursor = self.tree.cursor_mut(id);
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

        let mut cursor = self.tree.cursor_mut(id);
        *cursor.current().data_mut() = Some(widget);

        (primitive, bounds)
    }

    fn layout_child(&mut self, id: NodeId, box_constraints: BoxConstraints) -> Size {
        let mut cursor = self.tree.cursor_mut(id);
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

        let mut cursor = self.tree.cursor_mut(id);
        *cursor.current().data_mut() = Some(widget);

        size
    }

    fn draw_child(&mut self, id: NodeId, origin: Point) -> Primitive {
        let mut cursor = self.tree.cursor_mut(id);
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

        let mut cursor = self.tree.cursor_mut(id);
        *cursor.current().data_mut() = Some(widget);

        primitive
    }

    fn get_widget(&self, id: NodeId) -> &WidgetPod<Message> {
        self.tree
            .cursor(id)
            .current()
            .data()
            .as_ref()
            .expect("widget is currently in use elsewhere")
    }

    fn get_widget_mut(&mut self, id: NodeId) -> &mut WidgetPod<Message> {
        self.tree
            .cursor_mut(id)
            .current()
            .data_mut()
            .as_mut()
            .expect("widget is currently in use elsewhere")
    }
}

impl<Message: fmt::Debug> fmt::Display for WidgetTree<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.fmt(f)
    }
}

#[derive(Debug)]
pub struct WidgetPod<Message> {
    widget: RcWidget<Message>,
    attributes: Rc<Attributes>,
    state: Box<dyn Any>,
    event_mask: BitFlags<WindowEventMask>,
    position: Point,
    size: Size,
    box_constraints: BoxConstraints,
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
            position: Point::ZERO,
            size: Size::ZERO,
            box_constraints: BoxConstraints::LOOSE,
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
            position: Point::ZERO,
            size: Size::ZERO,
            box_constraints: BoxConstraints::LOOSE,
            draw_cache: None,
            needs_layout: true,
            needs_draw: true,
        }
    }

    fn update(&mut self, element: WidgetElement<Message>) -> Effect<Message> {
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
            Effect::None
        }
    }

    fn on_event(&mut self, event: &WindowEvent) -> Effect<Message> {
        self.widget.on_event(event, &mut self.state)
    }

    fn on_lifecycle(&mut self, lifecycle: Lifecycle<&dyn Any>) -> Effect<Message> {
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
pub struct LayoutContext<'a, Message> {
    storage: &'a mut WidgetTree<Message>,
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
    storage: &'a mut WidgetTree<Message>,
    origin: Point,
}

impl<'a, Message> DrawContext<'a, Message> {
    pub fn draw_child(&mut self, id: NodeId) -> Primitive {
        self.storage.draw_child(id, self.origin)
    }
}

fn process_effect<Message, Handler>(
    effect: Effect<Message>,
    id: NodeId,
    widget: &mut WidgetPod<Message>,
    command_handler: &Handler,
    event_manager: &mut EventManager,
) where
    Handler: Fn(Command<Message>),
{
    let mut queue = VecDeque::new();
    let mut current = effect;

    loop {
        match current {
            Effect::None => {}
            Effect::AddListener(event_mask) => {
                let new_events = event_mask & (event_mask ^ widget.event_mask);
                event_manager.add_listener(id, new_events);
                widget.event_mask |= event_mask;
            }
            Effect::RemoveListener(event_mask) => {
                let removed_events = event_mask & widget.event_mask;
                event_manager.remove_listener(id, removed_events);
                widget.event_mask ^= event_mask;
            }
            Effect::Command(command) => command_handler(command),
            Effect::Batch(effects) => queue.extend(effects),
        }

        if let Some(next) = queue.pop_front() {
            current = next;
        } else {
            break;
        }
    }
}
