use std::error;
use std::time::{Duration, Instant};
use yuiui_support::slot_tree::NodeId;

use super::message::ApplicationMessage;
use super::render_loop::{RenderFlow, RenderLoop};
use crate::event::WindowEvent;
use crate::graphics::{Color, Primitive, Renderer};
use crate::ui::{ControlFlow, Event, EventLoop, EventLoopContext, Window, WindowContainer};
use crate::widget::Element;

pub fn run<State, Message, Window, EventLoop, Renderer>(
    mut window_container: WindowContainer<Window>,
    mut event_loop: EventLoop,
    mut renderer: Renderer,
    element: Element<State, Message>,
) -> Result<(), Box<dyn error::Error>>
where
    State: 'static,
    Message: 'static,
    Window: 'static + self::Window,
    EventLoop: 'static + self::EventLoop<ApplicationMessage<Message>, WindowId = Window::Id>,
    Renderer: 'static + self::Renderer,
{
    let viewport = window_container.viewport();

    let mut render_loop = RenderLoop::new(element);
    let mut pipeline = renderer.create_pipeline(Primitive::None);
    let mut surface = renderer.create_surface(viewport);

    event_loop.run(|event, context| {
        match event {
            Event::LoopInitialized => {
                context.request_idle(|deadline| ApplicationMessage::Render(deadline));
            }
            Event::Message(ApplicationMessage::Quit) => return ControlFlow::Break,
            Event::Message(ApplicationMessage::RequestUpdate(id, component_index)) => {
                render_loop.schedule_update(id, component_index)
            }
            Event::Message(ApplicationMessage::Render(deadline)) => loop {
                let viewport = window_container.viewport();
                match render_loop.render(viewport, context) {
                    RenderFlow::Continue => {
                        if deadline - Instant::now() < Duration::from_secs(1) {
                            context.request_idle(|deadline| ApplicationMessage::Render(deadline));
                            break;
                        }
                    }
                    RenderFlow::Paint(primitive, _scissor_bounds) => {
                        let viewport = window_container.viewport();
                        pipeline = renderer.create_pipeline(primitive);
                        renderer.perform_pipeline(
                            &mut pipeline,
                            &mut surface,
                            &viewport,
                            Color::WHITE,
                        );
                        break;
                    }
                    RenderFlow::Idle => break,
                }
            },
            Event::Message(ApplicationMessage::Broadcast(_message)) => {}
            Event::WindowEvent(_, WindowEvent::RedrawRequested(bounds)) => {
                let viewport = window_container.viewport();
                renderer.perform_pipeline(&mut pipeline, &mut surface, &viewport, Color::WHITE);
                render_loop.dispatch(&WindowEvent::RedrawRequested(bounds).into(), context);
            }
            Event::WindowEvent(_, WindowEvent::SizeChanged(size)) => {
                if window_container.resize_viewport(size) {
                    let viewport = window_container.viewport();
                    renderer.configure_surface(&mut surface, &viewport);
                    render_loop.schedule_update(NodeId::ROOT, 0);
                    context.request_idle(|deadline| ApplicationMessage::Render(deadline));
                }
                render_loop.dispatch(&WindowEvent::SizeChanged(size).into(), context);
            }
            Event::WindowEvent(_, WindowEvent::Closed) => {
                render_loop.dispatch(&WindowEvent::Closed.into(), context);
                return ControlFlow::Break;
            }
            Event::WindowEvent(_, event) => {
                render_loop.dispatch(&event.into(), context);
            }
        }

        ControlFlow::Continue
    })?;

    Ok(())
}
