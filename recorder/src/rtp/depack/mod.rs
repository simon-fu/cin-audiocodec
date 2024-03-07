

use anyhow::Result;
use bytes::Bytes;

use crate::sdp::sdp::{SdpCodec, SdpCodecId};

use super::codec::h264::RtpDepackerH264;

pub mod depack_simple;

pub trait RtpCodecDepacker {
    fn push_rtp_slice(&mut self, rtp: &[u8]) -> Result<()> ;
    fn pull_frame(&mut self) -> Result<Option<Bytes>> ;
}


pub fn make_rtp_depacker(codec: &SdpCodec) -> Result<Option<Box<dyn RtpCodecDepacker>>>  {
    match codec.codec_id {
        SdpCodecId::H264 => {
            let fmtp = codec.fmtps.get(0).map(|x|x.as_str());
            Ok(Some(RtpDepackerH264::new(fmtp)?.into_box()))
        }
        _ => Ok(None),
    }
}

// pub fn make_rtp_depackers_from_sdp(sdp: &str) -> Result<Box<dyn GetDepacker>> {

//     let mut tracks: Vec<HashMap<u8, CodecItem>> = Vec::new();

//     let sdp = SdpMain::parse_from_str(sdp)?;
//     for media in sdp.medias.iter() {
//         match media {
//             SdpMedia::Video(media) => {
//                 let mut track = HashMap::new();
//                 for (_pt, codec) in media.codecs.iter() {
//                     if let Some(depacker) = make_rtp_depacker(codec)? {
//                         track.insert(codec.payload_type, CodecItem {
//                             codec_id: codec.codec_id,
//                             depacker,
//                         });
//                     }
//                 }
//                 tracks.push(track);
//             },
//             SdpMedia::Audio(media) => {},
//             SdpMedia::Unknown => {},
//         }
//     }


//     Ok(Box::new(RtpDivideDeapackers {
//         tracks,
//     }))

//     // let sdp = sdp_rs::SessionDescription::from_str(sdp)?; 
//     // for media in sdp.media_descriptions.iter() {
        
//     //     for attr in media.attributes.iter() {
//     //         match attr {
//     //             sdp_rs::lines::Attribute::Rtpmap(rtpmap)  => {
//     //                 if rtpmap.encoding_name.eq_ignore_ascii_case("H264") {
//     //                     let RtpDepackerH264::new();
//     //                 }
//     //             }
//     //             sdp_rs::lines::Attribute::Other(key, value) => {
//     //                 if key == "fmtp" {
//     //                     if let Some(value) = value {
//     //                         let r = value.split_once(' ');
//     //                         if let Some((num_str, fmtp)) = r {
//     //                             let payload_type: Option<u8> = num_str.parse().ok();
//     //                             if payload_type == h264_payload_type {
//     //                                 return Ok(payload_type.map(|x|(x, fmtp.to_string())))
//     //                             }
//     //                         }
//     //                     }
//     //                 }
//     //             }
//     //             _ => {}
//     //         }

//     //     }
//     // }

//     // Ok(None)
// }



// pub trait GetDepacker {
//     fn get_depacker<'a>(&'a mut self, index: usize, payload_type: u8) -> Option<DepackerMut<'a>>;
// }

// pub struct DepackerMut<'a> {
//     pub track_index: usize,
//     pub codec_id: CodecId,
//     pub depacker: &'a mut Box<dyn RtpCodecDepacker>,
// }

// pub struct RtpDivideDeapackers {
//     tracks: Vec<HashMap<u8, CodecItem>>,
// }

// impl RtpDivideDeapackers {
//     pub fn into_bundle(self) -> RtpBundleDeapackers {
//         let mut map = HashMap::new();
//         for (index, track) in self.tracks.into_iter().enumerate() {
//             for (payload_type, depack) in track {
//                 map.insert(payload_type, (index, CodecItem {
//                     codec_id: depack.codec_id,
//                     depacker: depack.depacker,
//                 }));
//             }
//         }
//         RtpBundleDeapackers {
//             map,
//         }
//     }
// }

// impl GetDepacker for RtpDivideDeapackers {
//     fn get_depacker<'a>(&'a mut self, index: usize, payload_type: u8) -> Option<DepackerMut<'a>> {
//         if let Some(track) = self.tracks.get_mut(index) {
//             if let Some(depack) = track.get_mut(&payload_type) {
//                 return Some(DepackerMut {
//                     track_index: index,
//                     codec_id: depack.codec_id,
//                     depacker: &mut depack.depacker,
//                 })
//             }
//         }
//         None
//     }
// }

// pub struct RtpBundleDeapackers {
//     map: HashMap<u8, (usize, CodecItem)>,
// }

// impl GetDepacker for RtpBundleDeapackers {
//     fn get_depacker<'a>(&'a mut self, _index: usize, payload_type: u8) -> Option<DepackerMut<'a>> {
//         self.map.get_mut(&payload_type)
//         .map(|item| DepackerMut{
//             track_index: item.0,
//             codec_id: item.1.codec_id,
//             depacker: &mut item.1.depacker,
//         })
//     }
// }

// struct CodecItem {
//     codec_id: CodecId,
//     depacker: Box<dyn RtpCodecDepacker>,
// }

