use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::randr::{self, ConnectionExt as _};
use x11rb::protocol::xproto;

pub fn refresh_rate(
    connection: &impl Connection,
    window: xproto::Window,
) -> Result<Option<f64>, ReplyError> {
    let screen_resources = connection.randr_get_screen_resources(window)?.reply()?;

    Ok(screen_resources
        .modes
        .first()
        .and_then(|mode_info| mode_refresh(mode_info)))
}

// See: https://gitlab.freedesktop.org/xorg/app/xrandr/-/blob/xrandr-1.5.1/xrandr.c#L576
fn mode_refresh(mode_info: &randr::ModeInfo) -> Option<f64> {
    let flags = mode_info.mode_flags;
    let vtotal = {
        let mut vtotal = mode_info.vtotal;
        if (flags & u32::from(randr::ModeFlag::DOUBLE_SCAN)) != 0 {
            vtotal *= 2;
        }
        if (flags & u32::from(randr::ModeFlag::INTERLACE)) != 0 {
            vtotal /= 2;
        }
        vtotal
    };

    if vtotal != 0 && mode_info.htotal != 0 {
        Some((mode_info.dot_clock as f64) / (vtotal as f64 * mode_info.htotal as f64))
    } else {
        None
    }
}
