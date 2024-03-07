// use ffmpeg_next as ff;

// pub type CodecId = ff::codec::Id;

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum CodecId {
    H264,
    H265,
    AAC,
    RtpRTX, // https://datatracker.ietf.org/doc/html/rfc4588
}


impl CodecId {
    pub fn parse_from_str(name: &str) -> Option<CodecId> {
        if name.eq_ignore_ascii_case("H264") {
            Some(CodecId::H264)
        } else if name.eq_ignore_ascii_case("H265") {
            Some(CodecId::H265)
        } else if name.eq_ignore_ascii_case("AAC") {
            Some(CodecId::AAC)
        } else if name.eq_ignore_ascii_case("MPEG4-GENERIC") {
            Some(CodecId::AAC)
        } else if name.eq_ignore_ascii_case("RTX") {
            Some(CodecId::RtpRTX)
        } 
        else {
            None
        }
    }
}

