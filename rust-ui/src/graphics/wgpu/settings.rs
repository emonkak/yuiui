#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub present_mode: wgpu::PresentMode,
    pub internal_backend: wgpu::BackendBit,
    pub default_font: Option<&'static [u8]>,
    pub default_text_size: u16,
    pub antialiasing: Option<Antialiasing>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Antialiasing {
    MSAAx2,
    MSAAx4,
    MSAAx8,
    MSAAx16,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            present_mode: wgpu::PresentMode::Mailbox,
            internal_backend: wgpu::BackendBit::PRIMARY,
            default_font: None,
            default_text_size: 20,
            antialiasing: None,
        }
    }
}

impl Antialiasing {
    pub fn sample_count(self) -> u32 {
        match self {
            Antialiasing::MSAAx2 => 2,
            Antialiasing::MSAAx4 => 4,
            Antialiasing::MSAAx8 => 8,
            Antialiasing::MSAAx16 => 16,
        }
    }
}
