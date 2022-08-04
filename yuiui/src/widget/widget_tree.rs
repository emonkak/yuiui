use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::fmt;
use yuiui_support::bit_flags::BitFlags;
use yuiui_support::slot_tree::{NodeId, SlotTree};

use super::event_manager::EventManager;
use super::{Command, Effect, Event, EventMask, Lifecycle, RcWidget, UnitOfWork};
use crate::geometrics::{BoxConstraints, Point, Rect, Size, Viewport};
use crate::graphics::Primitive;
use crate::style::LayoutStyle;

#[derive(Debug)]
pub struct WidgetTree<State, Message> {
    tree: SlotTree<WidgetNode<State, Message>>,
    event_manager: EventManager<NodeId>,
}

impl<State, Message> WidgetTree<State, Message> {
    pub fn new(root_widget: RcWidget<State, Message>) -> Self {
        let widget = WidgetPod::new(root_widget);
        Self {
            tree: SlotTree::new(WidgetNode::from(widget)),
            event_manager: EventManager::new(),
        }
    }

    pub fn commit<Handler>(&mut self, unit_of_work: UnitOfWork<State, Message>, handler: &Handler)
    where
        Handler: Fn(Command<Message>, NodeId),
    {
        match unit_of_work {
            UnitOfWork::Append(parent, widget) => {
                let id = self.tree.next_node_id();
                let mut widget = WidgetPod::new(widget);
                let effect = widget.on_lifecycle(Lifecycle::Mounted);
                process_effect(effect, id, &mut widget, &handler, &mut self.event_manager);
                let mut cursor = self.tree.cursor_mut(parent);
                cursor.append_child(WidgetNode::from(widget));
            }
            UnitOfWork::Insert(reference, widget) => {
                let id = self.tree.next_node_id();
                let mut widget = WidgetPod::new(widget);
                let effect = widget.on_lifecycle(Lifecycle::Mounted);
                process_effect(effect, id, &mut widget, &handler, &mut self.event_manager);
                let mut cursor = self.tree.cursor_mut(reference);
                cursor.insert_before(WidgetNode::from(widget));
            }
            UnitOfWork::Replace(id, widget) => {
                let mut widget = WidgetPod::new(widget);
                let effect = widget.on_lifecycle(Lifecycle::Mounted);
                process_effect(effect, id, &mut widget, &handler, &mut self.event_manager);
                let mut cursor = self.tree.cursor_mut(id);
                *cursor.node().data_mut() = WidgetNode::from(widget);
            }
            UnitOfWork::Update(id, new_widget) => {
                let cursor = self.tree.cursor(id);
                let mut widget = cursor.node().data().borrow_mut();
                let effect = widget.update(new_widget);
                process_effect(effect, id, &mut widget, &handler, &mut self.event_manager);
            }
            UnitOfWork::Move(id, reference) => {
                let mut cursor = self.tree.cursor_mut(id);
                cursor.move_before(reference);
            }
            UnitOfWork::Remove(id) => {
                let cursor = self.tree.cursor_mut(id);
                for (id, node) in cursor.drain_subtree() {
                    let mut widget = node.into_data().into_inner();
                    self.event_manager.remove_listener(id, widget.event_mask);
                    let effect = widget.on_lifecycle(Lifecycle::Unmounted);
                    process_effect(effect, id, &mut widget, &handler, &mut self.event_manager);
                }
            }
            UnitOfWork::RemoveChildren(id) => {
                let mut cursor = self.tree.cursor_mut(id);
                for (id, node) in cursor.drain_descendants() {
                    let mut widget = node.into_data().into_inner();
                    self.event_manager.remove_listener(id, widget.event_mask);
                    let effect = widget.on_lifecycle(Lifecycle::Unmounted);
                    process_effect(effect, id, &mut widget, &handler, &mut self.event_manager);
                }
            }
        }
    }

    pub fn dispatch<Handler>(&mut self, event: Event<State>, handler: Handler)
    where
        Handler: Fn(Command<Message>, NodeId),
    {
        let event_mask = event.event_mask();
        let listeners = self.event_manager.get_listerners(event_mask);

        for id in listeners {
            let mut widget = self.tree.cursor(id).node().data().borrow_mut();
            let effect = widget.on_event(event);
            process_effect(effect, id, &mut widget, &handler, &mut self.event_manager);
        }
    }

    pub fn layout(&self, id: NodeId, viewport: &Viewport) -> NodeId {
        let mut current = id;

        loop {
            let cursor = self.tree.cursor(current);
            let mut widget = cursor.node().data().borrow_mut();

            let box_constraints = if id.is_root() {
                BoxConstraints::tight(viewport.logical_size())
            } else {
                widget.box_constraints
            };
            let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
            let mut context = LayoutContext { widget_tree: self };
            let has_changed = widget.layout(box_constraints, &children, &mut context);

            match (has_changed, cursor.node().parent()) {
                (true, Some(parent)) => current = parent,
                _ => break current,
            }
        }
    }

    pub fn draw(&self, id: NodeId) -> (Primitive, Rect) {
        let cursor = self.tree.cursor(id);
        let mut widget = cursor.node().data().borrow_mut();

        let origin = cursor.ancestors().fold(Point::ZERO, |origin, (_, node)| {
            let mut parent = node.data().borrow_mut();
            parent.needs_draw = true;
            origin + parent.position
        });
        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();

        let mut context = DrawContext {
            widget_tree: self,
            origin: origin + widget.position,
        };
        widget.draw(origin, &children, &mut context)
    }

    fn layout_child(&self, id: NodeId, box_constraints: BoxConstraints) -> Size {
        let cursor = self.tree.cursor(id);
        let mut widget = cursor.node().data().borrow_mut();

        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
        let mut context = LayoutContext { widget_tree: self };

        widget.layout(box_constraints, &children, &mut context);
        widget.box_constraints = box_constraints;
        let size = widget.size;

        size
    }

    fn draw_child(&self, id: NodeId, origin: Point) -> Primitive {
        let cursor = self.tree.cursor(id);
        let mut widget = cursor.node().data().borrow_mut();

        let children = cursor.children().map(|(id, _)| id).collect::<Vec<_>>();
        let mut context = DrawContext {
            widget_tree: self,
            origin: origin + widget.position,
        };

        let (primitive, _) = widget.draw(origin, &children, &mut context);
        primitive
    }

    fn get(&self, id: NodeId) -> Ref<WidgetPod<State, Message>> {
        self.tree.cursor(id).node().data().borrow()
    }

    fn get_mut(&self, id: NodeId) -> RefMut<WidgetPod<State, Message>> {
        self.tree.cursor(id).node().data().borrow_mut()
    }
}

impl<State, Message> fmt::Display for WidgetTree<State, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.fmt(f)
    }
}

#[derive(Debug)]
struct WidgetNode<State, Message> {
    widget: RefCell<WidgetPod<State, Message>>,
}

impl<State, Message> WidgetNode<State, Message> {
    fn into_inner(self) -> WidgetPod<State, Message> {
        self.widget.into_inner()
    }

    fn borrow(&self) -> Ref<WidgetPod<State, Message>> {
        self.widget.borrow()
    }

    fn borrow_mut(&self) -> RefMut<WidgetPod<State, Message>> {
        self.widget.borrow_mut()
    }
}

impl<State, Message> From<WidgetPod<State, Message>> for WidgetNode<State, Message> {
    fn from(widget: WidgetPod<State, Message>) -> Self {
        Self {
            widget: RefCell::new(widget),
        }
    }
}

impl<State, Message> fmt::Display for WidgetNode<State, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.widget.borrow().fmt(f)
    }
}

#[derive(Debug)]
struct WidgetPod<State, Message> {
    widget: RcWidget<State, Message>,
    state: Box<dyn Any>,
    event_mask: BitFlags<EventMask>,
    position: Point,
    size: Size,
    box_constraints: BoxConstraints,
    origin: Point,
    draw_cache: Option<Primitive>,
    needs_layout: bool,
    needs_draw: bool,
}

impl<State, Message> WidgetPod<State, Message> {
    fn new(widget: RcWidget<State, Message>) -> Self {
        let state = widget.initial_state();
        Self {
            widget,
            state,
            event_mask: BitFlags::empty(),
            position: Point::ZERO,
            size: Size::ZERO,
            box_constraints: BoxConstraints::LOOSE,
            origin: Point::ZERO,
            draw_cache: None,
            needs_layout: true,
            needs_draw: true,
        }
    }

    fn update(&mut self, new_widget: RcWidget<State, Message>) -> Effect<Message> {
        let effect =
            new_widget.on_lifecycle(Lifecycle::Updated(self.widget.as_any()), &mut self.state);

        self.widget = new_widget;
        self.needs_layout = true;
        self.needs_draw = true;

        effect
    }

    fn on_event(&mut self, event: Event<State>) -> Effect<Message> {
        let bounds = self.bounds();
        self.widget.on_event(event, bounds, &mut self.state)
    }

    fn on_lifecycle(&mut self, lifecycle: Lifecycle<&dyn Any>) -> Effect<Message> {
        self.widget.on_lifecycle(lifecycle, &mut self.state)
    }

    fn layout(
        &mut self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<State, Message>,
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
        origin: Point,
        children: &[NodeId],
        context: &mut DrawContext<State, Message>,
    ) -> (Primitive, Rect) {
        let bounds = Rect::new(origin + self.position, self.size);
        self.origin = origin;

        if !self.needs_draw {
            if let Some(primitive) = &self.draw_cache {
                return (primitive.clone(), bounds);
            }
        }

        let primitive = self.widget.draw(bounds, children, context, &mut self.state);
        self.draw_cache = Some(primitive.clone());
        self.needs_draw = false;

        (primitive, bounds)
    }

    fn bounds(&self) -> Rect {
        Rect::new(self.origin + self.position, self.size)
    }
}

impl<State, Message> fmt::Display for WidgetPod<State, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}", self.widget.short_type_name())?;
        write!(f, " pos={:?}", (self.position.x, self.position.y))?;
        write!(f, " size={:?}", (self.size.width, self.size.height))?;
        write!(f, " origin={:?}", (self.origin.x, self.origin.y))?;
        if !self.event_mask.is_empty() {
            write!(
                f,
                " event_mask={:?}",
                self.event_mask.iter().collect::<Vec<_>>()
            )?;
        }
        if self.needs_layout {
            write!(f, " needs_layout")?;
        }
        if self.needs_draw {
            write!(f, " needs_draw")?;
        }
        write!(f, ">")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LayoutContext<'a, State, Message> {
    widget_tree: &'a WidgetTree<State, Message>,
}

impl<'a, State, Message> LayoutContext<'a, State, Message> {
    pub fn get_size(&mut self, id: NodeId) -> Size {
        let widget = self.widget_tree.get(id);
        widget.size
    }

    pub fn get_layout_style(&self, id: NodeId) -> LayoutStyle {
        self.widget_tree.get(id).widget.layout_style()
    }

    pub fn set_position(&self, id: NodeId, position: Point) {
        let mut widget = self.widget_tree.get_mut(id);
        widget.position = position;
    }

    pub fn layout_child(&self, id: NodeId, box_constraints: BoxConstraints) -> Size {
        self.widget_tree.layout_child(id, box_constraints)
    }
}

#[derive(Debug)]
pub struct DrawContext<'a, State, Message> {
    widget_tree: &'a WidgetTree<State, Message>,
    origin: Point,
}

impl<'a, State, Message> DrawContext<'a, State, Message> {
    pub fn draw_child(&self, id: NodeId) -> Primitive {
        self.widget_tree.draw_child(id, self.origin)
    }
}

fn process_effect<State, Message, Handler>(
    effect: Effect<Message>,
    id: NodeId,
    widget: &mut WidgetPod<State, Message>,
    handler: &Handler,
    event_manager: &mut EventManager<NodeId>,
) where
    Handler: Fn(Command<Message>, NodeId),
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
            Effect::Command(command) => handler(command, id),
            Effect::Batch(effects) => queue.extend(effects),
        }

        if let Some(next) = queue.pop_front() {
            current = next;
        } else {
            break;
        }
    }
}
