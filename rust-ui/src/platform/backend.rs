use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use mio::{Events, Poll, Token, Waker};

use crate::event::GenericEvent;
use crate::geometrics::WindowSize;
use crate::paint::tree::PaintTree;
use crate::render::tree::RenderTree;
use crate::widget::element::Element;

const WINDOE_EVENT_TOKEN: Token = Token(0);
const REDRAW_TOKEN: Token = Token(1);

pub enum Message {
    Invalidate,
    Resize(WindowSize),
    Event(GenericEvent),
    Quit,
}

pub trait Backend<Painter> {
    fn begin_paint(&mut self, window_size: WindowSize) -> Painter;

    fn commit_paint(&mut self, painter: &mut Painter);

    fn invalidate(&self);

    fn advance_event_loop(&mut self) -> Message;

    fn get_window_size(&self) -> WindowSize;

    fn subscribe_window_events(&self, poll: &Poll, token: Token);

    fn run(&mut self, element: Element<Painter>)
    where
        Painter: 'static,
    {
        let mut poll = Poll::new().unwrap();
        let waker = Waker::new(poll.registry(), REDRAW_TOKEN).unwrap();

        self.subscribe_window_events(&poll, WINDOE_EVENT_TOKEN);

        let (update_senter, update_receiver) = sync_channel(1);
        let (redraw_sender, redraw_receiver) = channel();

        thread::spawn(move || {
            let mut render_tree = RenderTree::new();

            let patches = render_tree.render(element);
            update_senter.send((render_tree.root_id(), patches)).unwrap();
            waker.wake().unwrap();

            loop {
                let target_id = redraw_receiver.recv().unwrap();
                let patches = render_tree.update(target_id);
                update_senter.send((target_id, patches)).unwrap();
                waker.wake().unwrap();
            }
        });

        let mut window_size = self.get_window_size();
        let mut paint_tree = PaintTree::new(window_size);
        let mut painter = self.begin_paint(window_size);

        let mut events = Events::with_capacity(8);

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in &events {
                match event.token() {
                    WINDOE_EVENT_TOKEN => {
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
                                    self.invalidate();
                                }
                            }
                            Message::Event(event) => {
                                paint_tree.dispatch(&event, &redraw_sender);
                            }
                            Message::Quit => {
                                break;
                            }
                        }
                    },
                    REDRAW_TOKEN => {
                        let (node_id, patches) = update_receiver.recv().unwrap();

                        for patch in patches {
                            paint_tree.apply_patch(patch);
                        }

                        paint_tree.layout_subtree(node_id, &mut painter);
                        paint_tree.paint(&mut painter);

                        self.commit_paint(&mut painter);
                    }
                    _ => {}
                }
            }
        }
    }
}
