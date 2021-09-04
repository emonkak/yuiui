use std::any::Any;
use std::error;
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use crate::graphics::{Color, Renderer};
use crate::paint::PaintTree;
use crate::render::RenderTree;
use crate::widget::element::Element;

use super::event::{Event, WindowEvent};
use super::event_loop::{ControlFlow, EventLoop};
use super::window::{Window, WindowContainer};

pub fn run<Window, EventLoop, Renderer>(
    mut event_loop: EventLoop,
    mut renderer: Renderer,
    mut window: WindowContainer<Window>,
    element: Element<Renderer>,
) -> Result<(), Box<dyn error::Error>>
where
    Window: 'static + self::Window,
    EventLoop: 'static + self::EventLoop<Box<dyn Any + Send>, WindowId = Window::Id>,
    Renderer: 'static + self::Renderer,
{
    let (update_senter, update_receiver) = sync_channel(1);
    let (message_sender, message_receiver) = channel();

    {
        let message_sender = message_sender.clone();

        thread::spawn(move || {
            let mut render_tree = RenderTree::new(message_sender);

            let patches = render_tree.render(element);
            update_senter
                .send((render_tree.root_id(), patches))
                .unwrap();
            // proxy.request_redraw(window_id);

            loop {
                let mut patches = Vec::new();
                let message = message_receiver.recv().unwrap();
                render_tree.update(message, &mut patches);
                if !patches.is_empty() {
                    // TODO: partial update
                    update_senter
                        .send((render_tree.root_id(), patches))
                        .unwrap();
                    // proxy.request_redraw(window_id);
                }
            }
        });
    }

    let viewport = window.viewport();
    let mut paint_tree = PaintTree::new(viewport.logical_size(), message_sender);
    let mut surface = renderer.create_surface(viewport);
    let mut pipeline = renderer.create_pipeline(viewport);

    event_loop.run(|event, _context| {
        match &event {
            Event::WindowEvent(_, WindowEvent::RedrawRequested(_)) => {
                let viewport = window.viewport();

                if let Some((element_id, patches)) = update_receiver.try_recv().ok() {
                    paint_tree.mark_update_root(element_id);

                    for patch in patches {
                        paint_tree.apply_patch(patch);
                    }

                    paint_tree.layout_subtree(element_id, &mut renderer);

                    pipeline = renderer.create_pipeline(&viewport);
                    paint_tree.paint(&mut pipeline, &mut renderer);
                }

                renderer.perform_pipeline(&mut surface, &mut pipeline, &viewport, Color::WHITE);
            }
            Event::WindowEvent(_, WindowEvent::Closed) => {
                return ControlFlow::Exit;
            }
            Event::WindowEvent(_, WindowEvent::SizeChanged(size)) => {
                if window.resize(*size) {
                    let viewport = window.viewport();

                    paint_tree.layout_root(viewport.logical_size(), &mut renderer);
                    renderer.configure_surface(&mut surface, &viewport);
                    pipeline = renderer.create_pipeline(&viewport);
                    paint_tree.paint(&mut pipeline, &mut renderer);
                }
            }
            _ => {}
        }

        paint_tree.broadcast(event);

        ControlFlow::Continue
    })?;

    Ok(())
}
