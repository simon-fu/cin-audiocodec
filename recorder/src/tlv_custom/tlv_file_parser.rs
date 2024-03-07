use std::{collections::HashMap, fmt, marker::PhantomData, path::Path};
use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use rtp_rs::RtpReader;
use crate::{rtp::rtp::check_is_rtcp, sdp::sdp::{SdpCodec, SdpMain, SdpMedia}, tlv2::{TlvFileSyncReader, Type, VecBuf}, tlv_custom::{TlvType, TLV_MAGIC}};

macro_rules! dbgd {
    ($($arg:tt)* ) => (
        // println!($($arg)*)
    );
}

#[test]
fn test_parse() {
    struct DumpHandler {
        max_ch_packets: u64,
        num_ch_packets: u64,
    }

    impl Handler for DumpHandler {
        // type Context = ();
        // type Stream = ();
        // type Track = ();
        type Flow = ();

        fn on_add_stream(&mut self, _ctx: ContextMut<'_, Self>, index: StreamIndex, ts: i64, info: &StreamInfo) -> Result<()>  {
            println!("on_add_stream: {index:?}, {ts}, {info:?}");
            Ok(())
        }
    
        fn on_add_track(&mut self, _ctx: ContextMut<'_, Self>, index: TrackIndex) -> Result<()> {
            println!("on_add_track: {index:?}");
            Ok(())
        }

        fn on_add_flow(&mut self, _ctx: ContextMut<'_, Self>, index: FlowIndex, codec: &SdpCodec) -> Result<Self::Flow> {
            println!("on_add_flow: {index:?}, {codec:?}");
            Ok(())
        }
    
        fn on_flow_rtp(&mut self, mut ctx: ContextMut<'_, Self>, flow: &mut FlowMut<Self::Flow>, packet: &ChPacket) -> Result<()> {
            println!("on_flow_rtp: {:?}, codec {:?}, {packet}", flow.index(),  flow.codec().codec_id);
            self.num_ch_packets += 1;
            if self.num_ch_packets >= self.max_ch_packets {
                ctx.set_finished();
            }
            Ok(())
        }

        fn on_track_rtcp(&mut self, _ctx: ContextMut<'_, Self>, index: TrackIndex, packet: &ChPacket) -> Result<()> {
            println!("on_track_rtcp: {index:?}, {packet}");
            Ok(())
        }
        
    }

    let ipath = "/tmp/output.tlv2";
    let mut handler = DumpHandler {
        max_ch_packets: 16,
        num_ch_packets: 0,
    };
    parse_tlv_file(ipath.as_ref(), &mut handler).unwrap();
}









pub struct ContextMut<'a, H: Handler>(&'a mut ParserContext<H>);

impl<'a, H: Handler> ContextMut<'a, H> {
    pub fn set_finished(&mut self) {
        self.0.finished = true;
    }

    // pub fn ext_mut(&mut self) -> &mut H::Context {
    //     &mut self.0.ext
    // }
}

// pub struct StreamMut<'a, H: Hanlder>(&'a mut ParserStream<H>, &'a mut ParserMut<'a, H>);

// impl<'a, H: Hanlder> StreamMut<'a, H> {
//     pub fn ctx_mut(&'a mut self) -> &'a mut ParserMut<'a, H> {
//         &mut self.1
//     }

//     pub fn ext_mut(&mut self) -> &mut H::Stream {
//         &mut self.0.ext
//     }
// }

// pub struct TrackMut<'a, H: Hanlder> {
//     info: &'a TrackInfo, 
//     stream: StreamMut<'a, H>,
//     ext: &'a mut H::Track,
// }

// impl<'a, H: Hanlder> TrackMut<'a, H> {

//     pub fn stream_mut(&'a mut self) -> &'a mut StreamMut<'a, H> {
//         &mut self.stream
//     }

//     pub fn ext_mut(&mut self) -> &mut H::Track {
//         &mut self.ext
//     }
// }


struct ParserContext<H: Handler> {
    finished: bool,
    _none: PhantomData<H>,
    // ext: H::Context,
}

// impl<H: Hanlder> ParserContext<H> {
//     pub fn set_finished(&mut self) {
//         self.finished = true;
//     }
// }



#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlowIndex {
    pub track: TrackIndex,
    pub flow: usize,
}

impl fmt::Debug for FlowIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlowIndex")
        .field("stream", &self.track.stream)
        .field("track", &self.track.track)
        .field("flow", &self.flow)
        .finish()
    }
}

pub struct FlowMut<'a, T>(&'a mut ParserFlow<T>);

impl<'a, T> FlowMut<'a, T> {
    pub fn codec(&self) -> &SdpCodec {
        &self.0.codec
    }

    pub fn index(&self) -> &FlowIndex {
        &self.0.index
    }

    pub fn ext_mut(&mut self) -> &mut T {
        &mut self.0.ext
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamInfo {
    pub name: String,
    pub ch_id: u64,
    pub sdp: String,
}

#[derive(Clone)]
pub struct ChPacket {
    pub ts: i64,
    pub ch_id: u64,
    pub data: Bytes,
}

impl fmt::Display for ChPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChPacket")
        .field("ts", &self.ts)
        .field("ch_id", &self.ch_id)
        .field("data", &self.data.len())
        .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamIndex {
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackIndex {
    pub stream: usize,
    pub track: usize,
}

pub trait Handler: Sized {
    // type Context;
    // type Stream;
    // type Track;
    type Flow;

    fn on_add_stream(&mut self, ctx: ContextMut<'_, Self>, index: StreamIndex, ts: i64, info: &StreamInfo) -> Result<()> ;
    
    fn on_add_track(&mut self, ctx: ContextMut<'_, Self>, index: TrackIndex) -> Result<()> ;

    fn on_add_flow(&mut self, ctx: ContextMut<'_, Self>, index: FlowIndex, codec: &SdpCodec) -> Result<Self::Flow>;

    fn on_flow_rtp(&mut self, ctx: ContextMut<'_, Self>, flow: &mut FlowMut<Self::Flow>, packet: &ChPacket) -> Result<()>;

    fn on_track_rtcp(&mut self, ctx: ContextMut<'_, Self>, index: TrackIndex, packet: &ChPacket) -> Result<()>;
}

impl Handler for () {
    type Flow = ();
    
    fn on_add_stream(&mut self, _ctx: ContextMut<'_, Self>, _index: StreamIndex, _ts: i64, _info: &StreamInfo) -> Result<()>  {
        Ok(())
    }
    
    fn on_add_track(&mut self, _ctx: ContextMut<'_, Self>, _index: TrackIndex) -> Result<()>  {
        Ok(())
    }
    
    fn on_add_flow(&mut self, _ctx: ContextMut<'_, Self>, _index: FlowIndex, _codec: &SdpCodec) -> Result<Self::Flow> {
        Ok(())
    }
    
    fn on_flow_rtp(&mut self, _ctx: ContextMut<'_, Self>, _flow: &mut FlowMut<Self::Flow>, _packet: &ChPacket) -> Result<()> {
        Ok(())
    }
    
    fn on_track_rtcp(&mut self, _ctx: ContextMut<'_, Self>, _index: TrackIndex, _packet: &ChPacket) -> Result<()> {
        Ok(())
    }

    
}


#[derive(Debug, Clone)]
pub struct Stream {
    pub index: usize,
    pub info: StreamInfo,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub flows: Vec<Flow>,
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub codec: SdpCodec,
}

// pub struct Parser<H: Hanlder> {
//     streams: MainContext<H>,
//     context: ParserContext<H>,
//     handler: H,
//     reader: TlvFileSyncReader,
// }

// impl <H: Hanlder> Parser<H> {
//     pub fn open(ipath: &Path, handler: H) -> Result<Self> {

//     }
// }

pub fn parse_tlv_file<H: Handler>(ipath: &Path, handler: &mut H) -> Result<FileInfo> 
{
    let mut reader = TlvFileSyncReader::open_with_magic(&ipath, Some(TLV_MAGIC))
    .with_context(||format!("failed open [{ipath:?}]"))?;
    dbgd!("opened input {ipath:?}");


    let mut buf = VecBuf::default();
    let buf  = &mut buf;

    let ctx = ParserContext {
        finished: false,
        _none: Default::default(),
        // ext: ctx,
    };

    // let mut parser = ParserMut(&mut ctx);

    let mut handler = HandlerMut {
        ctx,
        handler,
    };

    let mut parser = MainContext {
        streams: Vec::new(),
        stream_indexes: HashMap::new(),
    };

    while !handler.ctx.finished {
        let tag = reader.read_tag(buf)
            .with_context(||"read next tlv failed")?;

        let rtype = tag.rtype();
        if rtype.is_build_in() {
            match rtype {
                Type::ATTACH_END => {}
                Type::FILE_END => {
                    dbgd!("got file end\n");
                    break;
                },
                _ => {}
            }
            continue;
        }

        let r = TlvType::try_from(rtype);
        match r {
            Ok(rtype) => {
                
                match rtype {
                    TlvType::AddCh => {
                        let mut value = tag.value();
                        let ts = value.cut_var_i64()?;
                        let content = value.as_str()?;
                        dbgd!("read: add_ch, ts [{ts}], content [{}]-[{content}]\n", content.len());

                        let info: StreamInfo = serde_json::from_str(content)?;

                        let stream_index =  StreamIndex {
                            index: parser.streams.len(),
                        };

                        handler.handler.on_add_stream(
                            ContextMut(&mut handler.ctx), 
                            stream_index,
                            ts, 
                            &info
                        )?;

                        {
                            let stream = ParserStream::<H>::new(stream_index.index, info, &mut handler)?;
                    
                            for track_index in 0..stream.num_tracks() {
                                let ch_id = stream.start_ch_id() + (track_index << 1) as u64;
                                dbgd!("add track, index {track_index}, ch_id {ch_id}");

                                parser.stream_indexes.insert(ch_id, stream_index);
                                parser.stream_indexes.insert(ch_id + 1, stream_index);
                            }
                            parser.streams.push(stream);
                        }

                    }
                    TlvType::ChData => {
                        let mut value = tag.value();
                        let ts = value.cut_var_i64()?;
                        let ch_id = value.cut_var_u64()?;
                        let data = value.as_slice();

                        let packet = ChPacket {
                            ts,
                            ch_id,
                            data: Bytes::copy_from_slice(data),
                        };

                        dbgd!("read: ch packet, {packet}");

                        if let Some(stream_index) = parser.stream_indexes.get_mut(&packet.ch_id) {
                            if let Some(stream) = parser.streams.get_mut(stream_index.index) {
                                stream.handle_ch_data(packet, &mut handler)?;
                            }
                        }

                        // break;
                    },
                    _ => {
                        dbgd!("read: unhandle {rtype:?}");
                    }
                }
            },
            Err(_e) => {
                dbgd!("unknown tlv type [{rtype:?}]");
            },
        }
    }

    Ok(parser.into())
}

struct HandlerMut<'a, H: Handler> {
    ctx: ParserContext<H>,
    // ctx: &'a mut ParserMut<'a, H>,
    handler: &'a mut H,
}


#[derive(Debug, Clone, Default)]
pub struct FileInfo {
    pub streams: Vec<Stream>,
}

impl<H: Handler> From<MainContext<H>> for FileInfo {
    fn from(from: MainContext<H>) -> Self {
        Self {
            streams: from.streams.into_iter().map(|x|x.into()).collect(),
        }
    }
}


struct MainContext<H: Handler> {
    streams: Vec<ParserStream<H>>,
    stream_indexes: HashMap::<u64, StreamIndex>,
}


impl<H: Handler> From<ParserStream<H>> for Stream {
    fn from(from: ParserStream<H>) -> Self {
        Self {
            index: from.index,
            info: from.info,
            tracks: from.tracks.into_iter().map(|x|x.into()).collect(),
        }
    }
}

struct ParserStream<H: Handler> {
    info: StreamInfo,
    index: usize,
    // sdp: SdpMain,
    tracks: Vec<ParserTrack<H>>,
    // ext: H::Stream,
}

impl<H: Handler> ParserStream<H> {
    pub fn new(stream_index: usize, info: StreamInfo, handler: &mut HandlerMut<'_, H>) -> Result<Self> {

        // let sdp = sdp_rs::SessionDescription::from_str(&info.sdp)?; 
        let sdp = SdpMain::parse_from_str(&info.sdp)?;
        // let mut tracks = Vec::with_capacity(sdp.medias.len());

        let mut me = Self {
            index: stream_index,
            tracks: Vec::with_capacity(sdp.medias.len()),
            // ext: ch_ext,
            info,
            // sdp,
        };
        
        for (mindex, sdp_media) in sdp.medias.into_iter().enumerate() {
            let index = TrackIndex {
                stream: stream_index,
                track: mindex,
            };

            handler.handler.on_add_track(ContextMut(&mut handler.ctx), index)?;

            me.tracks.push(ParserTrack {
                index,
                flows: Default::default(),
                // ext: track_ext,
                info : TrackInfo {
                    // ch_id: (mindex << 1) as u64,
                    sdp_media,
                }
            });
        }
        dbgd!("add tracks {}", me.tracks.len());
        Ok(me)
    }

    pub fn start_ch_id(&self) -> u64 {
        self.info.ch_id
    }

    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }

    pub fn handle_ch_data(&mut self, ch_data: ChPacket, handler: &mut HandlerMut<'_, H>) -> Result<()> 
    {
        
        // if ch_data.ch_id % 2 != 0 {
        //     // TODO: 应该通过解析数据包来判断是否rtcp数据
        //     // ignore rtcp
        //     return Ok(())
        // }
        
        let track_index = (ch_data.ch_id >> 1) as usize;
        
        if let Some(track) = self.tracks.get_mut(track_index) {
            if check_is_rtcp(&ch_data.data) {
                handler.handler.on_track_rtcp(
                    ContextMut(&mut handler.ctx), 
                    track.index,
                    &ch_data,
                )?;
            }
            
            let rtp = RtpReader::new(&ch_data.data).map_err(|e|anyhow!("invalid rtp {e:?}"))?;

            dbgd!("handle_ch_data: track_index {track_index}, rtp {rtp:?}");

            let found = track.flows.get_mut(&rtp.payload_type());

            match found {
                Some(flow) => {
                    handler.handler.on_flow_rtp(ContextMut(&mut handler.ctx), &mut FlowMut(flow), &ch_data)?;
                },
                None => {
                    match &track.info.sdp_media {
                        SdpMedia::Video(mdesc) => {
                            if let Some(codec) = mdesc.codecs.get(&rtp.payload_type()) {
                                let flow_index = FlowIndex { 
                                    track: track.index, 
                                    flow: track.flows.len(), 
                                };

                                let ext = handler.handler.on_add_flow(
                                    ContextMut(&mut handler.ctx),
                                    flow_index,
                                    codec
                                )?;

                                let mut flow = ParserFlow {
                                    index: flow_index,
                                    codec: codec.clone(),
                                    ext,
                                };

                                handler.handler.on_flow_rtp(ContextMut(&mut handler.ctx), &mut FlowMut(&mut flow), &ch_data)?;

                                track.flows.insert(rtp.payload_type(), flow);
                            }
                        },
                        SdpMedia::Audio(mdesc) => {
                            if let Some(codec) = mdesc.codecs.get(&rtp.payload_type()) {
                                let flow_index = FlowIndex { 
                                    track: track.index, 
                                    flow: track.flows.len(), 
                                };

                                let ext = handler.handler.on_add_flow(
                                    ContextMut(&mut handler.ctx),
                                    flow_index,
                                    codec
                                )?;

                                let mut flow = ParserFlow {
                                    index: flow_index,
                                    codec: codec.clone(),
                                    ext,
                                };

                                handler.handler.on_flow_rtp(ContextMut(&mut handler.ctx), &mut FlowMut(&mut flow), &ch_data)?;

                                track.flows.insert(rtp.payload_type(), flow);
                            }
                        },
                        SdpMedia::Unknown => {},
                    }
                },
            }
        }
        Ok(())
    }
}

struct ParserTrack<H: Handler> {
    index: TrackIndex,
    info: TrackInfo,
    flows: HashMap<u8, ParserFlow<H::Flow>>, 
    // ext: H::Track,
}

struct TrackInfo {
    // ch_id: u64,
    sdp_media: SdpMedia,
}

impl <H: Handler> From<ParserTrack<H>> for Track {
    fn from(from: ParserTrack<H>) -> Self {
        Self {
            flows: from.flows
                .into_iter()
                .map(|(_k, v)| Flow::from(v))
                .collect(),
        }
    }
}


// impl<T> ParserTrack<T> {
//     pub fn has_got_all_flows(&self) -> bool {
//         let max_flows = if self.sdp_media.is_audio_or_video() {1} else {0};
//         self.flows.len() >= max_flows
//     }
// }

struct ParserFlow<T> {
    index: FlowIndex,
    codec: SdpCodec,
    ext: T,
}

impl <T> From<ParserFlow<T>> for Flow {
    fn from(from: ParserFlow<T>) -> Self {
        Self {
            codec: from.codec,
        }
    }
}



