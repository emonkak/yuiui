use gtk::{gdk, gio, glib, pango, prelude::*};
use yuiui::{ElementSeq, EventListener, Lifecycle, MessageContext, Store, View};
use yuiui_gtk_derive::WidgetBuilder;

#[derive(Clone, Debug, WidgetBuilder)]
#[widget(gtk::Label)]
pub struct Label {
    attributes: Option<pango::AttrList>,
    ellipsize: Option<pango::EllipsizeMode>,
    extra_menu: Option<gio::MenuModel>,
    justify: Option<gtk::Justification>,
    label: Option<String>,
    lines: Option<i32>,
    max_width_chars: Option<i32>,
    mnemonic_widget: Option<gtk::Widget>,
    selectable: Option<bool>,
    single_line_mode: Option<bool>,
    use_markup: Option<bool>,
    use_underline: Option<bool>,
    width_chars: Option<i32>,
    wrap: Option<bool>,
    wrap_mode: Option<pango::WrapMode>,
    xalign: Option<f32>,
    yalign: Option<f32>,
    can_focus: Option<bool>,
    can_target: Option<bool>,
    css_classes: Option<Vec<String>>,
    css_name: Option<String>,
    cursor: Option<gdk::Cursor>,
    focus_on_click: Option<bool>,
    focusable: Option<bool>,
    halign: Option<gtk::Align>,
    has_tooltip: Option<bool>,
    height_request: Option<i32>,
    hexpand: Option<bool>,
    hexpand_set: Option<bool>,
    layout_manager: Option<gtk::LayoutManager>,
    margin_bottom: Option<i32>,
    margin_end: Option<i32>,
    margin_start: Option<i32>,
    margin_top: Option<i32>,
    name: Option<String>,
    opacity: Option<f64>,
    overflow: Option<gtk::Overflow>,
    receives_default: Option<bool>,
    sensitive: Option<bool>,
    tooltip_markup: Option<String>,
    tooltip_text: Option<String>,
    valign: Option<gtk::Align>,
    vexpand: Option<bool>,
    vexpand_set: Option<bool>,
    visible: Option<bool>,
    width_request: Option<i32>,
    accessible_role: Option<gtk::AccessibleRole>,
}

impl<S, M, B> View<S, M, B> for Label {
    type Children = ();

    type State = gtk::Label;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut B,
    ) {
        match lifecycle {
            Lifecycle::Update(old_view) => {
                self.update(&old_view, state);
            }
            _ => {}
        }
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::State {
        self.build()
    }
}

impl<'event> EventListener<'event> for Label {
    type Event = ();
}
