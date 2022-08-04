use std::fmt;
use std::mem;
use yuiui_support::slot_tree::{NodeId, SlotTree};

use crate::component::{AnyComponent, Component};
use crate::element::{AnyElement, Element};
use crate::element_seq::ElementSeq as _;
use crate::view::{AnyView, View};
use crate::widget::AnyWidget;

pub type Id = NodeId;

pub type Version = u64;

pub struct World {
    pub element_tree: SlotTree<ElementNode>,
    pub widget_tree: SlotTree<WidgetNode>,
    pending_works: Vec<UnitOfWork>,
}

impl World {
    pub fn create<V: View, C: Component>(element: Element<V, C>) -> Self {
        Self::do_create(element, &mut Vec::new())
    }

    fn do_create<V: View, C: Component>(
        element: Element<V, C>,
        components: &mut Vec<Box<dyn AnyComponent>>,
    ) -> Self {
        match element {
            Element::View(view, children) => {
                let widget_node = WidgetNode::new(Box::new(View::build(&view, &children)));
                let element_node = ElementNode::new(Box::new(view), mem::take(components));
                let mut world = World {
                    element_tree: SlotTree::new(element_node),
                    widget_tree: SlotTree::new(widget_node),
                    pending_works: Vec::new(),
                };
                children.placement(Id::ROOT, &mut world);
                world
            }
            Element::Component(node) => {
                let child = Component::render(&node);
                components.push(Box::new(node));
                Self::do_create(child, components)
            }
        }
    }

    pub fn render(&mut self, target: Id, component_index: usize, root: Id) -> Option<(Id, usize)> {
        let mut cursor = self.element_tree.cursor_mut(target);
        let node = &mut cursor.node().data;

        if let Some(element) = node.pending_element.take() {
            match element {
                AnyElement::View(view, children) => {
                    view.reconcile_children(children, target, self);

                    let mut cursor = self.element_tree.cursor_mut(target);
                    let mut node = &mut cursor.node().data;

                    // TODO: Do lifecycle
                    if node.mounted {
                    } else {
                    }

                    node.view = view;
                    node.mounted = true;

                    let mut descendants = cursor.descendants_from(root);
                    descendants.next().map(|(next_id, _)| (next_id, 0))
                }
                AnyElement::Component(component) => {
                    node.pending_element = Some(component.render());
                    // TODO: Do lifecycle
                    if component_index < node.components.len() {
                        // update
                        node.components[component_index] = component;
                    } else {
                        // mount
                        node.components.push(component);
                    }
                    Some((target, component_index + 1))
                }
            }
        } else if component_index < node.components.len() {
            let component = &node.components[component_index];
            node.pending_element = Some(component.render());
            Some((target, component_index + 1))
        } else {
            None
        }
    }

    pub(crate) fn append<V: View, C: Component>(&mut self, origin: Id, element: Element<V, C>) {
        self.do_append(origin, element, &mut Vec::new())
    }

    fn do_append<V: View, C: Component>(
        &mut self,
        origin: Id,
        element: Element<V, C>,
        components: &mut Vec<Box<dyn AnyComponent>>,
    ) {
        match element {
            Element::View(view, children) => {
                let view = Box::new(view);
                let mut cursor = self.element_tree.cursor_mut(origin);
                let node = ElementNode::new(view, mem::take(components));
                let child = cursor.append_child(node);
                self.pending_works.push(UnitOfWork::Append(origin, child));
                children.placement(child, self);
            }
            Element::Component(node) => {
                let child = Component::render(&node);
                components.push(Box::new(node));
                self.do_append(origin, child, components);
            }
        }
    }

    pub(crate) fn insert<V: View, C: Component>(&mut self, reference: Id, element: Element<V, C>) {
        self.do_insert(reference, element, &mut Vec::new())
    }

    fn do_insert<V: View, C: Component>(
        &mut self,
        reference: Id,
        element: Element<V, C>,
        components: &mut Vec<Box<dyn AnyComponent>>,
    ) {
        match element {
            Element::View(view, children) => {
                let view = Box::new(view);
                let mut cursor = self.element_tree.cursor_mut(reference);
                let node = ElementNode::new(view, mem::take(components));
                let child = cursor.append_child(node);
                self.pending_works
                    .push(UnitOfWork::Insert(reference, child));
                children.placement(child, self);
            }
            Element::Component(node) => {
                let child = Component::render(&node);
                components.push(Box::new(node));
                self.do_insert(reference, child, components);
            }
        }
    }

    pub(crate) fn update<V: View, C: Component>(
        &mut self,
        target: Id,
        element: Element<V, C>,
    ) -> Option<Id> {
        self.do_update(target, 0, element)
    }

    pub(crate) fn do_update<V: View, C: Component>(
        &mut self,
        target: Id,
        component_index: usize,
        element: Element<V, C>,
    ) -> Option<Id> {
        let mut cursor = self.element_tree.cursor_mut(target);
        let mut node = &mut cursor.node().data;
        match element {
            Element::View(view, children) => {
                assert!(component_index == node.components.len());
                let view = Box::new(view);
                self.pending_works
                    .push(UnitOfWork::Update(target, node.version));
                node.view = view;
                node.removed = false;
                if children.len() > 0 {
                    let child = cursor.node().first_child.unwrap();
                    children.reconcile(child, self)
                } else {
                    cursor.node().next_sibling
                }
            }
            Element::Component(component) => {
                assert!(component_index < node.components.len());
                let old_component = &node.components[component_index];
                if component.should_update(old_component.as_any().downcast_ref().unwrap()) {
                    let element = Component::render(&component);
                    node.components[component_index] = Box::new(component);
                    self.do_update(target, component_index + 1, element)
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn reposition<V: View, C: Component>(&mut self, target: Id, reference: Id) {
        self.pending_works
            .push(UnitOfWork::Reposition(target, reference));
    }

    pub(crate) fn remove(&mut self, target: Id) -> Option<Id> {
        let mut cursor = self.element_tree.cursor_mut(target);
        let node = &mut cursor.node().data;

        for _component in &node.components {
            // TODO: do lifecycle
        }

        let next = cursor.node().next_sibling;

        self.pending_works.push(UnitOfWork::DetachSubtree(target));

        for _ in cursor.drain_subtree() {
            // TODO: do lifecycle
        }

        next
    }
}

pub struct ElementNode {
    view: Box<dyn AnyView>,
    components: Vec<Box<dyn AnyComponent>>,
    pending_element: Option<AnyElement>,
    version: Version,
    mounted: bool,
    removed: bool,
}

impl ElementNode {
    fn new(view: Box<dyn AnyView>, components: Vec<Box<dyn AnyComponent>>) -> ElementNode {
        Self {
            view,
            components,
            pending_element: None,
            version: 0,
            mounted: false,
            removed: false,
        }
    }
}

impl fmt::Display for ElementNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}", short_type_name(self.view.name()))?;
        if !self.components.is_empty() {
            write!(f, " components=")?;
            f.debug_list()
                .entries(
                    self.components
                        .iter()
                        .map(|component| short_type_name(component.name())),
                )
                .finish()?;
        }
        write!(f, ">")?;
        Ok(())
    }
}

pub struct WidgetNode {
    widget: Box<dyn AnyWidget>,
}

impl WidgetNode {
    fn new(widget: Box<dyn AnyWidget>) -> WidgetNode {
        Self { widget }
    }
}

impl fmt::Display for WidgetNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}", short_type_name(self.widget.name()))?;
        write!(f, ">")?;
        Ok(())
    }
}

enum UnitOfWork {
    Append(Id, Id),
    Insert(Id, Id),
    Update(Id, Version),
    Reposition(NodeId, NodeId),
    DetachSubtree(NodeId),
}

fn short_type_name<'a>(name: &'a str) -> &'a str {
    name.split('<')
        .next()
        .unwrap_or(name)
        .split("::")
        .last()
        .unwrap_or(name)
}
