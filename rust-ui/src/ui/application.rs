use std::any::Any;
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use crate::event::{GenericEvent, WindowEvent};
use crate::graphics::{Color, Renderer, Viewport};
use crate::paint::PaintTree;
use crate::render::RenderTree;
use crate::widget::element::Element;

use super::event_loop::{ControlFlow, Event, EventLoop, EventLoopProxy};
use super::window::Window;

pub fn run<Window, EventLoop, Renderer>(
    mut event_loop: EventLoop,
    mut renderer: Renderer,
    window: Window,
    element: Element<Renderer>,
) where
    Window: self::Window + 'static,
    EventLoop: self::EventLoop<WindowId = Window::WindowId> + 'static,
    Renderer: self::Renderer + 'static,
{
    let (update_senter, update_receiver) = sync_channel(1);
    let (message_sender, message_receiver) = channel();

    let proxy = event_loop.create_proxy();
    let window_id = window.window_id();

    {
        let message_sender = message_sender.clone();

        thread::spawn(move || {
            let mut render_tree = RenderTree::new(message_sender);

            let patches = render_tree.render(element);
            update_senter
                .send((render_tree.root_id(), patches))
                .unwrap();
            proxy.request_redraw(window_id);

            loop {
                let (target_id, version, messsge) = message_receiver.recv().unwrap();
                let patches = render_tree.update(target_id, version, messsge);
                if !patches.is_empty() {
                    update_senter.send((target_id, patches)).unwrap();
                    proxy.request_redraw(window_id);
                }
            }
        });
    }

    let mut viewport = window.get_viewport();
    let mut paint_tree = PaintTree::new(viewport.logical_size(), message_sender);
    let mut surface = renderer.create_surface(&viewport);
    let mut pipeline = renderer.create_pipeline(&viewport);

    event_loop.run(|event| {
        match event {
            Event::RedrawRequested(_) => {
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
            Event::WindowEvent(_, window_event) => {
                match window_event {
                    WindowEvent::WindowResize(size) => {
                        viewport = Viewport::from_physical(size, viewport.scale_factor());
                        paint_tree.layout_root(viewport.logical_size(), &mut renderer);

                        renderer.configure_surface(&mut surface, &viewport);
                        pipeline = renderer.create_pipeline(&viewport);
                        paint_tree.paint(&mut pipeline, &mut renderer);
                    }
                    _ => {}
                }

                let generic_event = GenericEvent {
                    type_id: window_event.type_id(),
                    payload: Box::new(window_event),
                };

                paint_tree.dispatch_event(&generic_event);
            }
            _ => {}
        }

        ControlFlow::Wait
    });
}
