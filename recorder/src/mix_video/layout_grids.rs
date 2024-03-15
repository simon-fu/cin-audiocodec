

use ffmpeg_next as ff;
use crate::ffeasy::video::{image::FFYuvImage, scale_fit, Point, VideoSize, YuvColor};

use super::{LayoutOp, VChannels};

type Result<T> = std::result::Result<T, ff::Error>;

#[derive(Default)]
pub struct LayoutGrids {
    // size: VideoSize,
    // unit: VideoSize,
    // grids: Vec<Grid>,
    // // image: FFYuvImage,
}

impl LayoutGrids {
    // pub fn new(_size: VideoSize) -> Self {
    //     Self {
    //         // size,
    //         // unit: size,
    //         // grids: Default::default(),
    //         // // image: FFYuvImage::new(size),
    //     }
    // }

    // pub fn grow_grids(&mut self, extra: usize) -> Result<()> {
        
    //     let (rows, cols) = calc_rows_cols((self.grids.len() + extra) as u32);

    //     let new_grids = (rows * cols) as usize;

    //     self.unit = VideoSize {
    //         width: self.size.width / cols,
    //         height: self.size.height / rows,
    //     };

    //     self.grids.resize_with(new_grids, || {
    //         Grid {
    //             at: Point::new(0, 0),
    //             ch: None
    //         }
    //     });


    //     for (index, grid) in self.grids.iter_mut().enumerate() {
    //         let num = index as u32;
    //         let row = num / cols;
    //         let col = num % cols;
            
    //         grid.at = pos_at(&self.unit, row, col);


    //         if let Some(ch) = &mut grid.ch {
    //             // ch.scaler.change_output(FFYuvImage::FORMAT, &self.unit)?;
    //         }
    //     }
        
    //     Ok(())
    // }

}

impl LayoutOp for LayoutGrids {
    // fn add_ch(&mut self, ch_id: VChId, flags: VChFlags) -> Result<()> {
        
    //     let r = self.grids.iter_mut().find(|x|x.ch.is_none());

    //     if let Some(grid) = r {
    //         grid.ch = Some(GridCh {
    //             ch_id,
    //             // scaler: FFAutoScaler::new(FFYuvImage::FORMAT, self.unit)?,
    //         });
    //         return Ok(())
    //     }

    //     self.grow_grids(1)?;

    //     self.add_ch(ch_id, flags)
    // }

    // fn remove_ch(&mut self, ch_id: &VChId) -> Result<()> {

    //     let pos = self.grids.iter().position(|x|{
    //         match &x.ch {
    //             Some(ch) => ch.ch_id == *ch_id,
    //             None => false,
    //         }
    //     });

    //     if let Some(pos) = pos {
    //         // let ch = self.grids[pos].ch.take();
    //         for n in pos..self.grids.len()-1 {
    //             self.grids[n].ch = self.grids[n+1].ch.take();
    //         }
    //     }
    //     Ok(())
    // }

    // fn update_ch(&mut self, _ch_id: &VChId, _image: &FFYuvImage) -> Result<()> {
    //     Ok(())
    // }

    fn get_output(&mut self, channels: &mut VChannels, canvas: &mut FFYuvImage) -> Result<()> {
        
        canvas.fill_color(&YuvColor::BLACK);
        draw_grid_channels(channels.len(), channels, canvas)?;
        // for grid in self.grids.iter_mut() {
        //     if let Some(grid_ch) = &mut grid.ch {
        //         if let Some(ch) = channels.get_mut(&grid_ch.ch_id) {
        //             if let Some(image) = &ch.image {
        //                 let (at,size) = scale_fit(image.frame().width(), image.frame().height(), self.unit.width, self.unit.height);
                        
        //                 // let dst = grid_ch.scaler.scale_to(image, &size)?;
        //                 let dst = ch.scaler.scale_to(image, &size)?;
        //                 canvas.draw_image(grid.at.add(&at), &dst);
        //             }
        //         }
        //     }
        // }

        Ok(())
    }
}

fn draw_grid_channels(num_grids: usize, channels: &mut VChannels, canvas: &mut FFYuvImage) -> Result<()> {
    
    let (rows, cols) = calc_rows_cols(num_grids as u32);

    let bk_size = canvas.size();

    let grid_size = VideoSize {
        width: bk_size.width / cols,
        height: bk_size.height / rows,
    };

    for (index, (_id, ch)) in channels.iter_mut().enumerate() {
        let num = index as u32;
        let row = num / cols;
        let col = num % cols;
        
        if row >= rows || col >= cols {
            break;
        }

        let grid_at = pos_at(&grid_size, row, col);

        if let Some(image) = &ch.image {
            let (at,size) = scale_fit(image.frame().width(), image.frame().height(), grid_size.width, grid_size.height);
            
            let dst = ch.scaler.scale_to(image, &size)?;
            let pt = grid_at.add(&at);
            canvas.draw_image(pt, &dst);
        }
    }

    Ok(())
}

// // #[derive(Clone)]
// struct Grid {
//     at: Point,
//     // size: VideoSize,
//     ch: Option<GridCh>,
// }


// struct GridCh {
//     ch_id: VChId,
//     // scaler: FFAutoScaler,
// }

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
