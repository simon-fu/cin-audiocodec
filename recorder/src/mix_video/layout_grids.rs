

use ffmpeg_next as ff;
use crate::ffeasy::video::{image::FFYuvImage, scaler::FFAutoScaler, Point, VideoSize, YuvColor};

use super::{LayoutOp, VChFlags, VChId, VChannels};

type Result<T> = std::result::Result<T, ff::Error>;

pub struct LayoutGrids {
    size: VideoSize,
    unit: VideoSize,
    grids: Vec<Grid>,
    image: FFYuvImage,
}

impl LayoutGrids {
    pub fn new(size: VideoSize) -> Self {
        Self {
            size,
            unit: size,
            grids: Default::default(),
            image: FFYuvImage::new(size),
        }
    }

    pub fn grow_grids(&mut self, extra: usize) -> Result<()> {
        
        let (rows, cols) = calc_rows_cols((self.grids.len() + extra) as u32);

        let new_grids = (rows * cols) as usize;

        self.unit = VideoSize {
            width: self.size.width / cols,
            height: self.size.height / rows,
        };

        self.grids.resize_with(new_grids, || {
            Grid {
                at: Point::new(0, 0),
                ch: None
            }
        });


        for (index, grid) in self.grids.iter_mut().enumerate() {
            let num = index as u32;
            let row = num / cols;
            let col = num % cols;
            
            grid.at = pos_at(&self.unit, row, col);

            println!("aaa grid.at {:?}, num {num}, row {row}, col {col}", grid.at);

            if let Some(ch) = &mut grid.ch {
                ch.scaler.change_output(FFYuvImage::FORMAT, self.unit)?;
            }
        }
        
        Ok(())
    }

}

impl LayoutOp for LayoutGrids {
    fn add_ch(&mut self, ch_id: VChId, flags: VChFlags) -> Result<()> {
        
        let r = self.grids.iter_mut().find(|x|x.ch.is_none());

        if let Some(grid) = r {
            grid.ch = Some(GridCh {
                ch_id,
                scaler: FFAutoScaler::new(FFYuvImage::FORMAT, self.unit)?,
            });
            return Ok(())
        }

        self.grow_grids(1)?;

        self.add_ch(ch_id, flags)
    }

    fn remove_ch(&mut self, ch_id: &VChId) -> Result<()> {

        let pos = self.grids.iter().position(|x|{
            match &x.ch {
                Some(ch) => ch.ch_id == *ch_id,
                None => false,
            }
        });

        if let Some(pos) = pos {
            // let ch = self.grids[pos].ch.take();
            for n in pos..self.grids.len()-1 {
                self.grids[n].ch = self.grids[n+1].ch.take();
            }
        }
        Ok(())
    }

    fn update_ch(&mut self, _ch_id: &VChId, _image: &FFYuvImage) -> Result<()> {
        Ok(())
    }

    fn get_output(&mut self, channels: &VChannels) -> Result<&FFYuvImage> {
        
        self.image.fill_color(&YuvColor::BLACK);

        for grid in self.grids.iter_mut() {
            if let Some(grid_ch) = &mut grid.ch {
                if let Some(ch) = channels.get(&grid_ch.ch_id) {
                    if let Some(image) = &ch.image {
                        let dst = grid_ch.scaler.scale(image)?;
                        println!("aaa draw {:?}, {:?}, {}x{}", grid_ch.ch_id, grid.at, dst.frame().width(), dst.frame().height());
                        self.image.draw_image(grid.at, &dst);
                    }
                }
            }
        }

        Ok(&self.image)
    }
}

// #[derive(Clone)]
struct Grid {
    at: Point,
    // size: VideoSize,
    ch: Option<GridCh>,
}


struct GridCh {
    ch_id: VChId,
    scaler: FFAutoScaler,
}

fn calc_rows_cols(num: u32) -> (u32, u32) {
    if num == 0 {
        (1, 1)
    } else if num == 1 {
        (1, 1)
    } else {
        for n in 2..num {
            if n * n >= num {
                return ((num + n-1) / n, n)
            }
        }
        (1, num)
    }
}

fn pos_at(unit: &VideoSize, row: u32, col: u32) -> Point {
    Point {
        x: col * unit.width,
        y: row * unit.height,
    }
}
