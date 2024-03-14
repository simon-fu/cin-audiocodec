
// use ffmpeg_next as ff;
// use ff::{software::resampling::context::Definition, Rescale};


// pub type Result<T> = std::result::Result<T, ff::Error>;

// pub type AudioArgs = Definition;

// // type ResamplerContext = ff::software::resampling::Context;
// type ResamplerContext = super::swr::SResampler;

// pub struct FFResampler {
//     resampler: ResamplerContext,
//     src_base: Option<Base>,
//     src_total_samples: u64,
//     dst_total_samples: u64,
//     dst_last_pts: i64,
// }

// impl FFResampler {
//     pub fn try_new(src: &Definition, dst: &Definition) -> Result<Self> {
//         println!("resample src {:?}, dst {:?}", DebugArgs(src), DebugArgs(dst));
//         // let resampler = ff::software::resampler(
//         //     (
//         //         src.format,
//         //         src.channel_layout,
//         //         src.rate,
//         //     ),
//         //     (
//         //         dst.format,
//         //         dst.channel_layout,
//         //         dst.rate,
//         //     ),
//         // )?;

//         let resampler = ResamplerContext::get(
//             src.format,
//             src.channel_layout,
//             src.rate,
//             dst.format,
//             dst.channel_layout,
//             dst.rate,
//         )?;

//         Ok(Self {
//             resampler,
//             src_base: None,
//             src_total_samples: 0,
//             dst_total_samples: 0,
//             dst_last_pts: 0,
//         })
//     }

//     pub fn convert(&mut self, input: &ff::frame::Audio, time_base: ff::Rational) -> Result<ff::frame::Audio> {

//         // let dst_delta_samples = self.calc_dst_delta_samples(input.samples());

//         // let pts = match input.pts() {
//         //     Some(src) => {
//         //         let src = src.rescale(time_base, ff::Rational::new(1, 1000));
//         //         let dst_pts = self.convert_ts(src, input.samples(), dst_delta_samples);
//         //         Some(dst_pts)
//         //     },
//         //     None => None,
//         // };


//         // let mut output = ff::frame::Audio::new(self.resampler.output().format, dst_delta_samples, self.resampler.output().channel_layout);
//         let mut output = ff::frame::Audio::empty();
//         let pts = input.pts().map(|x|x.rescale(time_base, ff::Rational::new(1, 1000)));
        
//         let _delay = self.resampler.run(input, &mut output)?;
//         println!(" aaa in samples {}, out samples {}, delay {_delay:?}", input.samples(), output.samples());

//         output.set_pts(pts);
//         Ok(output)
//     }

//     fn convert_ts(&mut self, src_ts: i64, src_samples: usize, dst_delta_samples: usize) -> i64 {

//         let (is_reset, src) = convert_ts(
//             &mut self.src_base, 
//             self.resampler.input(), 
//             src_ts, 
//             src_samples,
//         );

//         if is_reset {
//             self.dst_last_pts = src;
//         }

//         let dst_pts = self.dst_last_pts;

//         let dst_delta_pts = samples_to_millis(self.resampler.output().rate, dst_delta_samples);
//         self.dst_last_pts += dst_delta_pts;

//         dst_pts
//     }

//     fn calc_dst_delta_samples(&mut self, samples: usize) -> usize {
//         let src_total_samples = self.src_total_samples + samples as u64;
//         let total_millis = samples_to_millis_u64(self.resampler.input().rate, src_total_samples);
//         let dst_total_samples = millis_to_samples(self.resampler.output().rate, total_millis) as u64;
//         let dst_delta_samples = dst_total_samples - self.dst_total_samples;

//         let in_rate = self.resampler.input().rate;
//         let out_rate = self.resampler.output().rate;
//         println!("  src: {samples}/{src_total_samples}/{in_rate}, dst {dst_delta_samples}/{dst_total_samples}/{out_rate}, milli {total_millis}");

//         dst_delta_samples as usize
//     }
    
// }

// struct DebugArgs<'a>(&'a Definition);

// impl<'a> std::fmt::Debug for DebugArgs<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("AudioArgs")
//         .field("format", &self.0.format)
//         .field("channels", &self.0.channel_layout.channels())
//         .field("rate", &self.0.rate)
//         .finish()
//     }
// }

// // #[derive(Debug, Default)]
// // struct LegState {
// //     base: Option<Base>,
// //     total_samples: u64,
// // }

// // impl LegState {
// //     pub fn convert(&mut self, cfg: &Definition, src: i64, samples: usize) -> (bool, i64) {
// //         convert_ts(&mut self.base, cfg, src, samples)
// //     }
// // }

// fn convert_ts(base: &mut Option<Base>, cfg: &Definition, src: i64, samples: usize) -> (bool, i64) {
//     if let Some(base) = base {
//         let expect_ts = base.ts + samples_to_millis_u64(cfg.rate, base.samples);
//         let diff = expect_ts.abs_diff(src);
//         if diff <= 5 {
//             base.samples += samples as u64;
//             return (false, expect_ts)
//         }

//         println!("aaa exceed diff {diff}, expect {expect_ts} but {src}");
//     }

//     *base = Some(Base {
//         ts: src,
//         samples: samples as u64,
//     });
//     (true, src)
// }





// #[derive(Debug, Default, Clone, Copy)]
// struct Base {
//     ts: i64,
//     samples: u64,
// }

// fn samples_to_millis(rate: u32, len: usize) -> i64 {
//     1000 * (len as i64) / (rate as i64)
// }

// fn samples_to_millis_u64(rate: u32, len: u64) -> i64 {
//     1000 * (len as i64) / (rate as i64)
// }

// fn millis_to_samples(rate: u32, millis: i64) -> usize {
//     (millis as usize) * (rate as usize) / 1000
// }


// #[cfg(test)]
// mod test {
//     use ffmpeg_next as ff;
//     use std::{fs::File, io::Write};
//     use crate::ffeasy::audio::input::{InputAudioDecoder, AudioArgs};
//     use ff::ChannelLayout;
//     use super::FFResampler;
    
//     #[test]
//     fn test_mp4_resample_pcm() {
    
//         // let input_file = "/tmp/sample-data/sample.mp4";
//         let input_file = "/tmp/sample-data/ForBiggerBlazes.mp4";
//         let output_path_base = "/tmp/output";
//         let max_frames = Some(16000);
//         let dst_samplerate = 48000;
//         let dst_channels = 2;
    
    
//         let mut decoder = InputAudioDecoder::open(
//             &input_file, 
//             None,
//         ).unwrap();
    
    
//         let output_file = format!("{output_path_base}_{dst_samplerate}hz_{dst_channels}ch.pcm", );
//         let mut writer = File::create(&output_file).unwrap();
//         println!("opened output {output_file}");
    
//         let mut resampler = FFResampler::try_new(
//             &AudioArgs {
//                 format: decoder.get_format(),
//                 channel_layout: ChannelLayout::default(decoder.get_channels() as i32),
//                 rate: decoder.get_samplerate(),
//             }, 
//             &AudioArgs {
//                 format: ff::format::Sample::I16(ff::format::sample::Type::Packed),
//                 channel_layout: ChannelLayout::default(dst_channels),
//                 rate: dst_samplerate,
//             }
//         ).unwrap();
    
//         let input_time_base = decoder.get_time_base();

//         let mut time_checker = PcmTimeChecker::new(dst_samplerate);
    
//         let mut num_frames = 0_u64;
    
//         for frame in decoder.frame_iter().take(max_frames.unwrap_or(usize::MAX)) {
    
//             num_frames += 1;
    
//             println!(
//                 "Frame[{num_frames}]: {:?}, planes {}, samples {}, rate {:?}, ch {}, pts {:?}", 
//                 frame.format(),
//                 frame.planes(),
//                 frame.samples(),
//                 frame.rate(),
//                 frame.channels(),
//                 frame.pts(),
//             );
    
//             let dst = resampler.convert(&frame, input_time_base).unwrap();
//             time_checker.check(dst.pts().unwrap(), dst.samples());
    
//             println!(
//                 "  dst: {:?}, planes {}, samples {}, rate {:?}, ch {}, pts {:?}", 
//                 dst.format(),
//                 dst.planes(),
//                 dst.samples(),
//                 dst.rate(),
//                 dst.channels(),
//                 dst.pts(),
//             );
    
//             for index in 0..dst.planes() {
//                 let plane = dst.plane::<i16>(index);
//                 println!(
//                     "  plane[{index}]: raw len {}, samples {}, plane<i16> {}", 
//                     dst.data(index).len(), 
//                     dst.samples(),
//                     plane.len(),
//                 );
//                 let len = dst.samples() * 2 * dst.channels() as usize;
//                 let data = &dst.data(index)[..len];
//                 writer.write_all(data).unwrap();
//             }
//         }
    
//         println!("output {output_file}, wrote frames {num_frames}");
//     }
    

//     struct PcmTimeChecker {
//         samplerate: u32, 
//         // channels: u32,
//         first_ts: Option<i64>,
//         num_samples: usize,
//     }
    
//     impl PcmTimeChecker {
//         pub fn new(samplerate: u32) -> Self {
//             Self {
//                 samplerate,
//                 // channels,
//                 first_ts: None,
//                 num_samples: 0,
//             }
//         }
    
//         pub fn check(&mut self, ts: i64, samples: usize) -> i64 {
//             let expect_ts = match self.first_ts {
//                 Some(first) => {
//                     first + samples_to_millis(self.samplerate, self.num_samples)
//                 },
//                 None => {
//                     self.first_ts = Some(ts);
//                     ts
//                 },
//             };
    
//             let diff = ts - expect_ts;
//             println!("  check pcm: expect {expect_ts}, real {ts}, diff {diff}, samples {}", self.num_samples);
    
//             self.num_samples += samples;
//             diff
//         }
//     }

//     fn samples_to_millis(samplerate: u32, samples: usize) -> i64 {
//         1000 * (samples as i64) / (samplerate as i64)
//     }
// }

