use ff::ChannelLayout;
use ffmpeg_next as ff;

use super::{ffi::ff_codec_context_as, parameters::FFParameters};

pub struct FFVideoEncoder {
    encoder: ff::codec::encoder::video::Encoder,
}

impl FFVideoEncoder {
    
    pub fn h264(
        width: u32,
        height: u32,
        frame_rate: i32,
        pixel_format: ff::util::format::Pixel,
        realtime: bool,
        global_header: bool,
    ) -> Result<Self, ff::Error> {
        let codec = ff::encoder::find_by_name("libx264")
            .or(ff::encoder::find(ff::codec::Id::H264));
    
        let mut encoder_context = match codec {
            Some(codec) => ff_codec_context_as(&codec)?,
            None => ff::codec::Context::new(),
        };

        if global_header {
            encoder_context.set_flags(ff::codec::Flags::GLOBAL_HEADER);
        }
    
        let mut encoder = encoder_context.encoder().video()?;
    
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(pixel_format);
        encoder.set_frame_rate(Some((frame_rate, 1)));
    
        encoder.set_time_base(ff::util::mathematics::rescale::TIME_BASE);
    
        let mut opts = ff::Dictionary::new();
        if realtime {
            opts.set("preset", "medium");
            opts.set("tune", "zerolatency");
        } else {
            opts.set("preset", "medium");
        }
    

        let encoder = encoder.open_with(opts.clone())?;
        Ok(Self { encoder, })
    }

    pub fn send_frame(&mut self, frame: &ff::Frame) -> Result<(), ff::Error> {
        self.encoder.send_frame(frame)
    }

    pub fn send_eof(&mut self) -> Result<(), ff::Error> {
        self.encoder.send_eof()
    }


    pub fn receive_packet(&mut self) -> Result<Option<ff::codec::packet::Packet>, ff::Error> {
        let mut packet = ff::codec::packet::Packet::empty();
        let encode_result = self.encoder.receive_packet(&mut packet);
        match encode_result {
            Ok(()) => Result::<_, ff::Error>::Ok(Some(packet)),
            Err(ff::Error::Other { errno }) if errno == ff::util::error::EAGAIN => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn flush_receive_packet(&mut self) -> Result<Option<ff::codec::packet::Packet>, ff::Error> {
        let r = self.receive_packet();
        match r {
            Ok(r) => {
                Ok(r)
            },
            Err(ff::Error::Eof) => {
                Ok(None)
            }
            Err(e) => {
                Err(e)
            },
        }
    }

    pub fn into_inner(self) -> ff::codec::encoder::video::Encoder {
        self.encoder
    }

    pub fn inner(&self) -> &ff::codec::encoder::video::Encoder {
        &self.encoder
    }

    // pub fn inner_mut(&mut self) -> &mut ff::codec::encoder::video::Encoder {
    //     &mut self.encoder
    // }

    pub fn get_time_base(&self) -> ff::Rational {
        unsafe { (*self.encoder.0.as_ptr()).time_base.into() }
    }

    pub fn get_parameters(&self) -> FFParameters {
        ff::codec::Parameters::from(&self.encoder).into()
    }


}


pub struct FFAudioEncoder {
    encoder: ff::codec::encoder::audio::Encoder,
}

impl FFAudioEncoder {
    
    pub fn aac(
        samplerate: u32,
        channels: u32,
        format: ff::format::Sample,
        global_header: bool,
    ) -> Result<Self, ff::Error> {
        
        let codec = ff::encoder::find(ff::codec::Id::AAC);
    
        let mut encoder_context = match codec {
            Some(codec) => ff_codec_context_as(&codec)?,
            None => ff::codec::Context::new(),
        };

        if global_header {
            encoder_context.set_flags(ff::codec::Flags::GLOBAL_HEADER);
        }
    
        let mut encoder = encoder_context.encoder().audio()?;
    
        encoder.set_format(format);
        encoder.set_rate(samplerate as i32);
        encoder.set_channels(channels as i32);
        encoder.set_channel_layout(ChannelLayout::default(channels as i32));

        encoder.set_time_base(ff::util::mathematics::rescale::TIME_BASE);
    
        let opts = ff::Dictionary::new();
        let encoder = encoder.open_with(opts.clone())?;
        Ok(Self { encoder, })
    }

    pub fn send_frame(&mut self, frame: &ff::Frame) -> Result<(), ff::Error> {
        self.encoder.send_frame(frame)
    }

    pub fn send_eof(&mut self) -> Result<(), ff::Error> {
        self.encoder.send_eof()
    }


    pub fn receive_packet(&mut self) -> Result<Option<ff::codec::packet::Packet>, ff::Error> {
        let mut packet = ff::codec::packet::Packet::empty();
        let encode_result = self.encoder.receive_packet(&mut packet);
        match encode_result {
            Ok(()) => Result::<_, ff::Error>::Ok(Some(packet)),
            Err(ff::Error::Other { errno }) if errno == ff::util::error::EAGAIN => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn flush_receive_packet(&mut self) -> Result<Option<ff::codec::packet::Packet>, ff::Error> {
        let r = self.receive_packet();
        match r {
            Ok(r) => {
                Ok(r)
            },
            Err(ff::Error::Eof) => {
                Ok(None)
            }
            Err(e) => {
                Err(e)
            },
        }
    }

    pub fn into_inner(self) -> ff::codec::encoder::audio::Encoder {
        self.encoder
    }

    pub fn inner(&self) -> &ff::codec::encoder::audio::Encoder {
        &self.encoder
    }

    // pub fn inner_mut(&mut self) -> &mut ff::codec::encoder::video::Encoder {
    //     &mut self.encoder
    // }

    pub fn get_time_base(&self) -> ff::Rational {
        unsafe { (*self.encoder.0.as_ptr()).time_base.into() }
    }

    pub fn get_parameters(&self) -> FFParameters {
        ff::codec::Parameters::from(&self.encoder).into()
    }


}



// pub struct EasyEncoder {
//     encoder: ff::codec::encoder::video::Video,
//     encoder_time_base: ff::Rational,
//     scaler: ff::software::scaling::context::Context,
//     scaler_width: u32,
//     scaler_height: u32,
//     frame_count: u64,
// }

// impl EasyEncoder {
//     pub fn h264(
//         pic_width: u32,
//         pic_height: u32,
//         frame_rate: i32,
//         input_format: ff::util::format::Pixel,
//         encoder_pixel_format: ff::util::format::Pixel,
//         opts: ff::Dictionary,
//     ) -> Result<Self, ff::Error> {
//         let codec = ff::encoder::find_by_name("libx264")
//         .or(ff::encoder::find(ff::codec::Id::H264));

//         let mut encoder_context = match codec {
//             Some(codec) => vide_rs_emul::codec_context_as(&codec).unwrap(),
//             None => ff::codec::Context::new(),
//         };

//         let mut encoder = encoder_context.encoder().video().unwrap();

//         encoder.set_width(pic_width);
//         encoder.set_height(pic_height);
//         encoder.set_format(encoder_pixel_format);
//         encoder.set_frame_rate(Some((frame_rate, 1)));

//         encoder.set_time_base(ff::util::mathematics::rescale::TIME_BASE);

//         let mut encoder = encoder.open_with(opts.clone()).unwrap();
//         let encoder_time_base = vide_rs_emul::get_encoder_time_base(&encoder);

//         let scaler_width = encoder.width();
//         let scaler_height = encoder.height();

//         let mut scaler = ff::software::scaling::context::Context::get(
//             FRAME_PIXEL_FORMAT,
//             scaler_width,
//             scaler_height,
//             encoder.format(),
//             scaler_width,
//             scaler_height,
//             ff::software::scaling::flag::Flags::empty(),
//         ).unwrap();

//     }
// }


// mod vide_rs_emul {

//     use ffmpeg_next as ff;
//     use ndarray::Array3;

//     pub fn codec_context_as(codec: &ff::codec::codec::Codec) -> Result<ff::codec::context::Context, ff::Error> {
//         unsafe {
//             let context_ptr = ff::ffi::avcodec_alloc_context3(codec.as_ptr());
//             if !context_ptr.is_null() {
//                 Ok(ff::codec::context::Context::wrap(context_ptr, None))
//             } else {
//                 Err(ff::Error::Unknown)
//             }
//         }
//     }

// }
