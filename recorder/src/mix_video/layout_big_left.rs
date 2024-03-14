

use ffmpeg_next as ff;
use crate::ffeasy::video::{image::FFYuvImage, scale_fit, Point, VideoSize, YuvColor};

use super::{LayoutOp, VChFlag, VChId, VChannels};

type Result<T> = std::result::Result<T, ff::Error>;

#[derive(Debug, Default)]
pub struct LayoutBigLeft {

}

impl LayoutOp for LayoutBigLeft {
    fn get_output(&mut self, channels: &mut VChannels, canvas: &mut FFYuvImage) -> Result<()> {
        
        canvas.fill_color(&YuvColor::BLACK);

        if channels.len() == 0 {
            return Ok(())
        }

        let big_id = match get_big_ch_id(&channels) {
            Some(v) => v,
            None => return Ok(()),
        };

        let bk_size = canvas.size();
        let num_smalls = channels.len() - 1;

        let big_size;
        let small_limit;
        let small_size ;
        let small_at;
        

        if channels.len() == 1 {
            big_size = bk_size;
            small_limit = bk_size;
            small_size = bk_size;
            small_at = Point::new(0, 0);
        } else {
            big_size = VideoSize {
                width: bk_size.width * 4 / 5,
                height: bk_size.height,
            };
            small_limit = VideoSize {
                width: bk_size.width - big_size.width,
                height: bk_size.height,
            };
            small_size = VideoSize {
                width: small_limit.width,
                height: small_limit.height / num_smalls as u32,
            };
            small_at = Point::new(big_size.width, 0);
        }

        // println!("big_id {big_id:?}");
        // println!("big_size {big_size:?}");
        // println!("small_limit {small_limit:?}");
        // println!("small_size {small_size:?}");
        // println!("small_at {small_at:?}");


        let mut small_index = 0;
        

        for (id, ch) in channels.iter_mut() {
            let grid_at;
            let grid_size;
            
            if *id == big_id {
                grid_at = Point::new(0, 0);
                grid_size = big_size;

            } else {

                grid_at = Point::new(
                    small_at.x, 
                    small_limit.height * small_index / num_smalls as u32,
                );
                grid_size = small_size;

                small_index += 1;
            }

            // println!("big_id {big_id:?}, curr_id {id:?}");

            if let Some(image) = &ch.image {
                let (at,size) = scale_fit(image.frame().width(), image.frame().height(), grid_size.width, grid_size.height);
                
                let dst = ch.scaler.scale_to(image, &size)?;
                let pt = grid_at.add(&at);
                // println!("draw {pt:?}, {size:?}");
                canvas.draw_image(pt, &dst);
            }
        }

        Ok(())
    }
}

fn get_big_ch_id(channels: &VChannels) -> Option<VChId> {
    let mut iter = channels.iter();
    
    let mut selected = iter.next().map(|x|*x.0);

    for item in iter {
        if item.1.flags.contains(VChFlag::ShareScreen) {
            selected = Some(*item.0);
            break;
        }
    }
    selected
}



