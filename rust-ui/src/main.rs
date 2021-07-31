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

use rust_ui::event::handler::EventContext;
use rust_ui::event::mouse::{MouseDown, MouseEvent};
use rust_ui::geometrics::{Point, Rectangle, Size};
use rust_ui::paint::{PaintContext, PaintTree};
use rust_ui::platform::x11::event::XEvent;
use rust_ui::platform::x11::paint::XPaintContext;
use rust_ui::platform::x11::window::{self, XWindowHandle};
use rust_ui::platform::WindowHandle;
use rust_ui::render::{RenderContext, RenderTree};
use rust_ui::tree::NodeId;
use rust_ui::widget::element::Children;
use rust_ui::widget::element::Element;
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::{Flex, FlexItem};
use rust_ui::widget::padding::Padding;
use rust_ui::widget::subscriber::Subscriber;
use rust_ui::widget::tree::Patch;
use rust_ui::widget::{Widget, WidgetMeta};

struct App;

impl App {
    fn on_click(&self, _event: &MouseEvent, state: &mut bool, context: &mut EventContext) {
        *state = !*state;

        context.notify_changes();
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
) -> (Sender<NodeId>, Receiver<Vec<Patch<XWindowHandle>>>) {
    let (patch_tx, patch_rx) = sync_channel(1);
    let (update_tx, update_rx) = channel();

    thread::spawn(move || {
        let mut render_tree = RenderTree::new();

        let patch = render_tree.render(element);

        unsafe {
            notify_update(*display.get_mut(), window, update_atom);
        }

        patch_tx.send(patch).unwrap();

        loop {
            let target_id = update_rx.recv().unwrap();

            let patch = render_tree.update(target_id);

            println!("RENDER TREE:\n{}", render_tree);

            unsafe {
                notify_update(*display.get_mut(), window, update_atom);
            }

            patch_tx.send(patch).unwrap();
        }
    });

    (update_tx, patch_rx)
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
    let mut paint_tree: PaintTree<XWindowHandle> = PaintTree::new(tx);
    let mut paint_context = XPaintContext::new(&handle);

    handle.show_window();

    unsafe {
        xlib::XFlush(handle.display());
    }

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
                XEvent::ButtonRelease(event) => paint_tree.dispatch::<MouseDown>((&event).into()),
                XEvent::ConfigureNotify(event) => {
                    if window_width != event.width as _ || window_height != event.height as _ {
                        window_width = event.width as _;
                        window_height = event.height as _;

                        paint_tree.layout(
                            Size {
                                width: window_width as _,
                                height: window_height as _,
                            },
                            true,
                        );

                        paint_context = XPaintContext::new(&handle);

                        paint_tree.paint(&mut paint_context);
                    }
                }
                XEvent::ClientMessage(event) if event.message_type == update_atom => {
                    let patches = rx.recv().unwrap();

                    for patch in patches {
                        paint_tree.apply_patch(patch);
                    }

                    paint_tree.layout(
                        Size {
                            width: window_width as _,
                            height: window_height as _,
                        },
                        true,
                    );

                    paint_tree.paint(&mut paint_context);

                    println!("PAINT TREE:\n{}", paint_tree);

                    paint_context.commit(&Rectangle {
                        point: Point::ZERO,
                        size: Size {
                            width: window_width as _,
                            height: window_height as _,
                        },
                    });
                }
                _ => (),
            }
        }
    }
}
