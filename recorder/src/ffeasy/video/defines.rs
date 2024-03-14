
pub struct YuvColor {
    pub y: u8,
    pub u: u8,
    pub v: u8,
}

impl YuvColor {
    // https://learn.microsoft.com/en-us/windows/win32/medfound/about-yuv-video
    pub const BLACK: Self = Self {
        y: 16,
        u: 128,
        v: 128,
    };

    pub const RED: Self = Self {
        y: 81,
        u: 90,
        v: 240,
    };

    pub const GREEN: Self = Self {
        y: 145,
        u: 54,
        v: 34,
    };

    pub const BLUE: Self = Self {
        y: 41,
        u: 240,
        v: 110,
    };
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VideoSize {
    pub width: u32,
    pub height: u32,
}

impl VideoSize {
    pub fn new(w: u32, h: u32) -> Self {
        Self { width: w, height: h, }
    }

    pub fn scale_fit(&self, limit: &Self) -> (Point, VideoSize) {
        scale_fit(self.width, self.height, limit.width, limit.height)
    }
}

pub fn scale_fit(src_width: u32, src_height: u32, limit_width: u32, limit_height: u32) -> (Point, VideoSize) {
    let dst_height = limit_width * src_height / src_width;
    if dst_height <= limit_height {
        (
            Point::new(0, (limit_height - dst_height)/2),
            VideoSize::new(limit_width, dst_height),
        )
    } else {
        let dst_width = limit_height * src_width / src_height;
        (
            Point::new((limit_width - dst_width)/2, 0),
            VideoSize::new(dst_width, limit_height),
        )
    }
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y, }
    }

    pub fn add(&self, delta: &Self) -> Self {
        Self {
            x: self.x + delta.x,
            y: self.y + delta.y,
        }
    }
}
