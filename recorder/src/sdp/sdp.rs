use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use enumflags2::{bitflags, BitFlags};

use crate::media::CodecId;


pub struct SdpMain {
    pub medias: Vec<SdpMedia>,
}

impl SdpMain {
    pub fn parse_from_str(sdp: &str) -> Result<Self> {
        let sdp = sdp_rs::SessionDescription::from_str(sdp)?; 
        Self::parse_typed_sdp(&sdp)
    }

    fn parse_typed_sdp(sdp: &sdp_rs::SessionDescription) -> Result<Self> {
        let mut medias = Vec::new();
        for (index, mdesc) in sdp.media_descriptions.iter().enumerate() {
            match mdesc.media.media {
                sdp_rs::lines::media::MediaType::Audio => {
                    medias.push(SdpMedia::Audio(parse_audio(mdesc, index)?));
                },
                sdp_rs::lines::media::MediaType::Video => {
                    medias.push(SdpMedia::Audio(parse_video(mdesc, index)?));
                },
                _ => {
                    medias.push(SdpMedia::Unknown);
                }
            }
        }
        Ok(Self { 
            medias, 
        })
    }
}


pub type SdpCodecId = CodecId;
pub type SdpProtoType = sdp_rs::lines::media::ProtoType;



pub enum SdpMedia {
    Video(SdpVideo),
    Audio(SdpAudio),
    Unknown,
}

impl SdpMedia {
    pub fn is_audio_or_video(&self) -> bool {
        match self {
            SdpMedia::Video(_) => true,
            SdpMedia::Audio(_) => true,
            SdpMedia::Unknown => false,
        }
    }
}

// #[derive(Debug, Clone)]
// pub enum SdpCodecType {
//     Video,
//     Audio,
//     Unknown,
// }

pub type SdpMediaType = sdp_rs::lines::media::MediaType;

#[derive(Debug, Clone)]
pub struct SdpCodec {
    pub payload_type: u8,
    pub codec_id: SdpCodecId,
    pub media_type: SdpMediaType,
    pub clock_rate: u32,
    pub channels: Option<u32>,
    pub rtcpfb: BitFlags<SdpRtcpfbFlags>,
    pub fmtps: Vec<String>,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SdpRtcpfbFlags {
    GoogRemb    = 0b_0000_0001,
    Nack        , // = 0b_0000_0010,
    NackPli     , // = 0b_0000_0100,
    CcmFir      , // = 0b_0000_1000,
    TransportCC , // = 0b_0001_0000,
}

// pub struct SdpRtpMap {

// }

#[derive(Debug)]
pub struct SdpAV {
    pub index: usize,
    pub port: u16,
    pub proto: SdpProtoType,
    pub payload_types: Vec<u8>,
    pub codecs: HashMap<u8, SdpCodec>,
}

pub type SdpVideo = SdpAV;

pub type SdpAudio = SdpAV;






fn parse_video(mdesc:&sdp_rs::MediaDescription, index: usize) -> Result<SdpAV> {
    parse_av(mdesc, index)
}

fn parse_audio(mdesc:&sdp_rs::MediaDescription, index: usize) -> Result<SdpAV> {
    parse_av(mdesc, index)
}

fn parse_av(mdesc:&sdp_rs::MediaDescription, index: usize) -> Result<SdpAV> {
    
    let mut payload_types = Vec::new();
    for s in mdesc.media.fmt.split_ascii_whitespace() {
        let v = s.trim().parse::<u8>().with_context(||"invalid media line")?;
        payload_types.push(v);
    }

    let mut media = SdpAV {
        index,
        port: mdesc.media.port,
        proto: mdesc.media.proto.clone(),
        payload_types,
        codecs: Default::default(),
    };

    for attr in mdesc.attributes.iter() {
        match attr {
            sdp_rs::lines::Attribute::Rtpmap(rtpmap)  => {

                if let Some(codec_id) = CodecId::parse_from_str(&rtpmap.encoding_name) {
                    media.codecs.insert(rtpmap.payload_type as u8, SdpCodec {
                        payload_type: rtpmap.payload_type as u8,
                        media_type: mdesc.media.media.clone(),
                        codec_id,
                        clock_rate: rtpmap.clock_rate as u32,
                        channels: rtpmap.encoding_params.map(|x| x as u32),
                        rtcpfb: Default::default(),
                        fmtps: Default::default(),
                    }) ;
                }

                // if rtpmap.encoding_name.eq_ignore_ascii_case("H264") {
                //     media.codecs.insert(rtpmap.payload_type as u8, SdpCodec {
                //         payload_type: rtpmap.payload_type as u8,
                //         codec_id: CodecId::H264,
                //         clock_rate: rtpmap.clock_rate as u32,
                //         channels: rtpmap.encoding_params.map(|x| x as u32),
                //         rtcpfb: Default::default(),
                //         fmtps: Default::default(),
                //     }) ;
                // }
            }
            sdp_rs::lines::Attribute::Other(name, value) => {
                if name == "fmtp" {
                    if let Some(value) = value {
                        let r = value.split_once(' ');
                        if let Some((num_str, fmtp)) = r {
                            let payload_type: u8 = num_str.parse().with_context(||"invalid fmtp for pt")?;
                            if let Some(codec) = media.codecs.get_mut(&payload_type) {
                                codec.fmtps.push(fmtp.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

    }
    Ok(media)
}


#[test]
fn test_sdp() {
    let sdp = indoc::indoc!{
        "v=0
        o=- 0 0 IN IP4 127.0.0.1
        s=No Name
        t=0 0
        a=tool:libavformat 60.16.100
        m=video 0 RTP/AVP 96
        b=AS:640
        a=rtpmap:96 H264/90000
        a=fmtp:96 packetization-mode=1; sprop-parameter-sets=Z0LAFdoCAJbARAAAAwAEAAADAPA8WLqA,aM4yyA==; profile-level-id=42C015
        a=control:streamid=0
        m=audio 0 RTP/AVP 97
        b=AS:96
        a=rtpmap:97 MPEG4-GENERIC/48000/2
        a=fmtp:97 profile-level-id=1;mode=AAC-hbr;sizelength=13;indexlength=3;indexdeltalength=3; config=119056E500
        a=control:streamid=1
        "
    };
    SdpMain::parse_from_str(sdp).unwrap();
}
