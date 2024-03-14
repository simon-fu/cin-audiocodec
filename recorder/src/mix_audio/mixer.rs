


use std::collections::HashMap;
use ffmpeg_next as ff;
use crate::rwbuf::{RBuf, RwBufVec};



type Result<T> = std::result::Result<T, ff::Error>;

pub struct PcmMixer {
    next_id: u64,
    sources: HashMap<AChId, PcmChannel>,
    max_len: usize,
}

impl PcmMixer {
    pub fn new(max_len: usize) -> Result<Self> {
        Ok(Self {
            max_len,
            next_id: 0,
            sources: Default::default(),
        })
    }

    pub fn add_ch(&mut self) -> Result<AChId> {
        self.next_id += 1;
        let ch_id = AChId(self.next_id);

        self.sources.insert(ch_id, PcmChannel {
            pcm: RwBufVec::new(self.max_len),
        });
        
        Ok(ch_id)
    }

    pub fn remove_ch(&mut self, ch_id: &AChId) -> Result<()> {
        self.sources.remove(ch_id);
        Ok(())
    }

    pub fn update_ch(&mut self, ch_id: &AChId, samples: &[i16]) -> Result<()> {
        if let Some(ch) = self.sources.get_mut(ch_id) {
            ch.pcm.push_rotate(samples);
        }
        Ok(())
    }

    pub fn pull_mix(&mut self, buf: &mut [i16]) {
        buf.fill(0);
        for (_id, ch) in self.sources.iter_mut() {
            let pcm = ch.pcm.rdata();
            let len = pcm.len().min(buf.len());
            // println!("aaa pull_mix: id {_id:?}, len {len}, buf.len {}", buf.len());
            for i in 0..len {
                buf[i] = buf[i].saturating_add(pcm[i]); // aaa 务必改成这个 !!!!!!!!!!!!!!!!!!!!!!!!!!!
                // buf[i] += pcm[i];
            }
            ch.pcm.radvance(len);
        }
    }
}

struct PcmChannel {
    pcm: RwBufVec<i16>,
}


pub struct PcmTimedMixer {
    // frame_len: usize,
    first_mix_ts: Option<i64>,
    // mixer: PcmMixer,
    // buf: Vec<i16>,
    // frame_millis: i64,
    samplerate: u32, 
    channels: u32, 
    num_pulled: usize,
}

impl PcmTimedMixer {
    pub fn new(
        samplerate: u32, 
        channels: u32, 
        // frame_millis: i64,
        // max_buf_millis: i64, 
    ) -> Result<Self> {

        // let frame_len = (channels as usize) * (frame_millis as usize) * (samplerate as usize) / 1000;
        // let max_len = (channels as usize) * (max_buf_millis as usize) * (samplerate as usize) / 1000;
        Ok(Self {
            // buf: vec![0; frame_len],
            first_mix_ts: None,
            // mixer: PcmMixer::new(max_len)?,
            // frame_millis,
            samplerate,
            channels,
            num_pulled: 0,
        })
    }

    pub fn try_pull<'a>(&'a mut self, ts: i64, mixer: &mut PcmMixer, buf: &mut [i16]) -> Option<i64> {
        let first = match self.first_mix_ts {
            Some(v) => v,
            None => {
                self.first_mix_ts = Some(ts);
                return None
            },
        };

        let last = first + len_to_millis(self.samplerate, self.channels, self.num_pulled);

        let frame_millis = len_to_millis(self.samplerate, self.channels, buf.len());

        let elapsed = ts - last;
        
        if elapsed < (frame_millis + 0) { // aaa
            return None;
        }

        mixer.pull_mix(buf);
        self.num_pulled += buf.len();
        
        Some(last)
    }

    pub fn millis_to_len(&self, millis: i64) -> usize {
        millis_to_len(self.samplerate, self.channels, millis)
    }

}

fn millis_to_len(samplerate: u32, channels: u32, millis: i64) -> usize {
    (channels as usize) * (millis as usize) * (samplerate as usize) / 1000
}

fn len_to_millis(samplerate: u32, channels: u32, len: usize) -> i64 {
    1000 * (len as i64) / (channels as i64) / (samplerate as i64)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AChId(pub(super) u64);



#[cfg(test)]
mod poc {
    use ffmpeg_next as ff;
    use std::{fs::File, io::Write};
    use crate::ffeasy::audio::input::{InputAudioDecoder, AudioArgs};
    use crate::ffeasy::audio::input::Decoded;
    use crate::ffeasy::time::ScaleTime;
    use crate::mix_audio::mixer::{AChId, PcmMixer, PcmTimedMixer};
    use ff::{format::{sample, Sample}, ChannelLayout};
    use crate::ffeasy::ffi::audio_frame_packed_i16_samples;
    use crate::ffeasy::{audio::swr::SResampler, ffi::audio_packed_i16_to_bytes};

    #[test]
    fn test_mp4_to_pcm_mix() {
    
        let input_file1 = "/tmp/sample-data/sample.mp4";
        let input_file2 = "/tmp/sample-data/ForBiggerBlazes.mp4";
        let output_path_base = "/tmp/output";
        let max_frames = Some(64000);
        let dst_samplerate = 48000;
        let dst_channels = 2;
    
    
        let target_args = AudioArgs {
            format: Sample::I16(sample::Type::Packed),
            channel_layout: ChannelLayout::default(dst_channels),
            rate: dst_samplerate,
        };
    
        let mut decoder1 = InputAudioDecoder::open(
            &input_file1, 
            Some(target_args.clone())).unwrap();
    
        let mut decoder2 = InputAudioDecoder::open(
            &input_file2, 
            Some(target_args.clone())).unwrap();
    
    
        let output_file = format!("{output_path_base}_{dst_samplerate}hz_{dst_channels}ch.pcm", );
        let mut writer = File::create(&output_file).unwrap();
        println!("opened output {output_file}");
    
        let max_frames = max_frames.unwrap_or(usize::MAX);
        let milli_time_base = ff::Rational::new(1, 1000);
    
        let time_scalers = vec! [
            decoder1.time_scaler(milli_time_base),
            decoder2.time_scaler(milli_time_base),
        ];
    
        // fn make_resampler(decoder: &InputAudioDecoder, target: &AudioArgs) -> SResampler {
        //     println!(
        //         "make resampler: src {:?}, dst {:?}",
        //         (
        //             decoder.get_format(), 
        //             ChannelLayout::default(decoder.get_channels() as i32), 
        //             decoder.get_samplerate(),
        //         ),
        //         (
        //             target.format, 
        //             target.channel_layout, 
        //             target.rate,
        //         )
        //     );
    
        //     SResampler::get(
        //         decoder.get_format(), 
        //         ChannelLayout::default(decoder.get_channels() as i32), 
        //         decoder.get_samplerate(), 
        //         target.format, 
        //         target.channel_layout, 
        //         target.rate,
        //     ).unwrap()
        // }
        
        let mut resamplers: Vec<SResampler>  = vec! [
            decoder1.resampler(&target_args).unwrap(),
            decoder2.resampler(&target_args).unwrap(),
        ];
    
    
        type FrameIter<'a> = Box<dyn Iterator<Item = Decoded> + 'a>;
    
        let mut frame_iters: Vec<FrameIter<'_>> = vec![
            Box::new(decoder1.frame_iter().take(max_frames)),
            Box::new(decoder2.frame_iter().take(max_frames)),
        ];
    
    
    
        let mut decoded_frames: Vec<Option<Decoded>> = vec![None; frame_iters.len()];
    
        let mut timed = PcmTimedMixer::new(dst_samplerate, dst_channels as u32).unwrap();
        let mut mixer= PcmMixer::new(timed.millis_to_len(1000)).unwrap();
        
    
        let frame_millis = 20;
        let frame_len = timed.millis_to_len(frame_millis);
        let mut mixed_buf = vec![0_i16; frame_len];
        // let frame_len = (dst_channels as usize) * frame_millis * (dst_samplerate as usize) / 1000;
    
        
        let ids: Vec<AChId>  = (0..frame_iters.len())
            .map(|_x|mixer.add_ch().unwrap())
            .collect();
    
        // let mut checkers: Vec<PcmTimeChecker>  = (0..frame_iters.len())
        //     .map(|_x| PcmTimeChecker::new(dst_samplerate, dst_channels as u32) )
        //     .collect();
    
    
        let mut num_frames = 0_u64;
    
        loop {
            for (index, item) in decoded_frames.iter_mut().enumerate() {
                if item.is_none() {
                    let r = frame_iters[index].next();
                    if let Some(mut v) = r {
                        v.scale_time(&time_scalers[index]);
                        *item = Some(v)
                    }
                }
            }
            
            let (frame, ch_index) = {
                let mut min_index = None;
                let mut min_ts = i64::MAX;
                for (index, item) in decoded_frames.iter_mut().enumerate() {
                    if let Some(item) = item {
                        if let Some(pts) = item.pts() {
                            if pts < min_ts {
                                min_ts = pts;
                                min_index = Some(index);
                            }
                        }
                    }
                }
                match min_index {
                    Some(index) => {
                        (decoded_frames[index].take().unwrap(), index)
                    },
                    None => {
                        println!("reach end");
                        break
                    },
                }
            };
    
    
            num_frames += 1;
    
            println!(
                "Frame[{num_frames}]: src {ch_index}, {:?}, planes {}, samples {}, pts {:?}", 
                frame.format(),
                frame.planes(),
                frame.samples(),
                frame.pts(),
            );
            
            let frame = resamplers[ch_index].resample_whole(&frame).unwrap();
            {
                let dst = &frame;
    
                for index in 0..dst.planes() {
                    let plane = dst.plane::<i16>(index);
                    println!(
                        "  plane[{index}]: raw len {}, samples {}, plane<i16> {}", 
                        dst.data(index).len(), 
                        dst.samples(),
                        plane.len(),
                    );
    
                    // let len = dst.samples() * 2 * dst.channels() as usize;
                    // let data = &dst.data(index)[..len];
                    // writer.write_all(data).unwrap();
                }
            }
    
            let new_ts = frame.pts().unwrap();
            while let Some(ts) = timed.try_pull(new_ts, &mut mixer, &mut mixed_buf) {
                let data = audio_packed_i16_to_bytes(&mixed_buf[..]);
                println!("  write mixed pcm: ts {ts}, len {}", data.len());
                writer.write_all(data).unwrap();
            }
    
            // {
            //     let samples = audio_frame_packed_i16_samples(&frame);
            //     let mut buf = vec![0; samples.len()];
    
            //     let new_ts = frame.pts().unwrap();
                
            //     checkers[ch_index].check(new_ts, samples.len());
    
            //     while let Some(ts) = timed.try_pull(new_ts, &mut mixer, &mut buf) {
            //         let data = audio_packed_i16_to_bytes(&buf[..]);                
            //         println!("  write mixed pcm: ts {ts}, len {}", data.len());
            //         writer.write_all(data).unwrap();
            //     }
            // }
    
    
            
            {
    
                let samples = audio_frame_packed_i16_samples(&frame);
                mixer.update_ch(&ids[ch_index], samples).unwrap();
    
                // {
                //     let mut buf = vec![0; samples.len()];
                //     mixer.pull_mix(&mut buf);
    
                //     let data = unsafe { 
                //         std::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len() * 2) 
                //     };
                //     println!("  write simple mixed pcm len {}", data.len());
                //     writer.write_all(data).unwrap();
                // }
            }
            
        }
    
        println!("output {output_file}, wrote frames {num_frames}");
    }
    
    
    // struct PcmTimeChecker {
    //     samplerate: u32, 
    //     channels: u32,
    //     first_ts: Option<i64>,
    //     num_samples: usize,
    // }
    
    // impl PcmTimeChecker {
    //     pub fn new(samplerate: u32, channels: u32) -> Self {
    //         Self {
    //             samplerate,
    //             channels,
    //             first_ts: None,
    //             num_samples: 0,
    //         }
    //     }
    
    //     pub fn check(&mut self, ts: i64, samples: usize) -> i64 {
    //         let expect_ts = match self.first_ts {
    //             Some(first) => {
    //                 first + len_to_millis(self.samplerate, self.channels, self.num_samples)
    //             },
    //             None => {
    //                 self.first_ts = Some(ts);
    //                 ts
    //             },
    //         };
    
    //         let diff = ts - expect_ts;
    //         println!("  check pcm: expect {expect_ts}, real {ts}, diff {diff}, samples {}", self.num_samples);
    
    //         self.num_samples += samples;
    //         diff
    //     }
    // }
    
    #[test]
    fn test_mp4_to_pcm() {
        use std::{fs::File, io::Write};
        use crate::ffeasy::audio::input::{InputAudioDecoder, AudioArgs};
        use ff::{format::{sample, Sample}, ChannelLayout};
    
    
        let input_file = "/tmp/sample-data/sample.mp4";
        let output_path_base = "/tmp/output";
        let max_frames = Some(16);
        let dst_samplerate = 48000;
        let dst_channels = 2;
    
    
        let mut decoder = InputAudioDecoder::open(
            &input_file, 
            Some(AudioArgs {
                format: Sample::I16(sample::Type::Packed),
                channel_layout: ChannelLayout::default(dst_channels),
                rate: dst_samplerate,
            }
        )).unwrap();
    
    
        let output_file = format!("{output_path_base}_{dst_samplerate}hz_{dst_channels}ch.pcm", );
        let mut writer = File::create(&output_file).unwrap();
        println!("opened output {output_file}");
    
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
    
            let dst = frame;
    
            for index in 0..dst.planes() {
                let plane = dst.plane::<i16>(index);
                println!(
                    "  plane[{index}]: raw len {}, samples {}, plane<i16> {}", 
                    dst.data(index).len(), 
                    dst.samples(),
                    plane.len(),
                );
                writer.write_all(dst.data(index)).unwrap();
            }
        }
    
        println!("output {output_file}, wrote frames {num_frames}");
    }
    
    
    
    
    
    // #[test]
    // fn test_mp4_to_pcm() {
    //     use std::{fs::File, io::Write};
    //     use crate::ffeasy::parameters::FFParameters;
    //     use crate::ffeasy::audio::input::{audio_decoder_receive_frame, make_audio_decoder};
    //     use ff::{format::{sample, Sample}, ChannelLayout};
    
    //     let input_file = "/tmp/sample-data/sample.mp4";
    //     let output_path_base = "/tmp/output";
    //     let max_frames = Some(1600);
    //     // let format = ff::util::format::Sample::I16(ff::util::format::sample::Type::Packed);
    
    
        
    
    //     let mut ictx = ff::format::input(&input_file).unwrap();
    //     println!("opened input {input_file}");
    
    //     let i_track = ictx.streams()
    //     .best(ff::util::media::Type::Audio)
    //     .ok_or_else(||ff::Error::StreamNotFound).unwrap();
    
    //     let i_params: FFParameters = i_track.parameters().into();
    //     let i_time_base = i_track.time_base();
    //     let i_index = i_track.index();
    
        
    
    //     let mut decoder = make_audio_decoder(
    //         i_params.get_codec_id().into(), 
    //         i_params.get_samplerate(), 
    //         i_params.get_channels(), 
    //         i_time_base,
    //     ).unwrap();
    
        
        
    //     // let mut mixer = PcmMixer::new(dst_samplerate * dst_channels).unwrap();
    //     // let ch_id1 = mixer.add_ch().unwrap();
    //     // let ch_id2 = mixer.add_ch().unwrap();
    
    //     println!("params {}/{}/{:?}", i_params.get_samplerate(), i_params.get_channels(), i_params.get_format_audio());
    
    //     let dst_samplerate = i_params.get_samplerate();
    //     let dst_channels = i_params.get_channels();
    //     let mut resampler = ff::software::resampler(
    //         (
    //             // Sample::I16(sample::Type::Packed),
    //             i_params.get_format_audio().into(),
    //             ChannelLayout::default(i_params.get_channels()),
    //             i_params.get_samplerate() as u32,
    //         ),
    //         (
    //             Sample::I16(sample::Type::Packed),
    //             ChannelLayout::default(dst_channels),
    //             dst_samplerate as u32,
    //         ),
    //     ).unwrap();
        
    
    //     let output_file = format!("{output_path_base}_{dst_samplerate}hz_{dst_channels}ch.pcm", );
    //     let mut writer = File::create(&output_file).unwrap();
    //     println!("opened output {output_file}");
    
    //     let max_frames = max_frames.unwrap_or(u64::MAX);
    //     let mut num_frames = 0_u64;
    
    //     for (track, packet) in ictx.packets() {
    //         if track.index() != i_index {
    //             continue;
    //         }
    
    //         decoder.send_packet(&packet).unwrap();
    
    //         while let Some(frame) = audio_decoder_receive_frame(&mut decoder).unwrap() {
    //             num_frames += 1;
    
    //             println!(
    //                 "Frame[{num_frames}]: {:?}, planes {}, samples {}", 
    //                 frame.format(),
    //                 frame.planes(),
    //                 frame.samples(),
    //             );
        
    //             let delay;
    //             let mut dst;
    //             {
    //                 dst = ff::frame::Audio::empty();
    //                 delay = resampler.run(&frame, &mut dst).unwrap();
    //             }
    //             println!("resample delay {delay:?}");
        
    //             // mixer.update_ch(&ch_id1, frame.clone().into()).unwrap();
    //             // mixer.update_ch(&ch_id2, frame.clone().into()).unwrap();
    //             // let dst = mixer.get_output().unwrap();
    //             // let dst = dst.frame();
    
    //             // let dst = frame;
        
    //             for index in 0..dst.planes() {
    //                 let plane = dst.plane::<i16>(index);
    //                 println!(
    //                     "  plane[{index}]: raw len {}, samples {}, plane<i16> {}", 
    //                     dst.data(index).len(), 
    //                     dst.samples(),
    //                     plane.len(),
    //                 );
    //                 writer.write_all(dst.data(index)).unwrap();
    //             }
    
    //             if num_frames >= max_frames {
    //                 break;
    //             }
    //         }
            
    //         if num_frames >= max_frames {
    //             break;
    //         }
    //     }
    
    //     println!("output {output_file}, wrote frames {num_frames}");
    // }
    
    
}

