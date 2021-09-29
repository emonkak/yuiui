use yuiui_support::slot_tree::NodeId;

use crate::event::{MouseEvent, WindowEvent};
use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};
use crate::widget::{DrawContext, Effect, ElementInstance, Event, EventMask, Lifecycle, Widget};

pub struct Button<Message> {
    pub background: Background,
    pub on_click: Option<Box<dyn Fn(&MouseEvent) -> Effect<Message>>>,
}

impl<State, Message: 'static> Widget<State, Message> for Button<Message> {
    type LocalState = ButtonState;

    fn initial_state(&self) -> Self::LocalState {
        ButtonState { is_pressed: false }
    }

    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        _state: &mut Self::LocalState,
    ) -> Effect<Message> {
        match lifecycle {
            Lifecycle::Mounted => {
                Effect::AddListener([EventMask::PointerPressed, EventMask::PointerReleased].into())
            }
            _ => Effect::None,
        }
    }

    fn on_event(
        &self,
        event: Event<State>,
        bounds: Rectangle,
        state: &mut Self::LocalState,
    ) -> Effect<Message> {
        match event {
            Event::WindowEvent(WindowEvent::PointerPressed(event)) => {
                if bounds.snap().contains(event.position) {
                    state.is_pressed = true;
                }
            }
            Event::WindowEvent(WindowEvent::PointerReleased(event)) => {
                if state.is_pressed {
                    state.is_pressed = false;

                    if bounds.snap().contains(event.position) {
                        if let Some(ref on_click) = self.on_click {
                            return on_click(event);
                        }
                    }
                }
            }
            _ => {}
        }
        Effect::None
    }

    fn draw(
        &self,
        bounds: Rectangle,
        children: &[NodeId],
        context: &mut DrawContext<State, Message>,
        _state: &mut Self::LocalState,
    ) -> Primitive {
        let mut primitive = Primitive::Quad {
            bounds,
            background: self.background,
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        };

        for child in children {
            primitive = primitive + context.draw_child(*child);
        }

        primitive
    }
}

pub struct ButtonState {
    is_pressed: bool,
}

impl<State: 'static, Message: 'static> From<Button<Message>> for ElementInstance<State, Message> {
    fn from(widget: Button<Message>) -> Self {
        widget.into_rc().into()
    }
}
