use std::any::{Any, TypeId};
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;
use yuiui_support::bit_flags::BitFlags;
use yuiui_support::slot_tree::{CursorMut, NodeId, SlotTree};

use super::event_manager::EventManager;
use super::reconciler::{ReconcileResult, Reconciler};
use super::{
    Attributes, Lifecycle, Command, EventMask, Effect, Children, ComponentElement, Element, Key, RcComponent, RcWidget, UnitOfWork,
    WidgetElement,
};

type ComponentIndex = usize;

#[derive(Debug)]
pub struct ElementTree<State, Message> {
    tree: SlotTree<ElementNode<State, Message>>,
    event_manager: EventManager<(NodeId, ComponentIndex)>,
}

impl<State, Message> ElementTree<State, Message> {
    pub fn new(root_widget: RcWidget<State, Message>, element: Element<State, Message>) -> Self
    where
        State: 'static,
        Message: 'static,
    {
        let element = WidgetElement {
            widget: root_widget,
            attributes: Default::default(),
            key: None,
            children: Rc::new(vec![element]),
        };
        let element_node = ElementNode {
            element: Some(element),
            component_stack: Vec::new(),
        };
        Self {
            tree: SlotTree::new(element_node),
            event_manager: EventManager::new(),
        }
    }

    pub fn render<Handler>(
        &mut self,
        id: NodeId,
        component_index: ComponentIndex,
        root: NodeId,
        command_handler: &Handler,
        pending_works: &mut Vec<UnitOfWork<State, Message>>,
    ) -> Option<(NodeId, ComponentIndex)>
    where
        Handler: Fn(Command<Message>),
    {
        let mut cursor = self.tree.cursor_mut(id);
        let component_stack = &mut cursor.current().data_mut().component_stack;

        if component_index < component_stack.len() {
            let component = &mut component_stack[component_index];
            let (is_updated, effect) = if let Some(pending_element) = component.pending_element.take() {
                component.update(pending_element)
            } else {
                let effect = component.on_lifecycle(Lifecycle::Mounted);
                (true, effect)
            };

            process_effect(
                effect,
                id,
                component_index,
                component,
                command_handler,
                &mut self.event_manager
            );

            if is_updated {
                let children = vec![component.render()];
                let reconciler = create_reconciler(&mut cursor, children, component_index);

                for result in reconciler {
                    self.commit(result, id, true, command_handler, pending_works)
                }
            }

            Some((id, component_index + 1))
        } else {
            let element = cursor.current().data_mut().element.as_ref().unwrap();
            let children = (*element.children).clone();
            let reconciler = create_reconciler(&mut cursor, children, component_index);

            for result in reconciler {
                self.commit(result, id, false, command_handler, pending_works)
            }

            self.tree
                .cursor(id)
                .descendants_from(root)
                .next()
                .map(|(next_id, _)| (next_id, 0))
        }
    }

    fn commit<Handler>(
        &mut self,
        result: ReconcileResult<ElementId, Element<State, Message>>,
        parent: NodeId,
        in_component_rendering: bool,
        command_handler: &Handler,
        pending_works: &mut Vec<UnitOfWork<State, Message>>,
    )
    where
        Handler: Fn(Command<Message>),
    {
        match result {
            ReconcileResult::Append(Element::WidgetElement(element)) => {
                let mut cursor = self.tree.cursor_mut(parent);
                if in_component_rendering {
                    let widget_node = cursor.current().data_mut();
                    widget_node.element = Some(element.clone());
                    pending_works.push(UnitOfWork::Append(
                        cursor.current().parent().unwrap(),
                        element,
                    ))
                } else {
                    cursor.append_child(ElementNode {
                        element: Some(element.clone()),
                        component_stack: Vec::new(),
                    });
                    pending_works.push(UnitOfWork::Append(parent, element))
                }
            }
            ReconcileResult::Append(Element::ComponentElement(element)) => {
                let mut cursor = self.tree.cursor_mut(parent);
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
            ReconcileResult::Insert(reference, Element::WidgetElement(element)) => {
                let mut cursor = self.tree.cursor_mut(reference.id());
                cursor.insert_before(ElementNode {
                    element: Some(element.clone()),
                    component_stack: Vec::new(),
                });
                pending_works.push(UnitOfWork::Insert(reference.id(), element))
            }
            ReconcileResult::Insert(reference, Element::ComponentElement(element)) => {
                let mut cursor = self.tree.cursor_mut(reference.id());
                let component = ComponentPod::from_element(element);
                cursor.insert_before(ElementNode {
                    element: None,
                    component_stack: vec![component],
                });
            }
            ReconcileResult::Update(ElementId::Widget(id), Element::WidgetElement(element)) => {
                let mut cursor = self.tree.cursor_mut(id);
                cursor.current().data_mut().element = Some(element.clone());
                pending_works.push(UnitOfWork::Update(id, element))
            }
            ReconcileResult::Update(
                ElementId::Component(id, component_index),
                Element::ComponentElement(element),
            ) => {
                let mut cursor = self.tree.cursor_mut(id);
                let component = &mut cursor.current().data_mut().component_stack[component_index];
                component.pending_element = Some(element);
            }
            ReconcileResult::UpdateAndMove(
                ElementId::Widget(id),
                reference,
                Element::WidgetElement(element),
            ) => {
                let mut cursor = self.tree.cursor_mut(id);
                cursor.current().data_mut().element = Some(element.clone());
                cursor.move_before(reference.id());
                pending_works.push(UnitOfWork::UpdateAndMove(id, reference.id(), element))
            }
            ReconcileResult::UpdateAndMove(
                ElementId::Component(id, component_index),
                reference,
                Element::ComponentElement(element),
            ) => {
                let mut cursor = self.tree.cursor_mut(id);
                let component = &mut cursor.current().data_mut().component_stack[component_index];
                component.pending_element = Some(element);
                cursor.move_before(reference.id());
            }
            ReconcileResult::Remove(ElementId::Widget(id)) => {
                let cursor = self.tree.cursor_mut(id);
                let _ = cursor.drain_subtree();
                pending_works.push(UnitOfWork::Remove(id))
            }
            ReconcileResult::Remove(ElementId::Component(id, component_index)) => {
                let mut cursor = self.tree.cursor_mut(id);
                let mut widget_node = cursor.current().data_mut();
                for (component_index, mut component) in widget_node.component_stack.drain(component_index..).enumerate() {
                    let effect = component.on_lifecycle(Lifecycle::Unmounted);
                    process_effect(
                        effect,
                        id,
                        component_index,
                        &mut component,
                        command_handler,
                        &mut self.event_manager
                    );
                }
                if component_index > 0 {
                    widget_node.element = None;
                    let _ = cursor.drain_descendants();
                    pending_works.push(UnitOfWork::RemoveChildren(id))
                } else {
                    let _ = cursor.drain_subtree();
                    pending_works.push(UnitOfWork::Remove(id))
                }
            }
            _ => unreachable!("element kind mismatch"),
        }
    }
}

impl<State: fmt::Debug, Message: fmt::Debug> fmt::Display for ElementTree<State, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.fmt(f)
    }
}

#[derive(Debug)]
struct ElementNode<State, Message> {
    element: Option<WidgetElement<State, Message>>,
    component_stack: Vec<ComponentPod<State, Message>>,
}

#[derive(Debug)]
struct ComponentPod<State, Message> {
    component: RcComponent<State, Message>,
    attributes: Rc<Attributes>,
    children: Children<State, Message>,
    state: Box<dyn Any>,
    key: Option<Key>,
    pending_element: Option<ComponentElement<State, Message>>,
    event_mask: BitFlags<EventMask>,
}

impl<State, Message> ComponentPod<State, Message> {
    fn from_element(element: ComponentElement<State, Message>) -> Self {
        let state = element.component.initial_state();
        Self {
            component: element.component,
            attributes: element.attributes,
            children: element.children,
            key: element.key,
            state,
            pending_element: None,
            event_mask: BitFlags::empty(),
        }
    }

    fn update(&mut self, element: ComponentElement<State, Message>) -> (bool, Effect<Message>) {
        let should_update = &*self.attributes != &*element.attributes
            || self.component.should_update(
                element.component.as_any(),
                &self.children,
                &element.children,
                &self.state,
            );

        let effect = if should_update {
            self.on_lifecycle(
                Lifecycle::Updated(element.component.as_any()),
            )
        } else {
            Effect::None
        };

        self.component = element.component;
        self.attributes = element.attributes;
        self.children = element.children;

        (should_update, effect)
    }

    fn on_lifecycle(&mut self, lifecycle: Lifecycle<&dyn Any>) -> Effect<Message> {
        self.component.on_lifecycle(
            lifecycle,
            &mut self.state,
        )
    }

    fn render(&self) -> Element<State, Message> {
        self.component.render(&self.children, &self.state)
    }

    fn as_any(&self) -> &dyn Any {
        self.component.as_any()
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

fn create_reconciler<State, Message>(
    cursor: &mut CursorMut<ElementNode<State, Message>>,
    children: Vec<Element<State, Message>>,
    component_index: ComponentIndex,
) -> Reconciler<TypedKey, ElementId, Element<State, Message>> {
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
    let mut new_elements: Vec<Option<Element<State, Message>>> = Vec::with_capacity(children.len());

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

fn process_effect<State, Message, Handler>(
    effect: Effect<Message>,
    id: NodeId,
    component_index: ComponentIndex,
    component: &mut ComponentPod<State, Message>,
    command_handler: &Handler,
    event_manager: &mut EventManager<(NodeId, ComponentIndex)>,
) where
    Handler: Fn(Command<Message>),
{
    let mut queue = VecDeque::new();
    let mut current = effect;

    loop {
        match current {
            Effect::None => {}
            Effect::AddListener(event_mask) => {
                let new_events = event_mask & (event_mask ^ component.event_mask);
                event_manager.add_listener((id, component_index), new_events);
                component.event_mask |= event_mask;
            }
            Effect::RemoveListener(event_mask) => {
                let removed_events = event_mask & component.event_mask;
                event_manager.remove_listener((id, component_index), removed_events);
                component.event_mask ^= event_mask;
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
