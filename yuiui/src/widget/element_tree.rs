use std::any::{Any, TypeId};
use std::fmt;
use std::rc::Rc;
use yuiui_support::slot_tree::{CursorMut, NodeId, SlotTree};

use super::reconciler::{ReconcileResult, Reconciler};
use super::{
    Attributes, Children, ComponentElement, Element, Key, RcComponent, RcWidget, UnitOfWork,
    WidgetElement,
};

type ComponentIndex = usize;

#[derive(Debug)]
pub struct ElementTree<State, Message> {
    tree: SlotTree<ElementNode<State, Message>>,
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
        let tree = SlotTree::new(element_node);
        Self { tree }
    }

    pub fn render(
        &mut self,
        id: NodeId,
        component_index: ComponentIndex,
        root: NodeId,
        pending_works: &mut Vec<UnitOfWork<State, Message>>,
    ) -> Option<(NodeId, ComponentIndex)> {
        let mut cursor = self.tree.cursor_mut(id);
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

                for result in reconciler {
                    self.commit(result, id, true, pending_works)
                }
            }

            Some((id, component_index + 1))
        } else {
            let element = cursor.current().data_mut().element.as_ref().unwrap();
            let children = (*element.children).clone();
            let reconciler = create_reconciler(&mut cursor, children, component_index);

            for result in reconciler {
                self.commit(result, id, false, pending_works)
            }

            self.tree
                .cursor(id)
                .descendants_from(root)
                .next()
                .map(|(next_id, _)| (next_id, 0))
        }
    }

    fn commit(
        &mut self,
        result: ReconcileResult<ElementId, Element<State, Message>>,
        parent: NodeId,
        in_component_rendering: bool,
        pending_works: &mut Vec<UnitOfWork<State, Message>>,
    ) {
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
                let _ = widget_node.component_stack.drain(component_index..);
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
        }
    }

    fn update(&mut self, element: ComponentElement<State, Message>) -> bool {
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
