#[macro_use]
extern crate rust_ui;
extern crate x11;

use std::any::Any;
use std::env;
use std::mem;
use std::ptr;
use std::sync::atomic::AtomicPtr;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender};
use std::thread;
use x11::xlib;

use rust_ui::event::mouse::{MouseDown, MouseEvent};
use rust_ui::event::handler::EventContext;
use rust_ui::geometrics::{Point, Rectangle, Size};
use rust_ui::painter::{PaintContext, Painter};
use rust_ui::platform::x11::event::XEvent;
use rust_ui::platform::x11::paint::XPaintContext;
use rust_ui::platform::x11::window::{self, XWindowHandle};
use rust_ui::platform::WindowHandle;
use rust_ui::renderer::{RenderContext, Renderer};
use rust_ui::tree::{NodeId, Tree};
use rust_ui::widget::element::Children;
use rust_ui::widget::element::Element;
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::{Flex, FlexItem};
use rust_ui::widget::null::Null;
use rust_ui::widget::padding::Padding;
use rust_ui::widget::subscriber::Subscriber;
use rust_ui::widget::{Widget, WidgetMeta, WidgetPod, WidgetTree};

struct App;

impl App {
    fn on_click(&self, event: &MouseEvent, state: &mut bool, context: &mut EventContext) {
        *state = !*state;

        context.notify_changes();

        println!("on_click: {:?}", event)
    }
}

impl<Handle: 'static> Widget<Handle> for App {
    type State = bool;

    fn render(
        &self,
        _children: Children<Handle>,
        state: &Self::State,
        context: &RenderContext<Self, Handle, Self::State>,
    ) -> Children<Handle> {
        println!("render: {:?}", state);
        element!(
            Subscriber::new().on(context.use_handler(MouseDown, Self::on_click)) => {
                Padding::uniform(32.0) => {
                    Flex::row() => {
                        if *state {
                            None
                        } else {
                            Some(element!(FlexItem::new(1.0).with_key(1) => Fill::new(0xff0000ff)))
                        },
                        FlexItem::new(1.0).with_key(2) => Fill::new(0x00ff00ff),
                        FlexItem::new(1.0).with_key(3) => Fill::new(0x0000ffff),
                    }
                }
            }
        )
        .into()
    }
}

impl WidgetMeta for App {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn run_render_loop(
    mut display: AtomicPtr<xlib::Display>,
    window: xlib::Window,
    update_atom: xlib::Atom,
    element: Element<XWindowHandle>,
) -> (
    Sender<NodeId>,
    Receiver<(NodeId, WidgetTree<XWindowHandle>)>,
) {
    let (tree_tx, tree_rx) = sync_channel(1);
    let (update_tx, update_rx) = channel();

    thread::spawn(move || {
        let mut renderer = Renderer::new();
        let (root_id, mut tree) = renderer.render(element);

        unsafe {
            notify_update(*display.get_mut(), window, update_atom);
        }

        tree_tx.send((root_id, tree.clone())).unwrap();

        loop {
            let target_id = update_rx.recv().unwrap();

            renderer.update(target_id, &mut tree);

            unsafe {
                notify_update(*display.get_mut(), window, update_atom);
            }

            tree_tx
                .send((root_id, tree.split_subtree(target_id)))
                .unwrap();
        }
    });

    (update_tx, tree_rx)
}

unsafe fn notify_update(
    display: *mut xlib::Display,
    window: xlib::Window,
    message_type: xlib::Atom,
) {
    let data = xlib::ClientMessageData::new();

    let mut event = xlib::XEvent::from(xlib::XClientMessageEvent {
        type_: xlib::ClientMessage,
        serial: 0,
        send_event: xlib::True,
        display,
        window,
        message_type,
        format: 32,
        data,
    });

    xlib::XSendEvent(display, window, xlib::True, xlib::NoEventMask, &mut event);
    xlib::XFlush(display);
}

fn main() {
    let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
    if display.is_null() {
        panic!(
            "No display found at {}",
            env::var("DISPLAY").unwrap_or_default()
        );
    }

    let mut window_width: u32 = 640;
    let mut window_height: u32 = 480;
    let window = unsafe { window::create_window(display, window_width, window_height) };

    unsafe {
        xlib::XSelectInput(
            display,
            window,
            xlib::ButtonPressMask
                | xlib::ButtonReleaseMask
                | xlib::ExposureMask
                | xlib::StructureNotifyMask,
        );
    }

    let handle = XWindowHandle::new(display, window);

    let update_atom = unsafe {
        xlib::XInternAtom(
            display,
            "__RUST_UI_UPDATE\0".as_ptr() as *const _,
            xlib::False,
        )
    };

    let (tx, rx) = run_render_loop(
        AtomicPtr::new(handle.display()),
        handle.window(),
        update_atom,
        element!(App),
    );

    let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let mut painter: Painter<XWindowHandle> = Painter::new(tx);
    let mut paint_context = XPaintContext::new(&handle);

    handle.show_window();

    unsafe {
        xlib::XFlush(handle.display());
    }

    let mut tree: Tree<WidgetPod<XWindowHandle>> = Tree::new();
    let root_id = tree.attach(WidgetPod::from(element!(Null)));

    loop {
        unsafe {
            xlib::XNextEvent(handle.display(), &mut event);
            match XEvent::from(&event) {
                XEvent::Expose(_) => {
                    paint_context.commit(&Rectangle {
                        point: Point::ZERO,
                        size: Size {
                            width: window_width as _,
                            height: window_height as _,
                        },
                    });
                }
                XEvent::ButtonRelease(event) => {
                    painter.dispatch::<MouseDown>((&event).into(), &tree)
                }
                XEvent::ConfigureNotify(event) => {
                    println!("resize: {} {} {} {}", window_width, event.width, window_height, event.height);
                    if window_width != event.width as _ || window_height != event.height as _ {
                        window_width = event.width as _;
                        window_height = event.height as _;

                        painter.layout(
                            root_id,
                            &tree,
                            Size {
                                width: window_width as _,
                                height: window_height as _,
                            },
                            true,
                        );

                        paint_context = XPaintContext::new(&handle);

                        painter.paint(root_id, &tree, &tree, &mut paint_context);
                    }
                }
                XEvent::ClientMessage(event) if event.message_type == update_atom => {
                    let (target_id, new_tree) = rx.recv().unwrap();

                    painter.layout(
                        target_id,
                        &new_tree,
                        Size {
                            width: window_width as _,
                            height: window_height as _,
                        },
                        true,
                    );

                    println!("{}", painter.format_tree(target_id, &new_tree));

                    painter.paint(target_id, &tree, &new_tree, &mut paint_context);

                    paint_context.commit(&Rectangle {
                        point: Point::ZERO,
                        size: Size {
                            width: window_width as _,
                            height: window_height as _,
                        },
                    });

                    tree = new_tree;
                }
                _ => (),
            }
        }
    }
}
