use futures::FutureExt;
use std::time::{Duration, Instant};
use yuiui_support::slot_tree::NodeId;

use super::message::InternalMessage;
use super::{RenderFlow, RenderLoop, Store};
use crate::event::WindowEvent;
use crate::graphics::{Color, Primitive, Renderer};
use crate::ui::{
    ControlFlow, Event as UIEvent, EventLoop, EventLoopContext, Window, WindowContainer,
};
use crate::widget::{Command, ComponentIndex, Event};

pub fn run<State, Reducer, Message, Window, EventLoop, Renderer>(
    mut render_loop: RenderLoop<State, Message>,
    mut store: Store<State, Reducer>,
    mut window_container: WindowContainer<Window>,
    mut event_loop: EventLoop,
    mut renderer: Renderer,
) -> anyhow::Result<()>
where
    State: 'static,
    Reducer: Fn(&mut State, Message) -> bool,
    Message: 'static,
    Window: 'static + self::Window,
    EventLoop: 'static + self::EventLoop<InternalMessage<Message>, WindowId = Window::Id>,
    Renderer: 'static + self::Renderer,
{
    let viewport = window_container.viewport();
    let mut pipeline = renderer.create_pipeline(Primitive::None);
    let mut surface = renderer.create_surface(viewport);

    event_loop.run(|event, context| {
        match event {
            UIEvent::LoopInitialized => {
                context.request_idle(|deadline| InternalMessage::RequestRender(deadline));
            }
            UIEvent::Message(InternalMessage::Quit) => return ControlFlow::Break,
            UIEvent::Message(InternalMessage::RequestUpdate(id, component_index)) => {
                if render_loop.schedule_update(id, component_index) {
                    context.request_idle(|deadline| InternalMessage::RequestRender(deadline));
                }
            }
            UIEvent::Message(InternalMessage::RequestRender(deadline)) => loop {
                let viewport = window_container.viewport();
                match render_loop.render(viewport, &|command, id, component_index| {
                    run_command(context, command, id, component_index)
                }) {
                    RenderFlow::Continue => {
                        let time_remaining = deadline - Instant::now();
                        if time_remaining < Duration::from_millis(1) {
                            context
                                .request_idle(|deadline| InternalMessage::RequestRender(deadline));
                            break;
                        }
                    }
                    RenderFlow::Done(primitive, effective_bounds) => {
                        let viewport = window_container.viewport();
                        pipeline = renderer.create_pipeline(primitive);
                        renderer.perform_pipeline(
                            &mut pipeline,
                            &mut surface,
                            &viewport,
                            effective_bounds,
                            Color::WHITE,
                        );
                        break;
                    }
                    RenderFlow::Idle => break,
                }
            },
            UIEvent::Message(InternalMessage::Broadcast(message)) => {
                if store.dispatch(message) {
                    render_loop.dispatch(
                        Event::StateChanged(store),
                        &|command, id, component_index| {
                            run_command(context, command, id, component_index)
                        },
                    );
                }
            }
            UIEvent::WindowEvent(_, WindowEvent::RedrawRequested) => {
                let viewport = window_container.viewport();
                renderer.perform_pipeline(
                    &mut pipeline,
                    &mut surface,
                    &viewport,
                    None,
                    Color::WHITE,
                );
                render_loop.dispatch(
                    Event::WindowEvent(&WindowEvent::RedrawRequested),
                    &|command, id, component_index| {
                        run_command(context, command, id, component_index)
                    },
                );
            }
            UIEvent::WindowEvent(_, WindowEvent::Resized(size)) => {
                if window_container.resize_viewport(size) {
                    let viewport = window_container.viewport();
                    renderer.configure_surface(&mut surface, &viewport);
                    if render_loop.schedule_update(NodeId::ROOT, 0) {
                        context.request_idle(|deadline| InternalMessage::RequestRender(deadline));
                    }
                }
                render_loop.dispatch(
                    Event::WindowEvent(&WindowEvent::Resized(size)),
                    &|command, id, component_index| {
                        run_command(context, command, id, component_index)
                    },
                );
            }
            UIEvent::WindowEvent(_, WindowEvent::Closed) => {
                render_loop.dispatch(
                    Event::WindowEvent(&WindowEvent::Closed),
                    &|command, id, component_index| {
                        run_command(context, command, id, component_index)
                    },
                );
                return ControlFlow::Break;
            }
            UIEvent::WindowEvent(_, event) => {
                render_loop.dispatch(
                    Event::WindowEvent(&event),
                    &|command, id, component_index| {
                        run_command(context, command, id, component_index)
                    },
                );
            }
        }

        ControlFlow::Continue
    })?;

    Ok(())
}

fn run_command<Message, Context>(
    context: &Context,
    command: Command<Message>,
    id: NodeId,
    component_index: ComponentIndex,
) where
    Message: 'static,
    Context: EventLoopContext<InternalMessage<Message>>,
{
    match command {
        Command::QuitApplication => context.send(InternalMessage::Quit),
        Command::RequestUpdate => context.send(InternalMessage::RequestUpdate(id, component_index)),
        Command::Send(message) => context.send(InternalMessage::Broadcast(message)),
        Command::Perform(future) => {
            context.perform(future.map(InternalMessage::Broadcast));
        }
        Command::Delay(duration, callback) => {
            context.delay(duration, || InternalMessage::Broadcast(callback()));
        }
        Command::RequestAnimationFrame(callback) => {
            context.request_animation_frame(|| InternalMessage::Broadcast(callback()));
        }
        Command::RequestIdle(callback) => {
            context.request_idle(|deadline| InternalMessage::Broadcast(callback(deadline)));
        }
    }
}
