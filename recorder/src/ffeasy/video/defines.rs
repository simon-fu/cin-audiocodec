
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


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y, }
    }
}
