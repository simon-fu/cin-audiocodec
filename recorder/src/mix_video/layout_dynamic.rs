

use ffmpeg_next as ff;
use crate::ffeasy::video::{image::FFYuvImage, YuvColor};

use super::{layout_big_left::LayoutBigLeft, layout_grids::LayoutGrids, LayoutOp, VChannels};

type Result<T> = std::result::Result<T, ff::Error>;


pub struct LayoutDynamic {
    layouts: Vec<Box<dyn LayoutOp>>,
    count: u64,
}

impl Default for LayoutDynamic {
    fn default() -> Self {
        Self { 
            layouts: vec![
                Box::new(LayoutGrids::default()),
                Box::new(LayoutBigLeft::default()),
            ],
            count: 0,
        }
    }
}

impl LayoutOp for LayoutDynamic {
    fn get_output(&mut self, channels: &mut VChannels, canvas: &mut FFYuvImage) -> Result<()> {
        
        canvas.fill_color(&YuvColor::BLACK);

        let index = (self.count / 50) % self.layouts.len() as u64;
        self.layouts[index as usize].get_output(channels, canvas)?;
        self.count += 1;
        
        Ok(())
    }
}




