use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use crate::event::GenericEvent;
use crate::geometrics::WindowSize;
use crate::paint::tree::PaintTree;
use crate::render::tree::RenderTree;
use crate::tree::NodeId;
use crate::widget::element::Element;

pub enum Message {
    Invalidate,
    Resize(WindowSize),
    Update(NodeId),
    Event(GenericEvent),
    Quit,
}

pub trait Backend<Painter> {
    fn begin_paint(&mut self, window_size: WindowSize) -> Painter;

    fn commit_paint(&mut self, painter: &mut Painter);

    fn advance_event_loop(&mut self) -> Message;

    fn create_notifier(&mut self) -> Box<dyn Fn(NodeId) + Send>;

    fn get_window_size(&self) -> WindowSize;

    fn run(&mut self, element: Element<Painter>)
    where
        Painter: 'static,
    {
        let (patches_tx, patches_rx) = sync_channel(1);
        let (update_tx, update_rx) = channel();
        let notify_update = self.create_notifier();

        thread::spawn(move || {
            let mut render_tree = RenderTree::new();

            let patches = render_tree.render(element);
            patches_tx.send(patches).unwrap();
            notify_update(render_tree.root_id());

            loop {
                let target_id = update_rx.recv().unwrap();
                let patches = render_tree.update(target_id);
                patches_tx.send(patches).unwrap();
                notify_update(target_id);
            }
        });

        let mut window_size = self.get_window_size();
        let mut paint_tree = PaintTree::new(window_size);
        let mut painter = self.begin_paint(window_size);

        loop {
            match self.advance_event_loop() {
                Message::Invalidate => {
                    self.commit_paint(&mut painter);
                }
                Message::Resize(size) => {
                    if window_size != size {
                        window_size = size;
                        painter = self.begin_paint(window_size);
                        paint_tree.layout_root(window_size, &mut painter);
                        paint_tree.paint(&mut painter);
                    }
                }
                Message::Update(node_id) => {
                    for patch in patches_rx.recv().unwrap() {
                        paint_tree.apply_patch(patch);
                    }

                    paint_tree.layout_subtree(node_id, &mut painter);
                    paint_tree.paint(&mut painter);

                    self.commit_paint(&mut painter);
                }
                Message::Event(event) => {
                    paint_tree.dispatch(&event, &update_tx);
                }
                Message::Quit => {
                    break;
                }
            }
        }
    }
}
