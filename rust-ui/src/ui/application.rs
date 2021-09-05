use std::any::Any;
use std::error;
use std::collections::VecDeque;

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
    mut window_container: WindowContainer<Window>,
    element: Element<Renderer>,
) -> Result<(), Box<dyn error::Error>>
where
    Window: 'static + self::Window,
    EventLoop: 'static + self::EventLoop<Box<dyn Any + Send>, WindowId = Window::Id>,
    Renderer: 'static + self::Renderer,
{
    let viewport = window_container.viewport();

    let mut update_queue = VecDeque::new();

    let mut paint_tree = PaintTree::new(viewport.logical_size());
    let mut render_tree = RenderTree::new();
    let mut surface = renderer.create_surface(viewport);
    let mut pipeline = renderer.create_pipeline(viewport);

    {
        let patches = render_tree.render(element);
        update_queue.push_back((render_tree.root_id(), patches));
    }

    event_loop.run(|event, _context| {
        match &event {
            Event::WindowEvent(_, WindowEvent::RedrawRequested(_)) => {
                let viewport = window_container.viewport();

                if let Some((element_id, patches)) = update_queue.pop_front() {
                    paint_tree.mark_update_root(element_id);

                    for patch in patches {
                        paint_tree.apply_patch(patch);
                    }

                    pipeline = renderer.create_pipeline(&viewport);

                    paint_tree.layout_subtree(element_id, &mut renderer);
                    paint_tree.paint(&mut pipeline, &mut renderer);
                }

                renderer.perform_pipeline(&mut surface, &mut pipeline, &viewport, Color::WHITE);
            }
            Event::WindowEvent(_, WindowEvent::SizeChanged(size)) => {
                if window_container.resize(*size) {
                    let viewport = window_container.viewport();

                    pipeline = renderer.create_pipeline(&viewport);
                    renderer.configure_surface(&mut surface, &viewport);

                    paint_tree.layout_root(viewport.logical_size(), &mut renderer);
                    paint_tree.paint(&mut pipeline, &mut renderer);
                }
            }
            Event::WindowEvent(_, WindowEvent::Closed) => {
                return ControlFlow::Exit;
            }
            _ => {}
        }

        ControlFlow::Continue
    })?;

    Ok(())
}
