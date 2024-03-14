
use ffmpeg_next as ff;

pub struct FFParameters{
    param: ff::codec::Parameters,
}

impl FFParameters {
    pub fn new() -> Self {
        Self {
            param: ff::codec::Parameters::new(),
        }
    }

    pub fn inner(&self) -> &ff::codec::Parameters {
        &self.param
    }

    pub fn set_from(&mut self, src: &Self) -> bool {
        match src.inner().medium() {
            ff::media::Type::Video => {
                self.set_video(
                    src.inner().id().into(), 
                    src.get_width(), 
                    src.get_height(), 
                    src.get_extra(),
                );
                true
            },
            ff::media::Type::Audio => {
                self.set_audio(
                    src.inner().id().into(), 
                    src.get_samplerate(), 
                    src.get_channels(),
                );
                true
            },
            _ => false

        }
    }
    
    pub fn set_video(
        &mut self,
        codec_id: ff::ffi::AVCodecID,
        width: i32,
        height: i32,
        extradata: &[u8],
    ) {
        self.set_codec((ff::ffi::AVMediaType::AVMEDIA_TYPE_VIDEO, codec_id));
        self.set_resolution((width, height));
        self.set_extra(extradata);
    }

    pub fn set_audio(
        &mut self,
        codec_id: ff::ffi::AVCodecID,
        samplerate: i32,
        channels: i32,
    ) {
        self.set_codec((ff::ffi::AVMediaType::AVMEDIA_TYPE_AUDIO, codec_id));
        self.set_samplerate(samplerate);
        self.set_channels(channels);
    }

    pub fn set_samplerate( &mut self, samplerate: i32 ) {
        unsafe {
            let ptr = self.param.as_mut_ptr();
            (*ptr).sample_rate = samplerate;
        }
    }

    pub fn get_samplerate(&self) -> i32 {
        unsafe {
            (*self.param.as_ptr()).sample_rate
        }
    }

    pub fn set_channels( &mut self, channels: i32 ) {
        unsafe {
            let ptr = self.param.as_mut_ptr();
            (*ptr).channels = channels;
        }
    }

    pub fn get_channels(&self) -> i32 {
        unsafe {
            (*self.param.as_ptr()).channels
        }
    }


    pub fn set_codec(&mut self, (medium, id): (ff::ffi::AVMediaType, ff::ffi::AVCodecID)) {
        unsafe {
            let ptr = self.param.as_mut_ptr();
            (*ptr).codec_type = medium;
            (*ptr).codec_id = id;
        }
    }

    pub fn get_codec(&self) -> (ff::ffi::AVMediaType, ff::ffi::AVCodecID){
        unsafe {
            let ptr = self.param.as_ptr();
            (
                (*ptr).codec_type,
                (*ptr).codec_id,
            )
        }
    }

    pub fn get_codec_type(&self) -> ff::ffi::AVMediaType {
        unsafe {
            let ptr = self.param.as_ptr();
            (*ptr).codec_type
        }
    }

    pub fn get_codec_id(&self) -> ff::ffi::AVCodecID {
        unsafe {
            let ptr = self.param.as_ptr();
            (*ptr).codec_id
        }
    }

    pub fn set_resolution(&mut self, (width, height): (i32, i32)) {
        unsafe {
            let ptr = self.param.as_mut_ptr();
            (*ptr).width = width;
            (*ptr).height = height;
        }
    }

    pub fn get_resolution(&self) -> (i32, i32){
        (self.get_width(), self.get_height())
    }

    pub fn get_width(&self) -> i32 {
        unsafe {
            (*self.param.as_ptr()).width
        }
    }

    pub fn get_height(&self) -> i32 {
        unsafe {
            (*self.param.as_ptr()).height
        }
    }

    pub fn set_framerate(&mut self, framerate: ff::ffi::AVRational) {
        unsafe {
            let ptr = self.param.as_mut_ptr();
            (*ptr).framerate = framerate;
        }
    }

    pub fn get_framerate(&self) -> ff::ffi::AVRational{
        unsafe {
            let ptr = self.param.as_ptr();
            (*ptr).framerate
        }
    }

    pub fn set_extra(&mut self, extradata: &[u8]) {
        unsafe {

            let new_alloc_size = extradata.len() + ff::ffi::AV_INPUT_BUFFER_PADDING_SIZE as usize;
            let new_extradata = ff::ffi::av_mallocz(new_alloc_size) as *mut u8;
            std::ptr::copy_nonoverlapping(extradata.as_ptr(), new_extradata, extradata.len());

            let ptr = self.param.as_mut_ptr();
            (*ptr).extradata = new_extradata;
            (*ptr).extradata_size = extradata.len() as i32;
        }
    }

    pub fn get_extra(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (*self.param.as_ptr()).extradata,
                (*self.param.as_ptr()).extradata_size as usize,
            )
        }
    }

    pub fn set_video_format(&mut self, pixel: ff::util::format::Pixel) {
        let format: ffmpeg_sys_next::AVPixelFormat = pixel.into();
        self.set_format(format as i32);
    }

    pub fn set_audio_format(&mut self, pixel: ff::util::format::Sample) {
        let format: ffmpeg_sys_next::AVSampleFormat = pixel.into();
        self.set_format(format as i32);
    }

    pub fn set_format(&mut self, format: i32) {
        unsafe {
            let ptr = self.param.as_mut_ptr();
            (*ptr).format = format;
        }
    }

    pub fn get_format(&self) -> i32 {
        unsafe {
            let ptr = self.param.as_ptr();
            (*ptr).format
        }
    }

    pub fn get_format_audio(&self) -> ff::ffi::AVSampleFormat {
        unsafe {
            let ptr = self.param.as_ptr();
            std::mem::transmute::<_, ff::ffi::AVSampleFormat>((*ptr).format)
        }
    }

    pub fn get_format_video(&self) -> ff::ffi::AVPixelFormat {
        unsafe {
            let ptr = self.param.as_ptr();
            std::mem::transmute::<_, ff::ffi::AVPixelFormat>((*ptr).format)
        }
    }

    pub fn set_frame_size(&mut self, frame_size: i32) {
        unsafe {
            let ptr = self.param.as_mut_ptr();
            (*ptr).frame_size = frame_size;
        }
    }

    pub fn get_frame_size(&self) -> i32 {
        unsafe {
            let ptr = self.param.as_ptr();
            (*ptr).frame_size
        }
    }

}

impl From<ff::codec::Parameters> for FFParameters {
    fn from(param: ff::codec::Parameters) -> Self {
        Self { param, }
    }
}

impl From<FFParameters> for ff::codec::Parameters {
    fn from(value: FFParameters) -> Self {
        value.param
    }
}
