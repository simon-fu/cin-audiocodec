
use ff::Rescale;
use ffmpeg_next as ff;
use std::path::Path;

use crate::ffeasy::{ffi::set_decoder_context_time_base, parameters::FFParameters};

pub struct InputVideoDecoder {
    ictx: ff::format::context::Input,
    ctx: DecoderContext,
    i_index: usize,
    decoder_time_base: ff::Rational,
}

impl InputVideoDecoder {
    pub fn open<P: AsRef<Path>>(path: &P) -> Result<Self, ff::Error> {
        let ictx = ff::format::input(path)?;

        let i_track = ictx.streams()
        .best(ff::util::media::Type::Video)
        .ok_or_else(||ff::Error::StreamNotFound)?;

        let i_params: FFParameters = i_track.parameters().into();
        let i_time_base = i_track.time_base();
        let i_index = i_track.index();
    
        println!(
            "input video codec1 {:?}, size {:?}, fps {:?}, extra {}, format {}",
            i_params.get_codec(),
            i_params.get_resolution(),
            i_params.get_framerate(),
            i_params.get_extra().len(),
            i_params.get_format(),
        );

        // // let decoder_params = i_params;

        // let mut decoder_params = FFParameters::new();
        // decoder_params.set_codec(i_params.get_codec());
        // decoder_params.set_resolution(i_params.get_resolution());
        // // decoder_params.set_framerate(i_params.get_framerate());
        // decoder_params.set_extra(i_params.get_extra());
        // decoder_params.set_format(i_params.get_format());
        // // println!(
        // //     "input video codec2 {:?}, size {:?}, fps {:?}, extra {}, format {}",
        // //     decoder_params.get_codec(),
        // //     decoder_params.get_resolution(),
        // //     decoder_params.get_framerate(),
        // //     decoder_params.get_extra().len(),
        // //     decoder_params.get_format(),
        // // );
    
        // let mut decoder = ff::codec::Context::new();
        // set_decoder_context_time_base(&mut decoder, i_time_base);
        // decoder.set_parameters(decoder_params)?;
        // let decoder = decoder.decoder().video()?;

        let codec = i_params.get_codec();
        let res = i_params.get_resolution();
        let decoder = make_video_decoder(codec.1.into(), res.0, res.1, i_params.get_extra(), i_time_base)?;

        let decoder_time_base = decoder.time_base();
        let resize_width = decoder.width();
        let resize_height = decoder.height();
    
        // println!("decoder: fmt {:?}, w {}, h {}, wxh {}", decoder.format(), decoder.width(), decoder.height(), decoder.width() * decoder.height(),);
        assert! (decoder.format() != ff::util::format::Pixel::None || decoder.width() > 0 || decoder.height() > 0);
    
        
        let scaler = ff::software::scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ff::util::format::Pixel::YUV420P,
            resize_width,
            resize_height,
            ff::software::scaling::Flags::AREA,
        )?;

        Ok(Self {
            ictx,
            ctx: DecoderContext {
                decoder,
                scaler,
            },
            i_index,
            decoder_time_base,
        })
    }

    pub fn frame_iter<'a>(&'a mut self) -> FrameIter<'a> {
        FrameIter {
            owner: self,
        }
    }

    pub fn next_frame(&mut self) -> Result<Option<ff::util::frame::Video>, ff::Error> {

        if let Some(frame) = self.ctx.pull_next_frame()? {
            return Ok(Some(frame))
        }

        for (i_track, mut packet) in self.ictx.packets() {
        
            if i_track.index() != self.i_index {
                continue;
            }
    
            packet.rescale_ts(i_track.time_base(), self.decoder_time_base);    
            
            let pts = packet.pts().map(|x|x.rescale(i_track.time_base(), self.decoder_time_base));
            let dts = packet.dts().map(|x|x.rescale(i_track.time_base(), self.decoder_time_base));
            packet.set_pts(pts);
            packet.set_dts(dts);
    
            self.ctx.decoder.send_packet(&packet)?;
            // num_packets += 1;

            let r = self.ctx.pull_next_frame()?;
            if let Some(r) = r {
                return Ok(Some(r))
            }
        }
        Ok(None)
    }

}

struct DecoderContext {
    decoder: ff::codec::decoder::Video,
    scaler: ff::software::scaling::Context,
}

impl DecoderContext {
    fn pull_next_frame(&mut self) -> Result<Option<ff::util::frame::Video>, ff::Error> {
        match video_decoder_receive_frame(&mut self.decoder)? {
            Some(frame) => {
                let frame = {
                    let mut frame_scaled = ff::util::frame::Video::empty();
                    self.scaler.run(&frame, &mut frame_scaled)?;
                    frame_scaled
                };
                Ok(Some(frame))
                // println!(
                //     "Packet[{num_packets}]: frame {num_frames}, {:?}, planes {}", 
                //     frame.format(),
                //     frame.planes(),
                // );
    
                
                // for plane in 0..frame.planes() {
                //     println!(
                //         "  plane[{plane}]: len {}, w {}, h {}, stride {}", 
                //         frame.data(plane).len(), 
                //         frame.plane_width(plane), frame.plane_height(plane),
                //         frame.stride(plane),
                //     );
                // }
            },
            None => Ok(None),
        }
    }
}

pub struct FrameIter<'a> {
    owner: &'a mut InputVideoDecoder,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = ff::util::frame::Video;

    fn next(&mut self) -> Option<Self::Item> {
        self.owner.next_frame().ok().unwrap_or(None)
    }
}

pub fn video_decoder_receive_frame(decoder: &mut ff::codec::decoder::Video) -> std::result::Result<Option<ff::util::frame::Video>, ff::util::error::Error> {
    let mut frame = ff::util::frame::Video::empty();
    let decode_result = decoder.receive_frame(&mut frame);
    match decode_result {
        Ok(()) => Ok(Some(frame)),
        Err(ff::util::error::Error::Other { errno }) if errno == ff::ffi::EAGAIN => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[derive(Debug, Clone)]
pub struct FFVideoArgs {
    pub codec_id: ff::codec::Id,
    pub width: i32,
    pub height: i32,
    pub extra: bytes::Bytes,
    pub time_base: ff::Rational,
}


pub fn make_video_decoder(
    codec_id: ff::codec::Id,
    width: i32,
    height: i32,
    extra: &[u8],
    time_base: ff::Rational,
) -> Result<ff::codec::decoder::Video, ff::Error> {

    let mut decoder_params = FFParameters::new();
    decoder_params.set_codec((ff::ffi::AVMediaType::AVMEDIA_TYPE_VIDEO, codec_id.into()));
    decoder_params.set_resolution((width, height));
    // decoder_params.set_framerate(i_params.get_framerate());
    decoder_params.set_extra(extra);
    decoder_params.set_video_format(ff::util::format::Pixel::YUV420P);

    println!(
        "input video codec2 {:?}, size {:?}, fps {:?}, extra {}, format {}",
        decoder_params.get_codec(),
        decoder_params.get_resolution(),
        decoder_params.get_framerate(),
        decoder_params.get_extra().len(),
        decoder_params.get_format(),
    );

    let mut decoder = ff::codec::Context::new();
    set_decoder_context_time_base(&mut decoder, time_base);
    decoder.set_parameters(decoder_params)?;
    let decoder = decoder.decoder().video()?;
    Ok(decoder)
}


