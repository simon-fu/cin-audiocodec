
use ff::{ChannelLayout, Rescale};
use ffmpeg_next as ff;
use std::path::Path;

use crate::ffeasy::{ffi::set_decoder_context_time_base, parameters::FFParameters, time::FFTimeResaler};

use super::swr::SResampler;

// use super::resampler::FFResampler;

pub type AudioArgs = ff::software::resampling::context::Definition;



pub struct InputAudioDecoder {
    ictx: ff::format::context::Input,
    ctx: DecoderContext,
    i_index: usize,
    decoder_time_base: ff::Rational,
    i_params: FFParameters,
}

impl InputAudioDecoder {
    pub fn open<P: AsRef<Path>>(path: &P, _target: Option<AudioArgs>) -> Result<Self, ff::Error> {
        let ictx = ff::format::input(path)?;

        let i_track = ictx.streams()
        .best(ff::util::media::Type::Audio)
        .ok_or_else(||ff::Error::StreamNotFound)?;

        let i_params: FFParameters = i_track.parameters().into();
        let i_time_base = i_track.time_base();
        let i_index = i_track.index();
    
        println!(
            "input audio codec1 {:?}, samplerate {:?}, ch {:?}, format {:?}, time_base {:?}",
            i_params.get_codec(),
            i_params.get_samplerate(),
            i_params.get_channels(),
            i_params.get_format_audio(),
            i_time_base,
        );


        let codec = i_params.get_codec();
        let decoder = make_audio_decoder(codec.1.into(), i_params.get_samplerate(), i_params.get_channels(), i_time_base)?;

        let decoder_time_base = decoder.time_base();
    
    
        // let resampler = match target {
        //     Some(args) => {
        //         Some(ff::software::resampler(
        //             (
        //                 i_params.get_format_audio().into(),
        //                 ff::ChannelLayout::default(i_params.get_channels()),
        //                 i_params.get_samplerate() as u32,
        //             ),
        //             (
        //                 args.format,
        //                 args.channel_layout,
        //                 args.rate,
        //             ),
        //         )?)
        //     },
        //     None => None,
        // };

        // let convert = match target {
        //     Some(args) => {
        //         // FFResampler::try_new_packed(
        //         //     src_channels, 
        //         //     src_rate, 
        //         //     dst_channels, 
        //         //     dst_rate, 
        //         //     frame_milli, 
        //         //     buf_milli,
        //         // )?,
        //         let resampler =  ff::software::resampler(
        //             (
        //                 i_params.get_format_audio().into(),
        //                 ff::ChannelLayout::default(i_params.get_channels()),
        //                 i_params.get_samplerate() as u32,
        //             ),
        //             (
        //                 args.format,
        //                 args.channel_layout,
        //                 args.rate,
        //             ),
        //         )?;

        //         Some(SampleTsConvert {
        //             src_base: None,
        //             last_dst_ts: None,
        //             src_samples: 0,
        //             dst_samples: 0,
        //             src_rate: i_params.get_samplerate() as u32,
        //             dst_rate: args.rate,
        //             resampler,
        //         })
        //     },
        //     None => None,
        // };


        Ok(Self {
            ictx,
            ctx: DecoderContext {
                decoder,
                // convert,
                // time_base: decoder_time_base,
            },
            i_index,
            decoder_time_base,
            i_params,
        })
    }

    pub fn get_format(&self) -> ff::format::Sample {
        self.i_params.get_format_audio().into()
    }

    pub fn get_samplerate(&self) -> u32 {
        self.i_params.get_samplerate() as u32
    }

    pub fn get_channels(&self) -> u32 {
        self.i_params.get_channels() as u32
    }

    pub fn get_time_base(&self) -> ff::Rational {
        self.decoder_time_base
    }

    pub fn frame_iter<'a>(&'a mut self) -> impl Iterator<Item = Decoded> + 'a {
        FrameIter {
            owner: self,
        }
    }

    pub fn next_frame(&mut self) -> Result<Option<Decoded>, ff::Error> {

        if let Some(frame) = self.ctx.pull_next_frame()? {
            return Ok(Some(frame))
        }

        for (i_track, mut packet) in self.ictx.packets() {
        
            if i_track.index() != self.i_index {
                continue;
            }
    
            // packet.rescale_ts(i_track.time_base(), self.decoder_time_base);    
            
            let pts = packet.pts().map(|x|x.rescale(i_track.time_base(), self.decoder_time_base));
            let dts = packet.dts().map(|x|x.rescale(i_track.time_base(), self.decoder_time_base));
            packet.set_pts(pts);
            packet.set_dts(dts);

            println!("audio: send packet pts {pts:?}, dts {dts:?}");
    
            self.ctx.decoder.send_packet(&packet)?;
            // num_packets += 1;
            let r = self.ctx.pull_next_frame()?;
            if let Some(r) = r {
                return Ok(Some(r))
            }
            
        }
        Ok(None)
    }

    pub fn time_scaler(&self, dst: ff::Rational) -> FFTimeResaler {
        FFTimeResaler::new(self.decoder_time_base, dst)
    }

    pub fn resampler(&self, target: &AudioArgs) -> Result<SResampler, ff::Error> {
        SResampler::get(
            self.get_format(), 
            ChannelLayout::default(self.get_channels() as i32), 
            self.get_samplerate(), 
            target.format, 
            target.channel_layout, 
            target.rate,
        )
    }
}


// struct SampleTsConvert {
//     resampler: ff::software::resampling::Context,
//     src_base: Option<i64>,
//     last_dst_ts: Option<i64>,
//     src_samples: u64,
//     dst_samples: u64,
//     src_rate: u32,
//     dst_rate: u32,
// }

// impl SampleTsConvert {
//     // pub fn convert(&mut self, src: i64, samples: usize) -> i64 {
//     //     self.convert_opt(Some(src), samples).unwrap_or(src)
//     // }



//     pub fn convert_opt(&mut self, src: Option<i64>, samples: usize) -> (Option<i64>, usize) {
//         let src = match src {
//             Some(v) => v,
//             None => {
//                 let dst_samples = self.convert_samples(samples, self.last_dst_ts);
//                 return (None, dst_samples)
//             },
//         };

//         let dst_ts = self.convert_ts(src);

//         let dst_samples = self.convert_samples(samples, Some(dst_ts));

//         self.last_dst_ts = None;

//         (Some(dst_ts), dst_samples)
//     }

//     fn convert_ts(&mut self, src: i64) -> i64 {
//         match self.src_base {
//             Some(base) => {
                
//                 let ts = base + samples_to_millis_u64(self.src_rate, self.src_samples);

//                 let diff = ts.abs_diff(src);
//                 if diff > 5 {
//                     println!("aaa exceed diff {diff}, expect {ts} but {src}");
//                     self.src_base = Some(src);
//                     self.src_samples = 0;
//                     src
//                 } else {
//                     ts
//                 }

//             },
//             None => {
//                 self.src_base = Some(src);
//                 self.src_samples = 0;
//                 src
//             },
//         }
//     }

//     fn convert_samples(&mut self, samples: usize, dst_ts: Option<i64>) -> usize {
//         let num_samples = self.src_samples + samples as u64;
//         let total_millis = samples_to_millis_u64(self.src_rate, num_samples);
//         let dst_total_samples = millis_to_samples(self.dst_rate, total_millis) as u64;
//         let delta_samples = dst_total_samples - self.dst_samples;
//         self.dst_samples = dst_total_samples;
//         self.src_samples += samples as u64;
//         delta_samples as usize
//     }

// }

struct DecoderContext {
    decoder: ff::codec::decoder::Audio,
    // resampler: Option<ff::software::resampling::Context>,
    // resampler: Option<FFResampler<i16, i16>>,
    // convert: Option<SampleTsConvert>,
    // time_base: ff::Rational,
}

impl DecoderContext {
    pub fn pull_next_frame(&mut self) -> Result<Option<Decoded>, ff::Error> {
        match audio_decoder_receive_frame(&mut self.decoder)? {
            Some(frame) => {
                Ok(Some(frame))
            },
            None => Ok(None),
        }
    }
}

// fn samples_to_millis(rate: u32, len: usize) -> i64 {
//     1000 * (len as i64) / (rate as i64)
// }

// fn samples_to_millis_u64(rate: u32, len: u64) -> i64 {
//     1000 * (len as i64) / (rate as i64)
// }

// fn millis_to_samples(rate: u32, millis: i64) -> usize {
//     (millis as usize) * (rate as usize) / 1000
// }

// pub type Resampled = (Option<ff::software::resampling::Delay>, ff::util::frame::Audio);
pub type Decoded = ff::util::frame::Audio;

pub struct FrameIter<'a> {
    owner: &'a mut InputAudioDecoder,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = Decoded;

    fn next(&mut self) -> Option<Self::Item> {
        self.owner.next_frame().ok().unwrap_or(None)
    }
}


pub fn make_audio_decoder(
    codec_id: ff::codec::Id,
    samplerate: i32,
    channels: i32,
    time_base: ff::Rational,
) -> Result<ff::codec::decoder::Audio, ff::Error> {

    let mut decoder_params = FFParameters::new();
    decoder_params.set_codec((ff::ffi::AVMediaType::AVMEDIA_TYPE_AUDIO, codec_id.into()));
    decoder_params.set_samplerate(samplerate);
    decoder_params.set_channels(channels);
    // decoder_params.set_audio_format(ff::util::format::Sample::I16(ff::util::format::sample::Type::Packed));

    println!(
        "input audio codec2 {:?}, samplerate {:?}, ch {:?}, format {:?}, time_base {time_base:?}",
        decoder_params.get_codec(),
        decoder_params.get_samplerate(),
        decoder_params.get_channels(),
        decoder_params.get_format_audio(),
    );

    let mut decoder = ff::codec::Context::new();
    set_decoder_context_time_base(&mut decoder, time_base);
    decoder.set_parameters(decoder_params)?;
    let decoder = decoder.decoder().audio()?;
    Ok(decoder)
}

pub fn audio_decoder_receive_frame(decoder: &mut ff::codec::decoder::Audio) -> std::result::Result<Option<ff::util::frame::Audio>, ff::util::error::Error> {
    let mut frame = ff::util::frame::Audio::empty();
    let decode_result = decoder.receive_frame(&mut frame);
    match decode_result {
        Ok(()) => Ok(Some(frame)),
        Err(ff::util::error::Error::Other { errno }) if errno == ff::ffi::EAGAIN => Ok(None),
        Err(err) => Err(err.into()),
    }
}

