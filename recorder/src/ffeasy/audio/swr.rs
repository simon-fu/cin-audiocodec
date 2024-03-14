
/*
    - refer: ffmpeg_next::software::resampling::Context  (ffmpeg-next 6.1.1)

    - Fixed in run()
        Delay is None when delay small than 1 seconds
        Fill channel_layout and format for output frame;
        Do not alloc if output frame is empty;
        
    - Add fn convert_whole()
*/


use ffmpeg_next as ff;
use std::ptr;
// use super::Delay;

use ff::ffi::*;
use libc::c_int;
use std::ffi::c_void;
use ff::util::format;
use ff::Dictionary;
use ff::{frame, ChannelLayout, Error};

pub type Delay = ff::software::resampling::Delay;
pub type Definition = ff::software::resampling::context::Definition;

// #[derive(Eq, PartialEq, Copy, Clone)]
// pub struct Definition {
//     pub format: format::Sample,
//     pub channel_layout: ChannelLayout,
//     pub rate: u32,
// }

pub struct SResampler {
    ptr: *mut SwrContext,

    input: Definition,
    output: Definition,
}

unsafe impl Send for SResampler {}

impl SResampler {
    #[doc(hidden)]
    pub unsafe fn as_ptr(&self) -> *const SwrContext {
        self.ptr as *const _
    }

    #[doc(hidden)]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut SwrContext {
        self.ptr
    }
}

impl SResampler {
    /// Create a resampler with the given definitions.
    pub fn get(
        src_format: format::Sample,
        src_channel_layout: ChannelLayout,
        src_rate: u32,
        dst_format: format::Sample,
        dst_channel_layout: ChannelLayout,
        dst_rate: u32,
    ) -> Result<Self, Error> {
        Self::get_with(
            src_format,
            src_channel_layout,
            src_rate,
            dst_format,
            dst_channel_layout,
            dst_rate,
            Dictionary::new(),
        )
    }

    /// Create a resampler with the given definitions and custom options dictionary.
    pub fn get_with(
        src_format: format::Sample,
        src_channel_layout: ChannelLayout,
        src_rate: u32,
        dst_format: format::Sample,
        dst_channel_layout: ChannelLayout,
        dst_rate: u32,
        options: Dictionary,
    ) -> Result<Self, Error> {
        unsafe {
            let ptr = swr_alloc_set_opts(
                ptr::null_mut(),
                dst_channel_layout.bits() as i64,
                dst_format.into(),
                dst_rate as c_int,
                src_channel_layout.bits() as i64,
                src_format.into(),
                src_rate as c_int,
                0,
                ptr::null_mut(),
            );

            let mut opts = options.disown();
            let res = av_opt_set_dict(ptr as *mut c_void, &mut opts);
            Dictionary::own(opts);

            if res != 0 {
                return Err(Error::from(res));
            }

            if !ptr.is_null() {
                match swr_init(ptr) {
                    e if e < 0 => Err(Error::from(e)),

                    _ => Ok(SResampler {
                        ptr,

                        input: Definition {
                            format: src_format,
                            channel_layout: src_channel_layout,
                            rate: src_rate,
                        },

                        output: Definition {
                            format: dst_format,
                            channel_layout: dst_channel_layout,
                            rate: dst_rate,
                        },
                    }),
                }
            } else {
                Err(Error::InvalidData)
            }
        }
    }

    /// Get the input definition.
    pub fn input(&self) -> &Definition {
        &self.input
    }

    /// Get the output definition.
    pub fn output(&self) -> &Definition {
        &self.output
    }

    /// Get the remaining delay.
    pub fn delay(&self) -> Option<Delay> {
        unsafe {
            match swr_get_delay(self.as_ptr() as *mut _, 1000) {
                0 => None,
                _ => Some(get_delay(self)),
            }
        }
    }

    pub fn resample_whole(
        &mut self,
        input: &frame::Audio,
    ) -> Result<frame::Audio, Error> {

        let mut output = frame::Audio::empty();
        
        let r = unsafe {
            (*output.as_mut_ptr()).sample_rate = self.output.rate as i32;
            (*output.as_mut_ptr()).channel_layout = self.output.channel_layout.bits();
            (*output.as_mut_ptr()).format = AVSampleFormat::from(self.output.format) as i32;
            swr_convert_frame(self.as_mut_ptr(), output.as_mut_ptr(), input.as_ptr())
        };

        match r {
            0 => {
                output.set_pts(input.pts());
                Ok(output)
            },
            e => Err(Error::from(e)),
        }
    }

    /// Run the resampler from the given input to the given output.
    ///
    /// When there are internal frames to process it will return `Ok(Some(Delay { .. }))`.
    pub fn run(
        &mut self,
        input: &frame::Audio,
        output: &mut frame::Audio,
    ) -> Result<Option<Delay>, Error> {
        unsafe {
            (*output.as_mut_ptr()).sample_rate = self.output.rate as i32;
            (*output.as_mut_ptr()).channel_layout = self.output.channel_layout.bits();
            (*output.as_mut_ptr()).format = AVSampleFormat::from(self.output.format) as i32;
        }


        unsafe {
            // if output.is_empty() {
            //     output.alloc(
            //         self.output.format,
            //         input.samples(),
            //         self.output.channel_layout,
            //     );
            // }

            match swr_convert_frame(self.as_mut_ptr(), output.as_mut_ptr(), input.as_ptr()) {
                0 => Ok(self.delay()),

                e => Err(Error::from(e)),
            }
        }
    }

    /// Convert one of the remaining internal frames.
    ///
    /// When there are no more internal frames `Ok(None)` will be returned.
    pub fn flush(&mut self, output: &mut frame::Audio) -> Result<Option<Delay>, Error> {
        unsafe {
            (*output.as_mut_ptr()).sample_rate = self.output.rate as i32;
        }

        unsafe {
            match swr_convert_frame(self.as_mut_ptr(), output.as_mut_ptr(), ptr::null()) {
                0 => Ok(self.delay()),

                e => Err(Error::from(e)),
            }
        }
    }
}

impl Drop for SResampler {
    fn drop(&mut self) {
        unsafe {
            swr_free(&mut self.as_mut_ptr());
        }
    }
}



fn get_delay(context: &SResampler) -> Delay {
    unsafe {
        Delay {
            seconds: swr_get_delay(context.as_ptr() as *mut _, 1),
            milliseconds: swr_get_delay(context.as_ptr() as *mut _, 1000),
            input: swr_get_delay(context.as_ptr() as *mut _, i64::from(context.input().rate)),
            output: swr_get_delay(context.as_ptr() as *mut _, i64::from(context.output().rate)),
        }
    }
}



#[cfg(test)]
mod test {
    use ffmpeg_next as ff;
    use std::{fs::File, io::Write};
    use crate::ffeasy::audio::input::InputAudioDecoder;
    use ff::{ChannelLayout, Rescale};
    use super::SResampler;
    
    #[test]
    fn test_mp4_resample_pcm() {
    
        // let input_file = "/tmp/sample-data/sample.mp4";
        let input_file = "/tmp/sample-data/ForBiggerBlazes.mp4";
        let output_path_base = "/tmp/output";
        let max_frames = Some(16000);
        let dst_samplerate = 48000;
        let dst_channels = 2;
    
    
        let mut decoder = InputAudioDecoder::open(
            &input_file, 
            None,
        ).unwrap();
    
    
        let output_file = format!("{output_path_base}_{dst_samplerate}hz_{dst_channels}ch.pcm", );
        let mut writer = File::create(&output_file).unwrap();
        println!("opened output {output_file}");
    
        let mut resampler = SResampler::get(
            decoder.get_format(),
            ChannelLayout::default(decoder.get_channels() as i32),
            decoder.get_samplerate(),
            ff::format::Sample::I16(ff::format::sample::Type::Packed),
            ChannelLayout::default(dst_channels),
            dst_samplerate,
        ).unwrap();
        
    
        let input_time_base = decoder.get_time_base();

        let mut time_checker = PcmTimeChecker::new(dst_samplerate);
    
        let mut num_frames = 0_u64;
    
        for frame in decoder.frame_iter().take(max_frames.unwrap_or(usize::MAX)) {
    
            num_frames += 1;
    
            println!(
                "Frame[{num_frames}]: {:?}, planes {}, samples {}, rate {:?}, ch {}, pts {:?}", 
                frame.format(),
                frame.planes(),
                frame.samples(),
                frame.rate(),
                frame.channels(),
                frame.pts(),
            );
    
            // let mut dst = ff::frame::Audio::empty();
            // let delay = resampler.run(&frame, &mut dst).unwrap();
            // // let pts = frame.pts().map(|x|x.rescale(input_time_base, ff::Rational::new(1, 1000)));
            // dst.set_pts(frame.pts());

            let dst = resampler.resample_whole(&frame).unwrap();
            let delay = resampler.delay();
            
            time_checker.check(dst.pts().unwrap(), dst.samples(), input_time_base);
    
            println!(
                "  dst: {:?}, planes {}, samples {}, rate {:?}, ch {}, pts {:?}, delay {delay:?}", 
                dst.format(),
                dst.planes(),
                dst.samples(),
                dst.rate(),
                dst.channels(),
                dst.pts(),
            );
    
            for index in 0..dst.planes() {
                let plane = dst.plane::<i16>(index);
                println!(
                    "  plane[{index}]: raw len {}, samples {}, plane<i16> {}", 
                    dst.data(index).len(), 
                    dst.samples(),
                    plane.len(),
                );
                let len = dst.samples() * 2 * dst.channels() as usize;
                let data = &dst.data(index)[..len];
                writer.write_all(data).unwrap();
            }
        }
    
        println!("output {output_file}, wrote frames {num_frames}");
    }
    

    struct PcmTimeChecker {
        samplerate: u32, 
        // channels: u32,
        first_ts: Option<i64>,
        num_samples: usize,
    }
    
    impl PcmTimeChecker {
        pub fn new(samplerate: u32) -> Self {
            Self {
                samplerate,
                // channels,
                first_ts: None,
                num_samples: 0,
            }
        }
    
        pub fn check(&mut self, ts: i64, samples: usize, time_base: ff::Rational) -> i64 {
            let ts = ts.rescale(time_base, ff::Rational::new(1, 1000));
            let expect_ts = match self.first_ts {
                Some(first) => {
                    first + samples_to_millis(self.samplerate, self.num_samples)
                },
                None => {
                    self.first_ts = Some(ts);
                    ts
                },
            };
    
            let diff = ts - expect_ts;
            println!("  check pcm: expect {expect_ts}, real {ts}, diff {diff}, samples {}", self.num_samples);
    
            self.num_samples += samples;
            diff
        }
    }

    fn samples_to_millis(samplerate: u32, samples: usize) -> i64 {
        1000 * (samples as i64) / (samplerate as i64)
    }
}


