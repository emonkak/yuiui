use std::any::TypeId;
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use crate::event::window::WindowResize;
use crate::event::EventType;
use crate::graphics::color::Color;
use crate::graphics::renderer::Renderer;
use crate::graphics::viewport::Viewport;
use crate::paint::tree::PaintTree;
use crate::render::tree::RenderTree;
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
    let (patch_senter, patch_receiver) = sync_channel(1);
    let (update_sender, update_receiver) = channel();

    let proxy = event_loop.create_proxy();
    let window_id = window.window_id();

    thread::spawn(move || {
        let mut render_tree = RenderTree::new();

        let patches = render_tree.render(element);
        patch_senter.send((render_tree.root_id(), patches)).unwrap();
        proxy.request_redraw(window_id);

        loop {
            let target_id = update_receiver.recv().unwrap();
            let patches = render_tree.update(target_id);
            patch_senter.send((target_id, patches)).unwrap();
            proxy.request_redraw(window_id);
        }
    });

    let mut viewport = Viewport::new(window.get_bounds().size(), 1.0);
    let mut paint_tree = PaintTree::new(viewport.logical_size());
    let mut draw_area = renderer.create_draw_area(&viewport);
    let mut pipeline = renderer.create_pipeline(&viewport);

    event_loop.run(|event| {
        match event {
            Event::RedrawRequested(_) => {
                if let Some((node_id, patches)) = patch_receiver.try_recv().ok() {
                    for patch in patches {
                        paint_tree.apply_patch(patch);
                    }

                    paint_tree.layout_subtree(node_id, &mut renderer);

                    pipeline = renderer.create_pipeline(&viewport);
                    paint_tree.paint(&mut pipeline, &mut renderer);
                }

                renderer.perform_pipeline(&mut draw_area, &pipeline, &viewport, Color::WHITE);
            }
            Event::WindowEvent(_, window_event) => {
                if window_event.type_id == TypeId::of::<WindowResize>() {
                    let resize_event = WindowResize::downcast(&window_event).unwrap();

                    viewport = Viewport::new(resize_event.size, 1.0);
                    paint_tree.layout_root(viewport.logical_size(), &mut renderer);

                    draw_area = renderer.create_draw_area(&viewport);
                    pipeline = renderer.create_pipeline(&viewport);
                    paint_tree.paint(&mut pipeline, &mut renderer);
                }

                paint_tree.dispatch(&window_event, &update_sender);
            }
            _ => {}
        }

        ControlFlow::Wait
    });
}
