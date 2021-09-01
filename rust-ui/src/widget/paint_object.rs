use std::any::{self, Any};
use std::fmt;
use std::marker::PhantomData;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest};
use crate::support::generator::Generator;

use super::element::ElementId;
use super::message::MessageEmitter;
use super::widget::PolyWidget;

pub type PolyPaintObject<Renderer> =
    dyn PaintObject<Renderer, Widget = PolyWidget<Renderer>, Message = dyn Any + Send>;

pub trait PaintObject<Renderer> {
    type Widget: ?Sized;

    type Message: ?Sized;

    #[inline]
    fn layout<'a>(
        &'a mut self,
        _widget: &'a Self::Widget,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        Generator::new(move |co| async move {
            if let Some(child_id) = child_ids.first() {
                co.suspend(LayoutRequest::LayoutChild(*child_id, box_constraints))
                    .await
            } else {
                box_constraints.max
            }
        })
    }

    #[inline]
    fn draw(
        &mut self,
        _widget: &Self::Widget,
        _bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
    ) -> Option<Primitive> {
        None
    }

    #[inline]
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    #[inline]
    fn short_type_name(&self) -> &'static str {
        let name = self.type_name();
        name.split('<')
            .next()
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<R> fmt::Debug for PolyPaintObject<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {{ .. }}", self.short_type_name())
    }
}

pub struct PaintObjectProxy<P, R, W, M> {
    pub paint_object: P,
    pub renderer_type: PhantomData<R>,
    pub widget_type: PhantomData<W>,
    pub message_type: PhantomData<M>,
}

impl<P, R, W, M> PaintObjectProxy<P, R, W, M> {
    pub fn new(paint_object: P) -> Self {
        Self {
            paint_object,
            renderer_type: PhantomData,
            widget_type: PhantomData,
            message_type: PhantomData,
        }
    }
}

impl<P, R, W, M> PaintObject<R> for PaintObjectProxy<P, R, W, M>
where
    P: PaintObject<R, Widget = W, Message = M> + 'static,
    R: 'static,
    W: 'static,
    M: 'static,
{
    type Widget = PolyWidget<R>;
    type Message = dyn Any + Send;

    #[inline]
    fn layout<'a>(
        &'a mut self,
        widget: &'a Self::Widget,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        renderer: &mut R,
        context: &mut MessageEmitter,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.paint_object.layout(
            widget.as_any().downcast_ref().unwrap(),
            box_constraints,
            child_ids,
            renderer,
            context,
        )
    }

    #[inline]
    fn draw(
        &mut self,
        widget: &Self::Widget,
        bounds: Rectangle,
        renderer: &mut R,
        context: &mut MessageEmitter,
    ) -> Option<Primitive> {
        self.paint_object.draw(
            widget.as_any().downcast_ref().unwrap(),
            bounds,
            renderer,
            context,
        )
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self.paint_object.as_any()
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.paint_object.as_any_mut()
    }
}

unsafe impl<P: Send, R, W, M> Send for PaintObjectProxy<P, R, W, M> {}

unsafe impl<P: Sync, R, W, M> Sync for PaintObjectProxy<P, R, W, M> {}
