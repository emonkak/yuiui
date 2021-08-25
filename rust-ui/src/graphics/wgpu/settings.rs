use crate::text::FontDescriptor;

#[derive(Debug)]
pub struct Settings {
    pub present_mode: wgpu::PresentMode,
    pub internal_backend: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub default_font: FontDescriptor,
    pub text_multithreading: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            present_mode: wgpu::PresentMode::Mailbox,
            internal_backend: wgpu::Backends::PRIMARY,
            power_preference: wgpu::PowerPreference::LowPower,
            default_font: FontDescriptor::default(),
            text_multithreading: false,
        }
    }
}
