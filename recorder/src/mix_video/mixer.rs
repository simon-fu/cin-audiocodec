


use ffmpeg_next as ff;
use crate::ffeasy::video::{image::FFYuvImage, scaler::FFAutoScaler, VideoSize};
use super::{layout_big_left::LayoutBigLeft, layout_dynamic::LayoutDynamic, layout_grids::LayoutGrids, LayoutOp, VChFlags, VChId, VChannel, VChannels};


type Result<T> = std::result::Result<T, ff::Error>;

pub struct VideoMixer {
    layout: Box<dyn LayoutOp>,
    next_id: u64,
    channels: VChannels,
    image: FFYuvImage,
}

impl VideoMixer {
    pub fn new(size: VideoSize) -> Result<Self> {
        Ok(Self {
            layout: Box::new(LayoutDynamic::default()),
            next_id: 0,
            channels: Default::default(),
            image: FFYuvImage::new(size),
        })
    }

    pub fn add_ch(&mut self) -> Result<VChId> {
        self.add_ch_with(VChFlags::default())
    }

    pub fn add_ch_with(&mut self, flags: VChFlags) -> Result<VChId> {
        
        self.next_id += 1;
        let ch_id = VChId(self.next_id);

        // self.layout.add_ch(ch_id, flags)?;

        self.channels.insert(ch_id, VChannel {
            image: None,
            flags,
            scaler: FFAutoScaler::new(FFYuvImage::FORMAT, self.image.size())?,
        });
        
        Ok(ch_id)
    }

    pub fn remove_ch(&mut self, ch_id: &VChId) -> Result<()> {
        // self.layout.remove_ch(ch_id)?;
        self.channels.remove(ch_id);
        Ok(())
    }

    pub fn update_ch(&mut self, ch_id: &VChId, image: FFYuvImage) -> Result<()> {
        if let Some(ch) = self.channels.get_mut(ch_id) {
            // self.layout.update_ch(ch_id, &image)?;
            ch.image = Some(image);
        }
        Ok(())
    }

    pub fn get_output(&mut self) -> Result<&FFYuvImage> {
        self.layout.get_output(&mut self.channels, &mut self.image)?;
        Ok(&self.image)
    }
}








#[test]
fn test_mp4_to_yuv() {
    use std::{fs::File, io::Write};
    use crate::ffeasy::video::InputVideoDecoder;

    let input_file = "/tmp/sample-data/sample.mp4";
    let output_path_base = "/tmp/output";
    let max_frames = Some(160);
    let pixel = ff::util::format::Pixel::YUV420P;
    // let format: ffmpeg_sys_next::AVPixelFormat = pixel.into();
    let dst_w = 1280;
    let dst_h = 720;

    let mut decoder = InputVideoDecoder::open(&input_file).unwrap();
    
    let dst_size = VideoSize{ width: dst_w, height: dst_h };
    let mut mixer = VideoMixer::new(dst_size).unwrap();
    let ch_id1 = mixer.add_ch().unwrap();
    let ch_id2 = mixer.add_ch().unwrap();

    let output_file = format!("{output_path_base}_{dst_w}x{dst_h}_{pixel:?}.yuv", );
    let mut writer = File::create(&output_file).unwrap();
    println!("opened output {output_file}");

    let mut num_frames = 0_u64;
    let frame_iter = decoder.frame_iter();

    for frame in frame_iter.take(max_frames.unwrap_or(usize::MAX)) {

        num_frames += 1;
        
        println!(
            "Frame[{num_frames}]: {:?}, planes {}", 
            frame.format(),
            frame.planes(),
        );


        mixer.update_ch(&ch_id1, frame.clone().into()).unwrap();
        mixer.update_ch(&ch_id2, frame.clone().into()).unwrap();
        let dst = mixer.get_output().unwrap();
        let dst = dst.frame();

        for plane in 0..dst.planes() {
            println!(
                "  plane[{plane}]: len {}, w {}, h {}, stride {}", 
                dst.data(plane).len(), 
                dst.plane_width(plane), dst.plane_height(plane),
                dst.stride(plane),
            );
            writer.write_all(dst.data(plane)).unwrap();
        }

    }

    println!("output {output_file}, wrote frames {num_frames}");
}

