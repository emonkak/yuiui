use std::collections::BTreeMap;

use geometrics::{BoxConstraints, Point, Rectangle, Size};
use graph::{NodeId};
use ui::{LayoutContext, LayoutResult, UIState};
use widget::Widget;

pub struct Row;
pub struct Column;

pub struct Flex {
    params: BTreeMap<NodeId, Params>,
    direction: Axis,

    // layout continuation state

    phase: Phase,
    ix: usize,
    minor: f32,

    // the total measure of non-flex children
    total_non_flex: f32,

    // the sum of flex parameters of all children
    flex_sum: f32,
}

pub enum Axis {
    Horizontal,
    Vertical,
}

// Layout happens in two phases. First, the non-flex children
// are laid out. Then, the remaining space is divided across
// the flex children.
#[derive(Clone, Copy, PartialEq)]
enum Phase {
    NonFlex,
    Flex,
}

#[derive(Copy, Clone, Default)]
struct Params {
    flex: f32,
}

impl Params {
    // Determine the phase in which this child should be measured.
    fn get_flex_phase(&self) -> Phase {
        if self.flex == 0.0 {
            Phase::NonFlex
        } else {
            Phase::Flex
        }
    }
}

impl Axis {
    fn major(&self, coords: Size) -> f32 {
        match *self {
            Axis::Horizontal => coords.width,
            Axis::Vertical => coords.height,
        }
    }

    fn minor(&self, coords: Size) -> f32 {
        match *self {
            Axis::Horizontal => coords.height,
            Axis::Vertical => coords.width,
        }
    }

    fn pack_point(&self, major: f32, minor: f32) -> Point {
        match *self {
            Axis::Horizontal => Point { x: major, y: minor },
            Axis::Vertical => Point { x: minor, y: major },
        }
    }

    fn pack_size(&self, major: f32, minor: f32) -> Size {
        match *self {
            Axis::Horizontal => Size { width: major, height: minor },
            Axis::Vertical => Size { width: minor, height: major },
        }
    }
}

impl Row {
    pub fn new() -> Flex {
        Flex {
            params: BTreeMap::new(),
            direction: Axis::Horizontal,
            phase: Phase::NonFlex,
            ix: 0,
            minor: 0.0,
            total_non_flex: 0.0,
            flex_sum: 0.0,
        }
    }
}

impl Column {
    pub fn new() -> Flex {
        Flex {
            params: BTreeMap::new(),
            direction: Axis::Vertical,

            phase: Phase::NonFlex,
            ix: 0,
            minor: 0.0,
            total_non_flex: 0.0,
            flex_sum: 0.0,
        }
    }
}

impl Flex {
    /// Add to UI with children.
    pub fn ui<WindowHandle: Clone, PaintContext>(self, children: &[NodeId], context: &mut UIState<WindowHandle, PaintContext>) -> NodeId {
        context.add(self, children)
    }

    /// Set the flex for a child widget.
    ///
    /// This function is used to set flex for a child widget, and is done while
    /// building, before adding to the UI. Likely we will need to think of other
    /// mechanisms to change parameters dynamically after building.
    pub fn set_flex(&mut self, child: NodeId, flex: f32) {
        let params = self.get_params_mut(child);
        params.flex = flex;
    }

    fn get_params_mut(&mut self, child: NodeId) -> &mut Params {
        self.params.entry(child).or_default()
    }

    fn get_params(&self, child: NodeId) -> Params {
        self.params.get(&child).cloned().unwrap_or(Default::default())
    }

    /// Return the index (within `children`) of the next child that belongs in
    /// the specified phase.
    fn get_next_child(&self, children: &[NodeId], start: usize, phase: Phase) -> Option<usize> {
        for ix in start..children.len() {
            if self.get_params(children[ix]).get_flex_phase() == phase {
                return Some(ix);
            }
        }
        None
    }

    /// Position all children, after the children have all been measured.
    fn finish_layout(&self, box_constraints: &BoxConstraints, children: &[NodeId], layout_context: &mut LayoutContext)
        -> LayoutResult
    {
        let mut major = 0.0;
        for &child in children {
            // top-align, could do center etc. based on child height
            layout_context.position_child(child, self.direction.pack_point(major, 0.0));
            major += self.direction.major(layout_context.get_child_size(child));
        }
        let total_major = self.direction.major(box_constraints.max);
        LayoutResult::Size(self.direction.pack_size(total_major, self.minor))
    }
}

impl<WindowHandle: Clone, PaintContext> Widget<WindowHandle, PaintContext> for Flex {
    fn layout(&mut self, box_constraints: &BoxConstraints, children: &[NodeId], size: Option<Size>,
        ctx: &mut LayoutContext) -> LayoutResult
    {
        if let Some(size) = size {
            let minor = self.direction.minor(size);
            self.minor = self.minor.max(minor);
            if self.phase == Phase::NonFlex {
                self.total_non_flex += self.direction.major(size);
            }

            // Advance to the next child; finish non-flex phase if at end.
            if let Some(ix) = self.get_next_child(children, self.ix + 1, self.phase) {
                self.ix = ix;
            } else if self.phase == Phase::NonFlex {
                if let Some(ix) = self.get_next_child(children, 0, Phase::Flex) {
                    self.ix = ix;
                    self.phase = Phase::Flex;
                } else {
                    return self.finish_layout(box_constraints, children, ctx);
                }
            } else {
                return self.finish_layout(box_constraints, children, ctx)
            }
        } else {
            // Start layout process, no children measured yet.
            if children.is_empty() {
                return LayoutResult::Size(box_constraints.min);
            }
            if let Some(ix) = self.get_next_child(children, 0, Phase::NonFlex) {
                self.ix = ix;
                self.phase = Phase::NonFlex;
            } else {
                // All children are flex, skip non-flex pass.
                self.ix = 0;
                self.phase = Phase::Flex;
            }
            self.total_non_flex = 0.0;
            self.flex_sum = children.iter().map(|id| self.get_params(*id).flex).sum();
            self.minor = self.direction.minor(box_constraints.min);
        }
        let (min_major, max_major) = if self.phase == Phase::NonFlex {
            (0.0, ::std::f32::INFINITY)
        } else {
            let total_major = self.direction.major(box_constraints.max);
            // TODO: should probably max with 0.0 to avoid negative sizes
            let remaining = total_major - self.total_non_flex;
            let major = remaining * self.get_params(children[self.ix]).flex / self.flex_sum;
            (major, major)
        };
        let child_bc = match self.direction {
            Axis::Horizontal => BoxConstraints {
                min: Size {
                    width: min_major,
                    height: box_constraints.min.height,
                },
                max: Size {
                    width: max_major,
                    height: box_constraints.max.height,
                }
            },
            Axis::Vertical => BoxConstraints {
                min: Size {
                    width: box_constraints.min.width,
                    height: min_major,
                },
                max: Size {
                    width: box_constraints.max.width,
                    height: max_major,
                }
            },
        };
        LayoutResult::RequestChild(children[self.ix], child_bc)
    }

    fn connect(&mut self, parent_handle: &WindowHandle, _rectangle: &Rectangle, _paint_context: &mut PaintContext) -> WindowHandle {
        parent_handle.clone()
    }

    fn on_child_removed(&mut self, child: NodeId) {
        self.params.remove(&child);
    }
}
