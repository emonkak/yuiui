use yuiui_support::slot_tree::{NodeId, SlotTree};
use std::mem;
use std::fmt;

use crate::children::Children as _;
use crate::component::{AnyComponent, Component};
use crate::element::Element;
use crate::view::{AnyView, View};

pub type Id = NodeId;

pub struct World {
    pub tree: SlotTree<Node>,
}

impl World {
    pub fn create<V: View, C: Component>(element: Element<V, C>) -> Self {
        Self::do_create(element, &mut Vec::new())
    }

    fn do_create<V: View, C: Component>(element: Element<V, C>, components: &mut Vec<Box<dyn AnyComponent>>) -> Self {
        match element {
            Element::View(view, children) => {
                let node = Node::new(Box::new(view), mem::take(components));
                let mut world = World {
                    tree: SlotTree::new(node),
                };
                children.attach(NodeId::ROOT, &mut world);
                world
            }
            Element::Component(node) => {
                let child = node.render();
                components.push(Box::new(node));
                Self::do_create(child, components)
            }
        }
    }

    pub(crate) fn attach<V: View, C: Component>(&mut self, origin: NodeId, element: Element<V, C>) {
        self.do_attach(origin, element, &mut Vec::new())
    }

    fn do_attach<V: View, C: Component>(&mut self, origin: NodeId, element: Element<V, C>, components: &mut Vec<Box<dyn AnyComponent>>) {
        match element {
            Element::View(view, children) => {
                let node = Node::new(Box::new(view), mem::take(components));
                let child = self.tree.cursor_mut(origin).append_child(node);
                children.attach(child, self);
            }
            Element::Component(node) => {
                let child = node.render();
                components.push(Box::new(node));
                self.do_attach(origin, child, components);
            }
        }
    }

    pub(crate) fn update<V: View, C: Component>(&mut self, target: NodeId, element: Element<V, C>) -> Option<NodeId> {
        let mut cursor = self.tree.cursor_mut(target);
        let mut node = cursor.node().data_mut();
        let next = cursor.node().next_sibling();
        match element {
            Element::View(view, children) => {
                assert!(node.update_index == node.components.len());
                node.view = Box::new(view);
                node.update_index += 1;
                children.reconcile(cursor.id(), self);
                next
            }
            Element::Component(component) => {
                assert!(node.update_index < node.components.len());
                let element = component.render();
                node.components[node.update_index] = Box::new(component);
                node.update_index += 1;
                self.update(target, element)
            }
        }
    }
}

pub struct Node {
    view: Box<dyn AnyView>,
    components: Vec<Box<dyn AnyComponent>>,
    update_index: usize,
}

impl Node {
    fn new(view: Box<dyn AnyView>, components: Vec<Box<dyn AnyComponent>>) -> Node {
        Self {
            view,
            components,
            update_index: 0,
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{}",
            short_type_name(self.view.name())
        )?;
        if !self.components.is_empty() {
            write!(f, " components=")?;
            f.debug_list()
                .entries(self.components.iter().map(|component| short_type_name(component.name())))
                .finish()?;
        }
        write!(f, ">")?;
        Ok(())
    }
}

fn short_type_name<'a>(name: &'a str) -> &'a str {
    name.split('<')
        .next()
        .unwrap_or(name)
        .split("::")
        .last()
        .unwrap_or(name)
}
