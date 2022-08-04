use yuiui_support::slot_tree::{NodeId, SlotTree};
use std::mem;
use std::fmt;

use crate::children::Children as _;
use crate::component::{AnyComponent, Component};
use crate::element::Element;
use crate::view::{AnyView, View};
use crate::widget::AnyWidget;

pub type Id = NodeId;

pub struct World {
    pub element_tree: SlotTree<ElementNode>,
    pub widget_tree: SlotTree<WidgetNode>,
}

impl World {
    pub fn create<V: View, C: Component>(element: Element<V, C>) -> Self {
        Self::do_create(element, &mut Vec::new())
    }

    fn do_create<V: View, C: Component>(element: Element<V, C>, components: &mut Vec<Box<dyn AnyComponent>>) -> Self {
        match element {
            Element::View(view, children) => {
                let widget_node = WidgetNode::new(Box::new(view.build(&children)));
                let element_node = ElementNode::new(Box::new(view), mem::take(components));
                let mut world = World {
                    element_tree: SlotTree::new(element_node),
                    widget_tree: SlotTree::new(widget_node),
                };
                children.append(Id::ROOT, &mut world);
                world
            }
            Element::Component(node) => {
                let child = node.render();
                components.push(Box::new(node));
                Self::do_create(child, components)
            }
        }
    }

    pub(crate) fn append<V: View, C: Component>(&mut self, origin: Id, element: Element<V, C>) {
        self.do_append(origin, element, &mut Vec::new())
    }

    fn do_append<V: View, C: Component>(&mut self, origin: Id, element: Element<V, C>, components: &mut Vec<Box<dyn AnyComponent>>) {
        match element {
            Element::View(view, children) => {
                let widget_node = WidgetNode::new(Box::new(view.build(&children)));
                let widget_child = self.widget_tree.cursor_mut(origin).append_child(widget_node);
                let element_node = ElementNode::new(Box::new(view), mem::take(components));
                let child = self.element_tree.cursor_mut(origin).append_child(element_node);
                assert_eq!(widget_child, child);
                children.append(child, self);
            }
            Element::Component(node) => {
                let child = node.render();
                components.push(Box::new(node));
                self.do_append(origin, child, components);
            }
        }
    }

    pub(crate) fn update<V: View, C: Component>(&mut self, target: Id, component_index: usize, element: Element<V, C>) -> Option<Id> {
        let mut cursor = self.element_tree.cursor_mut(target);
        let mut node = cursor.node().data_mut();
        match element {
            Element::View(view, children) => {
                assert!(component_index == node.components.len());
                node.view = Box::new(view);
                node.removed = false;
                if children.len() > 0 {
                    let child = cursor.node().first_child().unwrap();
                    children.update(child, self)
                } else {
                    cursor.node().next_sibling()
                }
            }
            Element::Component(component) => {
                assert!(component_index < node.components.len());
                let element = component.render();
                node.components[component_index] = Box::new(component);
                self.update(target, component_index + 1, element)
            }
        }
    }

    pub(crate) fn remove(&mut self, target: Id, component_index: usize) -> Option<Id> {
        let mut cursor = self.element_tree.cursor_mut(target);
        let mut node = cursor.node().data_mut();

        assert!(component_index < node.components.len());

        for _component in node.components.drain(component_index..) {
            // TODO: component lifecycle
        }

        let next = cursor.node().next_sibling();

        if component_index > 0 {
            node.removed = true;
            // TODO: add unit of work
            let _ = cursor.drain_descendants();
        } else {
            let _ = cursor.drain_subtree();
        }

        next
    }
}

pub struct ElementNode {
    view: Box<dyn AnyView>,
    components: Vec<Box<dyn AnyComponent>>,
    removed: bool,
}

impl ElementNode {
    fn new(view: Box<dyn AnyView>, components: Vec<Box<dyn AnyComponent>>) -> ElementNode {
        Self {
            view,
            components,
            removed: false,
        }
    }
}

impl fmt::Display for ElementNode {
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

pub struct WidgetNode {
    widget: Box<dyn AnyWidget>,
}

impl WidgetNode {
    fn new(widget: Box<dyn AnyWidget>) -> WidgetNode {
        Self {
            widget
        }
    }
}

impl fmt::Display for WidgetNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{}",
            short_type_name(self.widget.name())
        )?;
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
