use std::any;
use std::array;
use std::fmt;

use geometrics::{Rectangle, Size};
use layout::{BoxConstraints, LayoutResult, LayoutContext};
use paint::PaintContext;
use tree::NodeId;

#[derive(Debug)]
pub struct Element<Window> {
    pub instance: Box<dyn Widget<Window>>,
    pub children: Box<[Element<Window>]>,
}

#[derive(Debug)]
pub enum Child<Window> {
    Multiple(Vec<Element<Window>>),
    Single(Element<Window>),
    Empty,
}

pub trait Widget<Window>: WidgetMaker {
    fn name(&self) -> &'static str {
        let full_name = any::type_name::<Self>();
        full_name
            .rsplit_once("::")
            .map(|(_, last)| last)
            .unwrap_or(full_name)
    }

    fn connect(&mut self, _parent_handle: &Window, _rectangle: &Rectangle) -> Option<Window> {
        None
    }

    fn disconnect(&mut self, _handle: &Window) {
    }

    fn layout(
        &mut self,
        node_id: NodeId,
        response: Option<(NodeId, Size)>,
        box_constraints: &BoxConstraints,
        layout_context: LayoutContext<'_, Window>,
    ) -> LayoutResult {
        if let Some((child, size)) = response {
            layout_context[child].arrange(Default::default());
            LayoutResult::Size(size)
        } else {
            if let Some(child) = layout_context[node_id].first_child() {
                LayoutResult::RequestChild(child, *box_constraints)
            } else {
                LayoutResult::Size(box_constraints.max)
            }
        }
    }

    fn paint(&mut self, _handle: &Window, _rectangle: &Rectangle, _paint_context: &mut PaintContext<Window>) {
    }

    fn render_children(&self, children: Box<[Element<Window>]>) -> Box<[Element<Window>]> {
        children
    }

    fn should_update(&self, _element: &Element<Window>) -> bool {
        true
    }

    fn same_widget(&self, other: &dyn Widget<Window>) -> bool where Self: Sized + PartialEq + 'static {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map(|other| self == other)
            .unwrap_or(false)
    }

    fn as_any(&self) -> &dyn any::Any;
}

pub trait WidgetMaker {
}

impl<Window> Element<Window> {
    pub fn new<const N: usize>(widget: impl Widget<Window> + 'static, children: [Element<Window>; N]) -> Self {
        Self {
            instance: Box::new(widget),
            children: Box::new(children),
        }
    }

    pub fn build<const N: usize>(widget: impl Widget<Window> + 'static, children: [Child<Window>; N]) -> Self {
        let mut flatten_children = Vec::with_capacity(N);

        for child in array::IntoIter::new(children) {
            match child {
                Child::Multiple(elements) => {
                    for element in elements {
                        flatten_children.push(element)
                    }
                }
                Child::Single(element) => {
                    flatten_children.push(element)
                }
                _ => {}
            }
        }

        Self {
            instance: Box::new(widget),
            children: flatten_children.into_boxed_slice(),
        }
    }

    fn fmt_rec(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let name = self.instance.name();
        let indent_str = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
        if self.children.len() > 0 {
            write!(f, "{}<{}>", indent_str, name)?;
            for i in 0..self.children.len() {
                write!(f, "\n")?;
                self.children[i].fmt_rec(f, level + 1)?
            }
            write!(f, "\n{}</{}>", indent_str, name)?;
        } else {
            write!(f, "{}<{}></{}>", indent_str, name, name)?;
        }
        Ok(())
    }
}

impl<Window> fmt::Display for Element<Window> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_rec(f, 0)
    }
}

impl<Window> From<Vec<Element<Window>>> for Child<Window> {
    fn from(elements: Vec<Element<Window>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<Window> From<Option<Element<Window>>> for Child<Window> {
    fn from(element: Option<Element<Window>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::Empty,
        }
    }
}

impl<Window> From<Element<Window>> for Child<Window> {
    fn from(element: Element<Window>) -> Self {
        Child::Single(element)
    }
}

impl<Window, W: Widget<Window> + WidgetMaker + 'static> From<W> for Child<Window> {
    fn from(widget: W) -> Self {
        Child::Single(Element {
            instance: Box::new(widget),
            children: Box::new([])
        })
    }
}

impl<Window> fmt::Display for dyn Widget<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<Window> fmt::Debug for dyn Widget<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
