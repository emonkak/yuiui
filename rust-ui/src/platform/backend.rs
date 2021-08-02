use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use crate::event::GenericEvent;
use crate::geometrics::WindowSize;
use crate::paint::tree::PaintTree;
use crate::render::tree::RenderTree;
use crate::widget::element::Element;

pub enum Message {
    Invalidate,
    Resize(WindowSize),
    Update,
    Event(GenericEvent),
    Quit,
}

pub trait Backend<Painter> {
    fn create_painter(&mut self, window_size: WindowSize) -> Painter;

    fn create_notifier(&mut self) -> Box<dyn Fn() + Send>;

    fn commit_paint(&mut self, painter: &mut Painter);

    fn advance_event_loop(&mut self) -> Message;

    fn run(&mut self, mut window_size: WindowSize, element: Element<Painter>)
    where
        Painter: 'static,
    {
        let (patch_tx, patch_rx) = sync_channel(1);
        let (update_tx, update_rx) = channel();

        let notify_update = self.create_notifier();

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
        let mut painter = self.create_painter(window_size);

        loop {
            match self.advance_event_loop() {
                Message::Invalidate => {
                    self.commit_paint(&mut painter);
                }
                Message::Resize(size) => {
                    if window_size != size {
                        window_size = size;

                        painter = self.create_painter(window_size);

                        paint_tree.layout((&window_size).into(), &mut painter);
                        paint_tree.paint(&mut painter);
                    }
                }
                Message::Update => {
                    let (node_id, patches) = patch_rx.recv().unwrap();

                    for patch in patches {
                        paint_tree.apply_patch(patch);
                    }

                    paint_tree.layout_subtree(node_id, (&window_size).into(), &mut painter);
                    paint_tree.paint(&mut painter);

                    self.commit_paint(&mut painter);
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
}
