use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use crate::geometrics::{Point, Rectangle, Size};
use crate::paint::tree::PaintTree;
use crate::render::tree::RenderTree;
use crate::widget::element::Element;
use crate::platform::{Message, Backend};

pub fn run<Backend, Painter>(mut backend: Backend, element: Element<Painter>)
where
    Backend: self::Backend<Painter> + 'static,
    Painter: 'static,
{
    let (patch_tx, patch_rx) = sync_channel(1);
    let (update_tx, update_rx) = channel();

    let notify_update = backend.create_notifier();

    thread::spawn(move || {
        let mut render_tree = RenderTree::new();

        let patch = render_tree.render(element);

        notify_update();

        patch_tx.send((render_tree.root_id(), patch)).unwrap();

        loop {
            let target_id = update_rx.recv().unwrap();

            let patch = render_tree.update(target_id);

            notify_update();

            patch_tx.send((target_id, patch)).unwrap();
        }
    });

    let mut paint_tree = PaintTree::new(update_tx);
    let mut painter = backend.create_painter();
    let mut window_size = backend.get_window_size();

    backend.initialize();

    loop {
        match backend.advance_event_loop() {
            Message::Invalidate => {
                backend.commit_paint(&mut painter, &Rectangle {
                    point: Point::ZERO,
                    size: Size {
                        width: window_size.0 as _,
                        height: window_size.1 as _,
                    },
                });
            }
            Message::Resize(size) => {
                if window_size != size {
                    window_size = size;

                    paint_tree.layout(
                        Size {
                            width: window_size.0 as _,
                            height: window_size.1 as _,
                        },
                        &mut painter,
                    );

                    painter = backend.create_painter();

                    paint_tree.paint(&mut painter);
                }
            }
            Message::Update => {
                let (node_id, patches) = patch_rx.recv().unwrap();

                for patch in patches {
                    paint_tree.apply_patch(patch);
                }

                paint_tree.layout_subtree(
                    node_id,
                    Size {
                        width: window_size.0 as _,
                        height: window_size.1 as _,
                    },
                    &mut painter,
                );

                paint_tree.paint(&mut painter);

                backend.commit_paint(&mut painter, &Rectangle {
                    point: Point::ZERO,
                    size: Size {
                        width: window_size.0 as _,
                        height: window_size.1 as _,
                    },
                });
            }
            Message::Event(event) => {
                paint_tree.dispatch(&event);
            }
            Message::Quit => {
                break;
            }
        }
    }
}
