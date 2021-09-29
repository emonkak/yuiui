use std::error;
use std::time::{Duration, Instant};
use yuiui_support::slot_tree::NodeId;

use super::message::ApplicationMessage;
use super::{RenderFlow, RenderLoop, Store};
use crate::event::WindowEvent;
use crate::graphics::{Color, Primitive, Renderer};
use crate::ui::{
    ControlFlow, Event as UIEvent, EventLoop, EventLoopContext, Window, WindowContainer,
};
use crate::widget::Event;

pub fn run<State, Reducer, Message, Window, EventLoop, Renderer>(
    mut render_loop: RenderLoop<State, Message>,
    mut store: Store<State, Reducer>,
    mut window_container: WindowContainer<Window>,
    mut event_loop: EventLoop,
    mut renderer: Renderer,
) -> Result<(), Box<dyn error::Error>>
where
    State: 'static,
    Reducer: Fn(&mut State, Message) -> bool,
    Message: 'static,
    Window: 'static + self::Window,
    EventLoop: 'static + self::EventLoop<ApplicationMessage<Message>, WindowId = Window::Id>,
    Renderer: 'static + self::Renderer,
{
    let viewport = window_container.viewport();
    let mut pipeline = renderer.create_pipeline(Primitive::None);
    let mut surface = renderer.create_surface(viewport);

    event_loop.run(|event, context| {
        println!("EVENT: {:?}", event);
        match event {
            UIEvent::LoopInitialized => {
                context.request_idle(|deadline| ApplicationMessage::RequestRender(deadline));
            }
            UIEvent::Message(ApplicationMessage::Quit) => return ControlFlow::Break,
            UIEvent::Message(ApplicationMessage::RequestUpdate(id, component_index)) => {
                if render_loop.schedule_update(id, component_index) {
                    context.request_idle(|deadline| ApplicationMessage::RequestRender(deadline));
                }
            }
            UIEvent::Message(ApplicationMessage::RequestRender(deadline)) => loop {
                let viewport = window_container.viewport();
                match render_loop.render(viewport, context) {
                    RenderFlow::Continue => {
                        if deadline - Instant::now() < Duration::from_millis(1) {
                            context.request_idle(|deadline| {
                                ApplicationMessage::RequestRender(deadline)
                            });
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
            UIEvent::Message(ApplicationMessage::Broadcast(message)) => {
                if store.dispatch(message) {
                    render_loop.dispatch(&Event::StateChanged(store.state()), context);
                }
            }
            UIEvent::WindowEvent(_, WindowEvent::RedrawRequested(bounds)) => {
                let viewport = window_container.viewport();
                renderer.perform_pipeline(&mut pipeline, &mut surface, &viewport, Color::WHITE);
                render_loop.dispatch(&WindowEvent::RedrawRequested(bounds).into(), context);
            }
            UIEvent::WindowEvent(_, WindowEvent::SizeChanged(size)) => {
                if window_container.resize_viewport(size) {
                    let viewport = window_container.viewport();
                    renderer.configure_surface(&mut surface, &viewport);
                    if render_loop.schedule_update(NodeId::ROOT, 0) {
                        context.request_idle(|deadline| ApplicationMessage::RequestRender(deadline));
                    }
                }
                render_loop.dispatch(&WindowEvent::SizeChanged(size).into(), context);
            }
            UIEvent::WindowEvent(_, WindowEvent::Closed) => {
                render_loop.dispatch(&WindowEvent::Closed.into(), context);
                return ControlFlow::Break;
            }
            UIEvent::WindowEvent(_, event) => {
                render_loop.dispatch(&event.into(), context);
            }
        }

        ControlFlow::Continue
    })?;

    Ok(())
}
