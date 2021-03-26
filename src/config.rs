pub struct Config {
    pub border_color: String,
    pub border_width: u32,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u32,
    pub normal_background: String,
    pub normal_foreground: String,
    pub selected_background: String,
    pub selected_foreground: String,
    pub window_width: u32,
}

impl Config {
    pub fn parse(_args: Vec<String>) -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            border_color: "#869096".to_string(),
            border_width: 2,
            font_family: "Sans".to_string(),
            font_size: 12.0,
            font_weight: 300,
            normal_background: "#21272b".to_string(),
            normal_foreground: "#e8eaeb".to_string(),
            selected_background: "#21272b".to_string(),
            selected_foreground: "#e8eaeb".to_string(),
            window_width: 300,
        }
    }
}
