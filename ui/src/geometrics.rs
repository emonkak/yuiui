#[derive(Debug, Clone, Copy)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Rectangle {
    pub point: Point,
    pub size: Size,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl BoxConstraints {
    pub fn tight(size: &Size) -> BoxConstraints {
        BoxConstraints {
            min: *size,
            max: *size,
        }
    }

    pub fn constrain(&self, size: &Size) -> Size {
        Size {
            width: size.width.clamp(self.min.width, self.max.width),
            height: size.height.clamp(self.min.height, self.max.height),
        }
    }
}

impl Point {
    pub fn offset(&self, offset: Point) -> Point {
        Point {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }
}
