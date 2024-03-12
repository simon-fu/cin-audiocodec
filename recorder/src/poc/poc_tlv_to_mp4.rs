use std::path::Path;
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use crate::{ffeasy::output::{FFOutput, FFTrack, FFWriter}, media::CodecId, rtp::{  codec::{aac::RtpDepackerAAC, h264::{RtpDepackerH264, RtpH264Parameters}}, depack::RtpCodecDepacker}, sdp::sdp::SdpCodec, tlv_custom::{parse_tlv_file, ChPacket, ContextMut, FlowIndex, FlowMut, Handler, StreamIndex, StreamInfo, TrackIndex}};
use ffmpeg_next as ff;

#[test]
fn test_probe() {
    let ipath = "/tmp/output.tlv2";
    let info = parse_tlv_file(ipath.as_ref(), &mut ()).unwrap();
    println!("{info:#?}");
}


#[test]
fn test_tlv_file_to_mp4() {
    let ipath = "/tmp/output.tlv2";
    let odir = "/tmp";
    let oname_prefix = "ostream_";

    tlv_file_to_mp4(ipath.as_ref(), odir.as_ref(), oname_prefix.as_ref());
}

pub fn tlv_file_to_mp4(ipath: &Path, odir: &Path, oname_prefix: &str)  {
    let file_info = parse_tlv_file(ipath, &mut ()).unwrap();

    let mut conver = Converter {
        max_packets: Some(64),
        ..Default::default()
    } ;

    let mut output_paths = Vec::new();

    for stream in file_info.streams.iter() {
        
        let opath = odir.join(format!("{oname_prefix}{}.mp4", stream.index));
        
        let mut output = FFOutput::open(&opath.as_path()).unwrap();
        println!("opened output [{opath:?}]");
        output_paths.push(opath);

        let mut depackers: Vec<Option<Box<dyn RtpCodecDepacker>>> = Vec::new();
        let mut ctracks = Vec::new();

        for track in stream.tracks.iter() {

            let mut ctrack = CTrack {
                flows: Default::default(),
            };

            for flow in track.flows.iter() {
                let flow = match flow.codec.codec_id {
                    CodecId::H264 => {
                        let fmtp: &str = flow.codec.fmtps
                        .get(0)
                        .map(|x|x.as_str()).unwrap_or("");

                        let param = RtpH264Parameters::parse_from_str(fmtp).map_err(|e|anyhow!("{e}")).unwrap();

                        let mut spspps = BytesMut::new();
                        if param.sps_nal.len() > 0 {
                            spspps.put(&[0,0,0,1][..]);
                            spspps.put(&param.sps_nal[..]);
                        }

                        if param.pps_nal.len() > 0 {
                            spspps.put(&[0,0,0,1][..]);
                            spspps.put(&param.pps_nal[..]);
                        }

                        let otrack_index = output.add_h264_track(
                            param.generic.pixel_dimensions.0 as i32, 
                            param.generic.pixel_dimensions.1 as i32, 
                            &spspps[..],
                        ).unwrap().index();
                        println!("add h264 track, spspps {}", spspps.len());

                        
                        depackers.push(Some( RtpDepackerH264::new(Some(fmtp)).unwrap().into_box() ));
                        
                        
                        CFlow {
                            otrack_index: Some(otrack_index),
                        }
                    },
                    CodecId::AAC => {
                        let otrack_index = output.add_aac_track(
                            flow.codec.clock_rate as i32, 
                            flow.codec.channels.unwrap_or(1) as i32,
                        ).unwrap().index();
                        
                        let fmtp = flow.codec.fmtps
                        .get(0)
                        .map(|x|x.as_str());

                        println!("add aac track");
                        
                        depackers.push(Some(RtpDepackerAAC::new (
                            flow.codec.clock_rate,
                            flow.codec.channels.map(|x| std::num::NonZeroU16::new(x as u16)).unwrap_or(None),
                            fmtp,
                        ).unwrap().into_box() ));
                        

                        CFlow {
                            otrack_index: Some(otrack_index),
                        }
                    },
                    _ => CFlow {
                        otrack_index: None,
                        // depacker: Some(Box::new(RtpDepackerH264::default())),
                    },
                };

                ctrack.flows.push(flow);
            }
            ctracks.push(ctrack);
        }

        
        let stream_index = conver.streams.len();
        let writer = output.begin_write().unwrap();
        for (index, item) in writer.tracks_iter().enumerate() {
            if let Some(depacker) = depackers[index].take() {
                let otrack = OTrack {
                    name: format!("stream_{stream_index}_track_{index}_item_{}", item.index),
                    track: item,
                    depacker,
                    wrote_packets: 0,
                };

                println!("add otrack [{}]", otrack.name);

                conver.otracks.push(otrack);
            }
        }

        conver.streams.push(CStream {
            writer,
            tracks: ctracks,
            first_ts: None,
        });
    }

    parse_tlv_file(ipath, &mut conver).unwrap();

    for (index, path) in output_paths.iter().enumerate() {
        println!("output[{index}]=[{path:?}]");
    }

    println!("total wrote packets {}", conver.num_packets);

    // Ok(())
}

#[derive(Default)]
struct Converter {
    streams: Vec<CStream>,
    otracks: Vec<OTrack>,
    num_packets: u64,
    max_packets: Option<u64>,
}

impl Converter {
    fn get_flow_mut(&mut self, index: &FlowIndex) -> Option<&mut CFlow> {
        if let Some(stream) = self.streams.get_mut(index.track.stream) {
            if let Some(track) = stream.tracks.get_mut(index.track.track) {
                if let Some(flow) = track.flows.get_mut(index.flow) {
                    return Some(flow)
                }
            }
        }
        None
    }
}

impl Handler for Converter {
    type Flow = FlowExt;
    
    fn on_add_stream(&mut self, _ctx: ContextMut<'_, Self>, _index: StreamIndex, _ts: i64, _info: &StreamInfo) -> Result<()>  {
        Ok(())
    }
    
    fn on_add_track(&mut self, _ctx: ContextMut<'_, Self>, _index: TrackIndex) -> Result<()>  {
        Ok(())
    }
    
    fn on_add_flow(&mut self, _ctx: ContextMut<'_, Self>, index: FlowIndex, _codec: &SdpCodec) -> Result<Self::Flow> {
        let otrack_index = self.get_flow_mut(&index).map(|x|x.otrack_index.clone()).unwrap_or(None);

        Ok(FlowExt {
            otrack_index,
        })
    }
    
    fn on_flow_rtp(&mut self, mut ctx: ContextMut<'_, Self>, flow: &mut FlowMut<Self::Flow>, packet: &ChPacket) -> Result<()> {
        let ext = flow.ext_mut();
        if let Some(otrack_index) = ext.otrack_index {
            // println!("on_flow_rtp: track {index}");

            let stream_index = flow.index().track.stream;

            let r1 = self.otracks.get_mut(otrack_index);
            let r2 = self.streams.get_mut(stream_index);
            if let (Some(otrack), Some(stream)) = (r1, r2) {
                println!("on_flow_rtp: otrack.name [{}]", otrack.name);

                otrack.depacker.push_rtp_slice(&packet.data).unwrap();
                while let Some(frame) = otrack.depacker.pull_frame().unwrap() {
                    let mut ffpacket = ff::Packet::copy(&frame[..]);
                    
                    // let src_time_base = ff::Rational::new(1, 30);
                    // let pts = otrack.wrote_packets as i64;
                    
                    let pts = match stream.first_ts {
                        Some(first) => packet.ts - first,
                        None => {
                            stream.first_ts = Some(packet.ts);
                            0
                        },
                    };

                    let src_time_base = ff::Rational::new(1, 1000);

                    println!("wrote pakcet, stream {stream_index}, track {otrack_index}, pts {pts}, {}", pretty_hex::simple_hex(&&frame[..4]));

                    ffpacket.set_pts(Some(pts));
                    // ffpacket.set_dts(Some(packet.ts));
                    
                    stream.writer.write_packet(&otrack.track, src_time_base, &mut ffpacket).unwrap();
                    otrack.wrote_packets += 1;
                }
            }
        }

        self.num_packets += 1;
        let max_packets = self.max_packets.unwrap_or(u64::MAX);
        // let max_packets = 16;
        if self.num_packets >= max_packets {
            ctx.set_finished();
        }

        Ok(())
    }
    
    fn on_track_rtcp(&mut self, _ctx: ContextMut<'_, Self>, _index: TrackIndex, _packet: &ChPacket) -> Result<()> {
        Ok(())
    }

    
}



struct OTrack {
    name: String,
    track: FFTrack,
    depacker: Box<dyn RtpCodecDepacker>,
    wrote_packets: u64,
}



struct CStream {
    writer: FFWriter,
    tracks: Vec<CTrack>,
    first_ts: Option<i64>,
}

struct CTrack {
    flows: Vec<CFlow>,
}

struct CFlow {
    otrack_index: Option<usize>,
    // depacker: Option<Box<dyn RtpCodecDepacker>>,
}

struct FlowExt {
    otrack_index: Option<usize>,
}



