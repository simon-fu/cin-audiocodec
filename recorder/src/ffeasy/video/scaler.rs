
use ffmpeg_next as ff;

use super::{image::FFYuvImage, VideoSize};


pub struct FFAutoScaler {
    scaler: ff::software::scaling::Context,
}

// impl Clone for FFAutoScaler {
//     fn clone(&self) -> Self {
//         let input = self.scaler.input();
//         let output = self.scaler.output();

//         Self { 
//             scaler: ff::software::scaling::Context::get(
//                 input.format,
//                 input.width,
//                 input.height,
//                 output.format,
//                 output.width,
//                 output.height,
//                 Self::DEFAULT_FLAGS,
//             ).unwrap(),
//         }
//     }
// }

impl FFAutoScaler {
    const DEFAULT_FLAGS: ff::software::scaling::Flags = ff::software::scaling::Flags::AREA;

    pub fn new(format: ff::util::format::Pixel, size: VideoSize) -> Result<Self, ff::Error> {
        Ok(Self {
            scaler: ff::software::scaling::Context::get(
                format,
                size.width,
                size.height,
                format,
                size.width,
                size.height,
                Self::DEFAULT_FLAGS,
            )?,
        })
    }

    pub fn change_output(&mut self, format: ff::util::format::Pixel, size: &VideoSize) -> Result<(), ff::Error> {

        let output = self.scaler.output();

        if size.width != output.width
        || size.height != output.height
        || format != output.format {

            let input = self.scaler.input();
            
            self.scaler = ff::software::scaling::Context::get(
                input.format,
                input.width,
                input.height,
                output.format,
                size.width,
                size.height,
                Self::DEFAULT_FLAGS,
            )?;
        }
        Ok(())
    }

    // pub fn scale_fit(&mut self, src: &FFYuvImage, limits: &VideoSize) -> Result<FFYuvImage, ff::Error> {
    //     let height = limits.width * src.frame().height() / src.frame().width();
    //     if height <= limits.height {
    //         self.scale_to(src, &VideoSize{
    //             width: limits.width,
    //             height,
    //         })
    //     } else {
    //         let width = limits.height * src.frame().width() / src.frame().height();
    //         self.scale_to(src, &VideoSize{
    //             width,
    //             height: limits.height,
    //         })
    //     }
    // }

    pub fn scale_to(&mut self, src: &FFYuvImage, size: &VideoSize) -> Result<FFYuvImage, ff::Error> {
        self.change_output(self.scaler.output().format, size)?;
        self.scale(src)
    }

    pub fn scale(&mut self, src: &FFYuvImage) -> Result<FFYuvImage, ff::Error> {
        let src = &src.frame;
        if src.width() != self.scaler.input().width
            || src.height() != self.scaler.input().height
            || src.format() != self.scaler.input().format {

                self.scaler = ff::software::scaling::Context::get(
                    // ff::util::format::Pixel::YUV420P,
                    src.format(),
                    src.width(),
                    src.height(),
                    self.scaler.output().format,
                    self.scaler.output().width,
                    self.scaler.output().height,
                    Self::DEFAULT_FLAGS,
                )?;
            }

            let mut dst = ff::util::frame::Video::empty();
            self.scaler.run(&src, &mut dst)?;
            
            Ok(dst.into())
    }
}
