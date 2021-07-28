#[macro_use]
extern crate rust_ui;
extern crate x11;

use std::any::Any;
use std::env;
use std::mem;
use std::ptr;
use x11::xlib;

use rust_ui::event::mouse::{MouseDown, MouseEvent};
use rust_ui::event::EventContext;
use rust_ui::geometrics::{Point, Rectangle, Size};
use rust_ui::paint::PaintContext;
use rust_ui::platform::x11::event::XEvent;
use rust_ui::platform::x11::paint::XPaintContext;
use rust_ui::platform::x11::window::{self, XWindowHandle};
use rust_ui::platform::WindowHandle;
use rust_ui::render::RenderContext;
use rust_ui::updater::Updater;
use rust_ui::widget::element::Element;
use rust_ui::widget::element::{Child, Children};
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::{Flex, FlexItem};
use rust_ui::widget::padding::Padding;
use rust_ui::widget::subscriber::Subscriber;
use rust_ui::widget::{Widget, WidgetMeta};

struct App;

impl App {
    fn on_click(&self, event: &MouseEvent, _state: &mut bool, _context: &mut EventContext) {
        println!("on_click: {:?}", event)
    }
}

impl<Handle: 'static> Widget<Handle> for App {
    type State = bool;

    fn render(
        &self,
        _children: Children<Handle>,
        _state: &Self::State,
        context: &RenderContext<Self, Handle, Self::State>,
    ) -> Child<Handle> {
        element!(
            Padding::uniform(32.0) => {
                Flex::row() => {
                    FlexItem::new(1.0).with_key(1) => {
                        Subscriber::new().on(context.use_handler(MouseDown, Self::on_click)) => {
                            Fill::new(0xff0000ff)
                        }
                    }
                    FlexItem::new(1.0).with_key(2) => { Fill::new(0x0000ffff) }
                    FlexItem::new(1.0).with_key(3) => { Fill::new(0x00ff00ff) }
                    FlexItem::new(1.0).with_key(4) => { Fill::new(0x00ffffff) }
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

fn render() -> Element<XWindowHandle> {
    element!(App)
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

    let mut updater: Updater<XWindowHandle> = Updater::new();

    updater.update(render());
    updater.render();
    updater.layout(
        Size {
            width: window_width as _,
            height: window_height as _,
        },
        false,
    );

    let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let mut paint_context = XPaintContext::new(handle.clone());

    updater.paint(&mut paint_context);

    handle.show_window();

    println!("{}", updater);

    unsafe {
        xlib::XFlush(handle.display);
    }

    loop {
        unsafe {
            xlib::XNextEvent(handle.display, &mut event);
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
                    updater.dispatch_events::<MouseDown>((&event).into())
                }
                XEvent::ConfigureNotify(event) => {
                    if window_width != event.width as _ || window_height != event.height as _ {
                        window_width = event.width as _;
                        window_height = event.height as _;

                        updater.layout(
                            Size {
                                width: window_width as _,
                                height: window_height as _,
                            },
                            true,
                        );

                        paint_context = XPaintContext::new(handle.clone());
                        updater.paint(&mut paint_context);

                        paint_context.commit(&Rectangle {
                            point: Point::ZERO,
                            size: Size {
                                width: window_width as _,
                                height: window_height as _,
                            },
                        });
                    }
                }
                _ => (),
            }
        }
    }
}
