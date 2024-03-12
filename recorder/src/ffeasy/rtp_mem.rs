
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

// use anyhow::Context;
use bytes::Bytes;

use tokio::time::Instant;
use video_rs::Locator;
use video_rs::Options;
use video_rs::Reader;
use video_rs::RtpBuf;
use video_rs::RtpMuxer;
use video_rs::StreamInfo;


// use crate::oddity_rtsp_server as thiz_root;
// use thiz_root::media::MediaDescriptor;
// use oddity_sdp_protocol::{CodecInfo, Direction, Kind, Protocol, TimeRange};
// use crate::oddity_rtsp_server::media::video::reader::backend::make_reader_with_sane_settings;
// pub use oddity_sdp_protocol::Sdp;

use anyhow::Result;

pub fn load_rtp_mem_sync(filename: &Path, max_frames: u64) -> Result<Arc<RtpMemData>> {
    // // let filename = "/tmp/sample-data/sample.mp4";
    // let source_name = "MemSource";
    // let source_descriptor = MediaDescriptor::File(filename.into());

    // const FMT_RTP_PAYLOAD_DYNAMIC: u8 = 96;
    // let mut pt = FMT_RTP_PAYLOAD_DYNAMIC;


    // 因为 .h264 不支持 reader.seek_to_start(); 这里创建了两个reader

    let opts = Options::new_from_hashmap(&[
        ("bsf:v".to_string(), "extract_extradata".to_string()),
        ("bsf:v".to_string(), "dump_extra".to_string()),
    ].into());
    let mut video_reader = Reader::new_with_options(&Locator::Path(filename.into()), &opts)?;

    // let mut video_reader = Reader::new(&Locator::Path(filename.into()))?;
    let mut audio_reader = Reader::new(&Locator::Path(filename.into()))?;

    // let mut video_reader = make_reader_with_sane_settings(source_descriptor.clone().into()).await?;

    // let mut audio_reader = make_reader_with_sane_settings(source_descriptor.clone().into()).await?;

    let mut func = move || {
        let mut ch_id = 0;
        let video = match best_media_index(&video_reader, ffmpeg_next::media::Type::Video) {
            Some(index) => {
                let track = load_track(&mut video_reader, index, ch_id, max_frames)?;
                ch_id += 2;
                // pt += 1;
                Some(track)
            },
            None => None,
        };
    
        let audio = match best_media_index(&audio_reader, ffmpeg_next::media::Type::Audio) {
            Some(index) => {
                // TODO： 
                //   按道理 audio_reader 是新创建的，不需要 seek_to_start
                //   但实际上，如果不 seek_to_start ， mp4 文件读出来的时间戳是负数
                // let _r = audio_reader.seek_to_start();
                let track = load_track(&mut audio_reader, index, ch_id, max_frames)?;
                // ch_id += 2;
                // pt += 1;
                Some(track)
            },
            None => None,
        };
    
        // let sdp = make_sdp(source_name, &video, &audio)?;
    
        Ok(Arc::new(RtpMemData {
            sdp: Bytes::new(),
            video,
            audio,
        }))
    };
    func()

}

pub struct RtpMemReader {
    data: Arc<RtpMemData>,
    cursor: RtpMemCursor,
    ts_base: Option<TsBase>,
}

impl RtpMemReader {
    pub fn data(&self) -> &Arc<RtpMemData> {
        &self.data
    }

    pub async fn pace_read(&mut self,) -> Option<RtpMemPacket> {
        let r = self.read_next();
        if let Some(packet) = r {
            match &self.ts_base {
                Some(ts_base) => {
                    if let Some(d) = ts_base.check(packet.dts) {
                        tokio::time::sleep(d).await;
                    }
                },
                None => {
                    self.ts_base = Some(TsBase {
                        instant: Instant::now(),
                        ts: packet.dts,
                    });
                },
            }

            return Some(packet.clone())
        }

        None
    }

    pub fn read_next(&mut self) -> Option<RtpMemPacket> {
        self.data.read_at(&mut self.cursor)
    }
}

pub struct RtpMemData {
    sdp: Bytes,
    video: Option<MemTrack>,
    audio: Option<MemTrack>,
}

impl RtpMemData {
    pub fn video(&self)-> &Option<MemTrack> {
        &self.video
    }
    
    pub fn sdp(&self) -> &Bytes {
        &self.sdp
    }

    pub fn track_index_of(&self, control: &str) -> Option<usize> {
        if let Some(track) = self.video.as_ref() {
            if control == track.control {
                return Some(track.ch_track_index())
            }
        }

        if let Some(track) = self.audio.as_ref() {
            if control == track.control {
                return Some(track.ch_track_index())
            }
        }

        None
    }

    pub fn read_at(&self, cursor: &mut RtpMemCursor) -> Option<RtpMemPacket> {
        let video = self.video.as_ref().map(|x|x.packets.get(cursor.video)).unwrap_or(None);

        let audio = self.audio.as_ref().map(|x|x.packets.get(cursor.audio)).unwrap_or(None);
        
        if let (Some(video), Some(audio)) = (video, audio) {
            if video.dts <= audio.dts {
                cursor.video += 1;
                return Some(video.clone())
            } else {
                cursor.audio += 1;
                return Some(audio.clone())
            }
        } else if let Some(video) = video {

            cursor.video += 1;
            return Some(video.clone())

        } else if let Some(audio) = audio {

            cursor.audio += 1;
            return Some(audio.clone())
        } else {
            return None
        }
    }

    pub fn make_reader(self: &Arc<Self>) -> RtpMemReader {
        RtpMemReader {
            data: self.clone(),
            cursor: Default::default(),
            ts_base: None,
        }
    }

}


#[derive(Default, Debug)]
pub struct RtpMemCursor {
    video: usize,
    audio: usize,
}

#[derive(Clone)]
pub struct RtpMemPacket {
    pts: i64,
    dts: i64,
    ch_id: u64,
    data: Bytes,
}

impl RtpMemPacket {

    pub fn set_ts(&mut self, pts: i64, dts: i64) {
        self.pts = pts;
        self.dts = dts;
    }

    pub fn pts(&self) -> i64 {
        self.pts
    }

    pub fn set_ch_id(&mut self, ch_id: u64) {
        self.ch_id = ch_id;
    }

    pub fn dts(&self) -> i64 {
        self.dts
    }

    pub fn ch_id(&self) -> u64 {
        self.ch_id
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }
    
    fn from_rtp_buf(buf: RtpBuf, ch_id: u64, pts: i64, dts: i64) -> Self {
        // let ch_id = (index << 1) as u8;

        match buf {
            RtpBuf::Rtp(data) => {
                // let d1 = data[1];
                // data[1] = (d1 & 0x80) | (pt & 0x7F);

                Self {
                    pts,
                    dts,
                    ch_id: ch_id + 0,
                    data: Bytes::from(data),
                }
            },
            RtpBuf::Rtcp(data) => Self {
                pts,
                dts,
                ch_id: ch_id + 1,
                data: Bytes::from(data),
            },
        }
    }
}



struct TsBase {
    instant: Instant,
    ts: i64,
}

impl TsBase {
    fn check(&self, next: i64) -> Option<Duration> {
        if next > self.ts {
            let ts_delta = next - self.ts;
            let ts_delta = Duration::from_millis(ts_delta as u64);
            let elapsed = self.instant.elapsed();
            if ts_delta > elapsed {
                return Some(ts_delta - elapsed)
            }
        }
        None
    }
}

fn load_track(reader: &mut Reader, index: usize, ch_id: u64, max_frames: u64) -> Result<MemTrack> {
    load_track1(reader, index, ch_id, max_frames)
}

fn load_track1(reader: &mut Reader, index: usize, ch_id: u64, max_frames: u64) -> Result<MemTrack> {

    // let val = reader
    // .input
    // .stream(index).with_context(||"NOT found stream")?
    // .time_base();
    // println!("track[{index}]: time_base {val:?}");

    let mut pt = 0;
    let info = reader.stream_info(index)?;
    let control = format!("track_id={index}");


    let mut muxer = RtpMuxer::new()?
    .with_stream(info.clone())?;


    let spspps: Vec<(SpsBytes, Vec<PpsBytes>)> = muxer.parameter_sets_h264()
    .into_iter()
    .filter_map(|x|x.ok())
    .map(|x|(
        x.0.to_vec(), 
        x.1.into_iter().map(|v|v.to_vec()).collect::<Vec<VecBytes>>() 
    ))
    .collect();

    let packets = {
        let mut packets = Vec::new();
        let mut pts_gen = TsGen::new(40);
        let mut dts_gen = TsGen::new(40);

        for _num in 0..max_frames {
            let frame = match reader.read(index){
                Ok(v) => v,
                Err(_e) => break,
            };
            let pts = pts_gen.convert(frame.pts().into_parts());
            let dts = dts_gen.convert(frame.pts().into_parts());

            // let pts = Duration::from(frame.pts()).as_millis() as i64;
            // let dts = Duration::from(frame.dts()).as_millis() as i64;
            // println!("rtpmem: ch[{ch_id}] frame {_num}: pts {}, dts {}", pts, dts);
            let rtp_packets = muxer.mux(frame)?;
            packets.extend(rtp_packets.into_iter().map(|buf| {
                if let RtpBuf::Rtp(d) = &buf {
                    pt = d[1] & 0x7F;
                }
                RtpMemPacket::from_rtp_buf(buf, ch_id, pts, dts)
            }));
        }
        // println!("rtpmem: ch[{ch_id}] rtp packets {}", packets.len());
        packets
    };

    Ok(MemTrack {
        ch_id,
        _payload_type: pt,
        _info: info,
        control,
        packets,
        spspps,
    })
}

// fn load_track2(reader: &mut Reader, index: usize, ch_id: u8, max_frames: u64) -> Result<MemTrack> {

//     // let val = reader
//     // .input
//     // .stream(index).with_context(||"NOT found stream")?
//     // .time_base();
//     // println!("track[{index}]: time_base {val:?}");

//     let mut pt = 0;
//     let info = reader.stream_info(index)?;
//     let control = format!("track_id={index}");

//     // https://stackoverflow.com/questions/65800733/send-sprop-parameter-sets-inband-rather-than-in-sdp
//     //       -bsf:v extract_extradata,dump_extra
//     let opts = Options::new_from_hashmap(&[
//         ("bsf:v".to_string(), "extract_extradata,dump_extra".to_string()),
//         // ("bsf:v".to_string(), "dump_extra".to_string()),
//     ].into());
    
//     let muxer = video_rs::PacketizedBufMuxer;::new_to_packetized_buf_with_options("rtp", opts)?;

//     let mut muxer = muxer.with_stream(info.clone())?;

//     // let mut muxer = RtpMuxer::new()?
//     // .with_stream(info.clone())?;

//     let packets = {
//         let mut packets = Vec::new();
//         let mut pts_gen = TsGen::new(40);
//         let mut dts_gen = TsGen::new(40);

//         for num in 0..max_frames {
//             let frame = match reader.read(index){
//                 Ok(v) => v,
//                 Err(_e) => break,
//             };
//             let pts = pts_gen.convert(frame.pts().into_parts());
//             let dts = dts_gen.convert(frame.pts().into_parts());

//             // let pts = Duration::from(frame.pts()).as_millis() as i64;
//             // let dts = Duration::from(frame.dts()).as_millis() as i64;
//             println!("ch[{ch_id}] frame {num}: pts {}, dts {}", pts, dts);

//             let rtp_packets: Vec<RtpBuf> = muxer
//             .mux(frame)
//             .map(|bufs| bufs.into_iter().map(|buf| buf.into()).collect())?;

//             // let rtp_packets = muxer.mux(frame)?;

//             packets.extend(rtp_packets.into_iter().map(|buf| {
//                 if let RtpBuf::Rtp(d) = &buf {
//                     pt = d[1] & 0x7F;
//                 }
//                 RtpMemPacket::from_rtp_buf(buf, ch_id, pts, dts)
//             }));
//         }
//         println!("ch[{ch_id}] rtp packets {}", packets.len());
//         packets
//     };

//     Ok(MemTrack {
//         ch_id,
//         _payload_type: pt,
//         _info: info,
//         control,
//         packets,
//     })
// }


struct TsGen {
    last: i64,
    fake_interval: i64,
}

impl TsGen {
    pub fn new(fake_interval: i64) -> Self {
        Self {
            fake_interval,
            last: 0,
        }
    }

    pub fn convert(&mut self, ts: (Option<i64>, ffmpeg_next::Rational)) -> i64 {
        match ts.0 {
            Some(value) => {
                self.last  = 1000 * value 
                    * ts.1.numerator() as i64 / ts.1.denominator() as i64;
                self.last
            },
            None => {
                self.next_fake()
            },
        }
    }

    fn next_fake(&mut self) -> i64 {
        self.last += self.fake_interval;
        self.last
    }
}



fn best_media_index(reader: &Reader, kind: ffmpeg_next::media::Type) -> Option<usize> {
    reader
        .input
        .streams()
        .best(kind)
        .map(|x|x.index())
}

type VecBytes = Vec<u8>;
type SpsBytes = VecBytes;
type PpsBytes = VecBytes;


pub struct MemTrack {
    ch_id: u64,
    _payload_type: u8,
    _info: StreamInfo,
    control: String,
    packets: Vec<RtpMemPacket>,
    spspps: Vec<(SpsBytes, Vec<PpsBytes>)>,
}

impl MemTrack {
    pub fn ch_track_index(&self) -> usize {
        (self.ch_id >> 1) as usize
    }

    pub fn spspps(&self) -> &Vec<(SpsBytes, Vec<PpsBytes>)> {
        &self.spspps
    }
}



// fn make_sdp(name: &str, video: &Option<MemTrack>, audio: &Option<MemTrack>) -> Result<Sdp> {

//     const ORIGIN_DUMMY_HOST: [u8; 4] = [0, 0, 0, 0];
//     const TARGET_DUMMY_HOST: [u8; 4] = [0, 0, 0, 0];
//     const TARGET_DUMMY_PORT: u16 = 0;

//     // const FMT_RTP_PAYLOAD_DYNAMIC: usize = 96;

//     // let mut format = FMT_RTP_PAYLOAD_DYNAMIC;

//     let mut sdp = Sdp::new(
//         ORIGIN_DUMMY_HOST.into(),
//         name.to_string(),
//         TARGET_DUMMY_HOST.into(),
//         TimeRange::Live,
//     );

//     if let Some(track) = video {

//         let muxer = RtpMuxer::new()?
//         .with_stream(track.info.clone())?;

//         let (sps, pps) = muxer
//         .parameter_sets_h264()
//         .into_iter()
//         // The `parameter_sets` function will return an error if the
//         // underlying stream codec is not supported, we filter out
//         // the stream in that case, and return `CodecNotSupported`.
//         .filter_map(Result::ok)
//         .next()
//         .with_context(||"video is NOT H264")?;

//         let codec_info = CodecInfo::h264(sps, pps.as_slice(), muxer.packetization_mode());

//         sdp = sdp.with_media(
//             Kind::Video,
//             TARGET_DUMMY_PORT,
//             Protocol::RtpAvp,
//             codec_info,
//             Direction::ReceiveOnly,
//         );

//         if let Some(last) = sdp.media.last_mut() {
//             last.tags.push(oddity_sdp_protocol::Tag::Property(format!("control:{}", track.control)));
//             last.format = track.payload_type as usize;
//             // format +=1;
//         }
//     }

//     if let Some(track) = audio {
//         sdp.media.push(oddity_sdp_protocol::Media {
//             kind: Kind::Audio,
//             port: TARGET_DUMMY_PORT,
//             protocol: Protocol::RtpAvp,
//             format: track.payload_type as usize,
//             tags: vec![
//                 oddity_sdp_protocol::Tag::Value(
//                     "rtpmap".to_string(),
//                     format!("{} {}",  track.payload_type, "MPEG4-GENERIC/48000/2"),
//                 ),
//                 oddity_sdp_protocol::Tag::Value(
//                     "fmtp".to_string(),
//                     format!(
//                         "{} {}", 
//                         track.payload_type,
//                         "profile-level-id=1;mode=AAC-hbr;sizelength=13;indexlength=3;indexdeltalength=3; config=119056E500",
//                     ),
//                 ),
//                 oddity_sdp_protocol::Tag::Property(Direction::ReceiveOnly.to_string()),
//                 oddity_sdp_protocol::Tag::Property(format!("control:{}", track.control)),
//                 // oddity_sdp_protocol::Tag::Property("control:streamid=1".into()),
//                 // oddity_sdp_protocol::Tag::Property("control:1234".into()),
//             ],
//         });
//         // format += 1;
//     }

//     Ok(sdp)
// }


// #[tokio::test]
// async fn test_poc() -> Result<()> {
//     let file_url = "/tmp/sample-data/sample.mp4";
//     // let file_url = "/tmp/output.h264";
//     let mut reader = load_rtp_mem(file_url, 16).await.unwrap().make_reader();

//     let mut num = 0_u64;
//     while let Some(mem) = reader.read_next() {
//         num += 1;
//         let milli = mem.dts;
//         let ch_id = mem.ch_id;
//         let data_len = mem.data.len();
//         println!("rtp {num}: milli {milli}, ch_id {ch_id}, len {data_len}");
//     }

//     Ok(())
// }

// #[test]
// fn test_ffmpeg_input() {
//     let file_url = "/tmp/sample-data/sample.mp4";
//     let max_frames = 16_u64;

//     let mut input = ffmpeg_next::format::input_with_dictionary(&file_url, Default::default()).unwrap();

//     // input
//     //     .seek(i64::min_value(), ..)
//     //     .unwrap();

//     for num in 0..max_frames {
//         let (stream, packet) = input.packets().next().unwrap();
//         println!("Packet No.{num}: track {}, pts {:?}, dts {:?}", stream.index(), packet.pts(), packet.dts());
//     }
// }

