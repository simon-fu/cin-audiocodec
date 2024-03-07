
use ffmpeg_next as ff;



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


