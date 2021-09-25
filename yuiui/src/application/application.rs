use std::any::Any;
use std::error;
use std::time::{Duration, Instant};
use yuiui_support::slot_tree::NodeId;

use crate::event::{Event, WindowEvent};
use crate::graphics::{Color, Primitive, Renderer};
use crate::ui::{ControlFlow, EventLoop, EventLoopContext, Window, WindowContainer};
use crate::widget::{Element, WidgetStorage};
use super::render_loop::{RenderLoop, RenderFlow};

#[derive(Debug)]
pub enum Message {
    Render(Instant),
    Broadcast(Box<dyn Any>),
}

pub fn run<Window, EventLoop, Renderer>(
    mut window_container: WindowContainer<Window>,
    mut event_loop: EventLoop,
    mut renderer: Renderer,
    element: Element<Message>,
) -> Result<(), Box<dyn error::Error>>
where
    Window: 'static + self::Window,
    EventLoop: 'static + self::EventLoop<Message, WindowId = Window::Id>,
    Renderer: 'static + self::Renderer,
{
    let viewport = window_container.viewport();
    let storage = WidgetStorage::new(element, viewport.clone());

    let mut render_loop = RenderLoop::new(storage);
    let mut pipeline = renderer.create_pipeline(Primitive::None);
    let mut surface = renderer.create_surface(viewport);

    event_loop.run(|event, context| {
        match event {
            Event::LoopInitialized => {
                context.request_idle(|deadline| Message::Render(deadline));
            }
            Event::Message(Message::Render(deadline)) => loop {
                match render_loop.render() {
                    RenderFlow::Continue => {
                        context.request_idle(|deadline| Message::Render(deadline));
                    }
                    RenderFlow::Commit(primitive, _bounds) => {
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
                if deadline - Instant::now() < Duration::from_secs(1) {
                    break;
                }
            },
            Event::WindowEvent(_, WindowEvent::RedrawRequested(_bounds)) => {
                let viewport = window_container.viewport();
                renderer.perform_pipeline(&mut pipeline, &mut surface, &viewport, Color::WHITE);
            }
            Event::WindowEvent(_, WindowEvent::SizeChanged(size)) => {
                if window_container.resize(size) {
                    let viewport = window_container.viewport();
                    renderer.configure_surface(&mut surface, &viewport);
                    render_loop.schedule_update(NodeId::ROOT, 0);
                    context.request_idle(|deadline| Message::Render(deadline));
                }
            }
            Event::WindowEvent(_, WindowEvent::Closed) => {
                return ControlFlow::Break;
            }
            _ => {}
        }

        ControlFlow::Continue
    })?;

    Ok(())
}
