
use ffmpeg_next as ff;
// use ffmpeg_sys_next as sys;



// pub fn ff_extradata(param: &ffmpeg_next::codec::parameters::Parameters) -> Result<&[u8], ff::Error> {
//     Ok(unsafe {
//         std::slice::from_raw_parts(
//             (*param.as_ptr()).extradata,
//             (*param.as_ptr()).extradata_size as usize,
//         )
//     })
// }


pub fn ff_codec_context_as(codec: &ff::codec::codec::Codec) -> Result<ff::codec::context::Context, ff::Error> {
    unsafe {
        let context_ptr = ff::ffi::avcodec_alloc_context3(codec.as_ptr());
        if !context_ptr.is_null() {
            Ok(ff::codec::context::Context::wrap(context_ptr, None))
        } else {
            Err(ff::Error::Unknown)
        }
    }
}

pub fn set_decoder_context_time_base(decoder_context: &mut ff::codec::Context, time_base: ff::Rational) {
    unsafe {
        (*decoder_context.as_mut_ptr()).time_base = time_base.into();
    }
}

pub fn audio_frame_packed_i16_samples(frame: &ff::frame::Audio) -> &[i16] {
    let len = frame.samples() * 2 * frame.channels() as usize;
    let data = &frame.data(0)[..len];
    slice_u8_to_i16(data)
}

pub fn audio_frame_packed_i16_samples_mut(frame: &mut ff::frame::Audio) -> &mut [i16] {
    let len = frame.samples() * 2 * frame.channels() as usize;
    let data = &mut frame.data_mut(0)[..len];
    slice_u8_to_i16_mut(data)
}

pub fn audio_packed_i16_to_bytes(samples: &[i16]) -> &[u8] {
    unsafe { 
        std::slice::from_raw_parts(samples.as_ptr() as *const u8, samples.len() * 2) 
    }
}

fn slice_u8_to_i16(data: &[u8]) -> &[i16] {
    unsafe { 
        std::slice::from_raw_parts(data.as_ptr() as *const i16, data.len()/2) 
    }
}

fn slice_u8_to_i16_mut(data: &mut [u8]) -> &mut [i16] {
    unsafe { 
        std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut i16, data.len()/2) 
    }
}


// pub fn rtp_h264_mode_0(output: &ff::format::Output) -> bool {
//     unsafe {
//         sys::av_opt_flag_is_set(
//             (*output.as_ptr()).priv_data,
//             "rtpflags".as_ptr() as *const std::ffi::c_char,
//             "h264_mode0".as_ptr() as *const std::ffi::c_char,
//         ) != 0
//     }
// }

// pub fn rtp_seq_and_timestamp(output: &ff::format::Output) -> (u16, u32) {
//     unsafe {
//         let rtp_mux_context = &*((*output.as_ptr()).priv_data as *const RTPMuxContext);
//         (rtp_mux_context.seq, rtp_mux_context.timestamp)
//     }
// }

// /// Rust version of the `RTPMuxContext` struct in `libavformat`.
// #[repr(C)]
// struct RTPMuxContext {
//     _av_class: *const sys::AVClass,
//     _ic: *mut sys::AVFormatContext,
//     _st: *mut sys::AVStream,
//     pub payload_type: std::ffi::c_int,
//     pub ssrc: u32,
//     pub cname: *const std::ffi::c_char,
//     pub seq: u16,
//     pub timestamp: u32,
//     pub base_timestamp: u32,
//     pub cur_timestamp: u32,
//     pub max_payload_size: std::ffi::c_int,
// }

