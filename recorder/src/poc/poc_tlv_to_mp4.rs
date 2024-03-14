use std::{collections::HashMap, path::Path};
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use ff::{ChannelLayout, Rescale};
use crate::{ffeasy::{audio::{audio_packed_i16_format, audio_planar_f32_format, input::{audio_decoder_receive_frame, make_audio_decoder}, swr::SResampler}, encoder::{FFAudioEncoder, FFVideoEncoder}, ffi::{audio_frame_packed_i16_samples, audio_frame_packed_i16_samples_mut}, output::{FFOutput, FFTrack, FFWriter}, video::{image::FFYuvImage, make_video_decoder, video_decoder_receive_frame, FFVideoArgs, VideoSize}}, media::CodecId, mix_audio::mixer::{AChId, PcmMixer, PcmTimedMixer}, mix_video::{mixer::VideoMixer, VChId}, rtp::{  codec::{aac::RtpDepackerAAC, h264::{RtpDepackerH264, RtpH264Parameters}}, depack::RtpCodecDepacker}, sdp::sdp::{SdpCodec, SdpMediaType}, tlv_custom::{parse_tlv_file, ChPacket, ContextMut, Flow, FlowIndex, FlowMut, Handler, StreamIndex, StreamInfo, TrackIndex}};
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
    let max_packets: Option<u64> = Some(8000);
    tlv_file_to_mp4(ipath.as_ref(), odir.as_ref(), oname_prefix.as_ref(), max_packets);
}

pub fn tlv_file_to_mp4(ipath: &Path, odir: &Path, oname_prefix: &str, max_packets: Option<u64>)  {
    let file_info = parse_tlv_file(ipath, &mut ()).unwrap();

    let mut output_paths = Vec::new();

    let mut mix_output;
    {
        let opath = odir.join(format!("{oname_prefix}mix.mp4"));
        mix_output = FFOutput::open(&opath).unwrap();
        println!("opened output [{opath:?}]");
        output_paths.push(opath);
    }


    let mix_video_args = FFVideoArgs {
        codec_id: ff::codec::Id::H264,
        width: 1280,
        height: 720,
        extra: Default::default(),
        time_base: milli_time_base(),
    };


    let video_encoder = FFVideoEncoder::h264(mix_video_args.width as u32, mix_video_args.height as u32, 25, FFYuvImage::FORMAT, true, true).unwrap();

    let video_track_index;

    {
        let video_params = video_encoder.get_parameters();
        video_track_index = mix_output.add_h264_track(mix_video_args.width, mix_video_args.height, video_params.get_extra()).unwrap().index(); 
    }

    
    // let mixed_audio_format = audio_packed_i16_format();
    let audio_samplerate = 48000_u32;
    let audio_channels = 2_u32;

    let audio_encoder = {

        FFAudioEncoder::aac(audio_samplerate, audio_channels, audio_planar_f32_format(), false).unwrap()
    };

    let audio_track_index;

    {
        audio_track_index = mix_output.add_aac_track(
            audio_samplerate as i32,
            audio_channels as i32
        ).unwrap().index();
    }

    let writer = mix_output.begin_write().unwrap();
    let video_track = writer.get_track(video_track_index).unwrap();  
    let audio_track = writer.get_track(audio_track_index).unwrap();  
    

    let mut conver = Converter {
        max_packets,
        mixer: Some(Mixer {
            video: Some(MixContextVideo {
                mixer: VideoMixer::new(VideoSize {
                    width: mix_video_args.width as u32,
                    height: mix_video_args.height as u32,
                }).unwrap(),
                tracks: Default::default(),
                last_mix_ts: None,
                encoder: video_encoder,
                o_track: video_track,
            }),
            audio: Some(MixContextAudio::new(audio_encoder, audio_track)),
            first_ts: None,
            writer,
        }),
        ..Default::default()
    } ;

    let mut otrack_count = 0_usize;

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

                        depackers.push(Some( RtpDepackerH264::new(Some(fmtp)).unwrap().into_box() ));

                        let args = parse_h264(&flow.codec).unwrap();


                        output.add_h264_track(
                            args.width, 
                            args.height, 
                            &args.extra,
                        ).unwrap().index();
                        println!("add h264 track, spspps {}", args.extra.len());

                        let otrack_index = otrack_count;
                        otrack_count += 1;
                        
                        CFlow {
                            otrack_index: Some(otrack_index),
                            // otrack: None,
                        }
                    },
                    CodecId::AAC => {
                        output.add_aac_track(
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
                        
                        let otrack_index = otrack_count;
                        otrack_count += 1;

                        CFlow {
                            otrack_index: Some(otrack_index),
                            // otrack: None,
                        }
                    },
                    _ => CFlow {
                        otrack_index: None,
                        // otrack: None,
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

                // let flow = find_cflow_mut(&mut ctracks, index).unwrap();
                // flow.otrack = Some(otrack);

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

// fn find_cflow_mut(ctracks: &mut Vec<CTrack>, index: usize) -> Option<&mut CFlow> {
//     for track in ctracks.iter_mut() {
//         for flow in track.flows.iter_mut() {
//             if flow.otrack_index == Some(index) {
//                 return Some(flow)
//             }
//         }
//     }
//     None
// }

#[derive(Default)]
struct Converter {
    streams: Vec<CStream>,
    otracks: Vec<OTrack>,
    mixer: Option<Mixer>,
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

    fn handle_flow_rtp(&mut self, flow: &mut FlowMut<FlowExt>, rtp_packet: &ChPacket) {
        let ext = flow.ext_mut();
        if let Some(otrack_index) = ext.otrack_index {
            // println!("on_flow_rtp: track {index}");

            let stream_index = flow.index().track.stream;

            let r1 = self.otracks.get_mut(otrack_index);
            let r2 = self.streams.get_mut(stream_index);
            if let (Some(otrack), Some(stream)) = (r1, r2) {
                // println!("on_flow_rtp: otrack.name [{}]", otrack.name);

                otrack.depacker.push_rtp_slice(&rtp_packet.data).unwrap();
                while let Some(frame) = otrack.depacker.pull_frame().unwrap() {
                    let mut ffpacket = ff::Packet::copy(&frame[..]);
                    
                    // let src_time_base = ff::Rational::new(1, 30);
                    // let pts = otrack.wrote_packets as i64;
                    
                    let pts = match stream.first_ts {
                        Some(first) => rtp_packet.ts - first,
                        None => {
                            stream.first_ts = Some(rtp_packet.ts);
                            0
                        },
                    };

                    let src_time_base = milli_time_base();

                    println!("wrote pakcet, stream {stream_index}, track {otrack_index}, pts {pts}, {}", pretty_hex::simple_hex(&&frame[..4]));

                    ffpacket.set_pts(Some(pts));
                    // ffpacket.set_dts(Some(packet.ts));
                    
                    stream.writer.write_packet(&otrack.track, src_time_base, &mut ffpacket).unwrap();
                    otrack.wrote_packets += 1;

                    if let Some(mixer) = &mut self.mixer {
                        mixer.handle_flow_depacked(flow.flow(), &ffpacket, rtp_packet.ts)
                    }
                }
            }
        }


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
    
    fn on_flow_rtp(&mut self, mut ctx: ContextMut<'_, Self>, flow: &mut FlowMut<Self::Flow>, rtp_packet: &ChPacket) -> Result<()> {
        
        self.handle_flow_rtp(flow, rtp_packet);

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

fn milli_time_base() -> ff::Rational {
    ff::Rational::new(1, 1000)
}

struct Mixer {
    video: Option<MixContextVideo>,
    audio: Option<MixContextAudio>,

    first_ts: Option<i64>,
    writer: FFWriter,
}

impl Mixer {
    pub fn handle_flow_depacked(&mut self, flow: &Flow, packet: &ff::Packet, ts: i64) {

        let ts = match self.first_ts {
            Some(first) => ts - first,
            None => {
                self.first_ts = Some(ts);
                0
            },
        };

        match flow.codec.media_type {
            SdpMediaType::Video => {
                if let Some(video) = &mut self.video {
                    video.handle_flow_depacked(flow, packet, ts);
                }
            }
            SdpMediaType::Audio => {
                if let Some(audio) = &mut self.audio {
                    audio.handle_flow_depacked(flow, packet, ts);
                }
            }
            _ => {}
        }

        if let Some(audio) = &mut self.audio {
            audio.try_mix(ts, &mut self.writer);
        }

        if let Some(video) = &mut self.video {
            video.try_mix(ts, &mut self.writer);
        }

    }
}

struct MixContextVideo {
    mixer: VideoMixer,
    tracks: HashMap<FlowIndex, MixVideoTrack>,
    last_mix_ts: Option<i64>,
    encoder: FFVideoEncoder,
    o_track: FFTrack,
}

impl MixContextVideo {
    pub fn handle_flow_depacked(&mut self, flow: &Flow, packet: &ff::Packet, ts: i64) -> bool {
        if self.try_handle_video(flow, packet) {
            true
        } else {
            match flow.codec.codec_id {
                CodecId::H264 => {
                    self.add_h264_flow(flow);
                    self.handle_flow_depacked(flow, packet, ts)
                },
                _ => false,
            }
        }
        // self.try_mix_video(ts);
    }

    pub fn try_mix(&mut self, ts: i64, writer: &mut FFWriter) {
        // 1000/25=40


        let elapsed = match self.last_mix_ts {
            Some(last) => ts - last,
            None => {
                40
            },
        };

        if elapsed >= 40 {
            self.last_mix_ts = Some(ts);

            let mut image = self.mixer.get_output().unwrap().clone();

            let ts = ts.rescale(milli_time_base(), self.encoder.get_time_base());
            image.frame_mut().set_pts(Some(ts));

            println!("mix video: decode pts {:?}", image.frame().pts());
            self.encoder.send_frame(image.frame()).unwrap();

            while let Some(mut packet) = self.encoder.receive_packet().unwrap() {
                println!("mix video: wrote pts {:?}, time_base {:?}", packet.pts(), self.encoder.get_time_base());
                writer.write_packet(&self.o_track, self.encoder.get_time_base(), &mut packet).unwrap();
            }
        }
    }

    fn try_handle_video(&mut self, flow: &Flow, packet: &ff::Packet) -> bool {
        if let Some(track) = self.tracks.get_mut(&flow.index) {
            // ff::util::frame::Video::new(format, width, height)
            // decoder.time_base();
            track.decoder.send_packet(packet).unwrap();
            while let Some(frame) = video_decoder_receive_frame(&mut track.decoder).unwrap() {
                self.mixer.update_ch(&track.id, frame.into()).unwrap();
            }
            true
        } else {
            false
        }
    }

    fn add_h264_flow(&mut self, flow: &Flow) {
        let args = parse_h264(&flow.codec).unwrap();
        let id = self.mixer.add_ch().unwrap();
        self.tracks.insert(flow.index, MixVideoTrack {
            decoder: make_video_decoder(
                ff::codec::Id::H264, 
                args.width, 
                args.height, 
                &args.extra,
                milli_time_base(),
            ).unwrap(),
            id,
        });
    }    
}



struct MixVideoTrack {
    decoder: ff::codec::decoder::Video,
    id: VChId,
}

struct MixContextAudio {
    mixer: PcmMixer,
    timed: PcmTimedMixer,
    tracks: HashMap<FlowIndex, MixAudioTrack>,
    last_mix_ts: Option<i64>,
    encoder: FFAudioEncoder,
    o_track: FFTrack,
    mixed_frame: ff::frame::Audio,
    frame_size: usize,
    mixed_resampler: SResampler,
}

impl MixContextAudio {
    pub fn new(encoder: FFAudioEncoder, o_track: FFTrack) -> Self {
        let enc_params = encoder.get_parameters();
        // let format = params.get_format_audio();
        let samplerate = enc_params.get_samplerate() as u32;
        let channels = enc_params.get_channels() as u32;
        let frame_size = enc_params.get_frame_size();
        let mixed_format = audio_packed_i16_format();

        let ch_layout = ChannelLayout::default(channels as i32);
        let max_len = (samplerate * channels) as usize;
        
        let mut mixed_frame = ff::frame::Audio::new(mixed_format, frame_size as usize, ch_layout);
        mixed_frame.set_rate(samplerate);

        let mixed_resampler = SResampler::get(
            mixed_format, 
            ch_layout, 
            samplerate, 
            enc_params.get_format_audio().into(), 
            ch_layout, 
            samplerate
        ).unwrap();

        Self {
            mixer: PcmMixer::new(max_len).unwrap(),
            timed: PcmTimedMixer::new(samplerate, channels).unwrap(),
            tracks: Default::default(),
            last_mix_ts: Default::default(),
            frame_size: frame_size as usize,
            mixed_resampler,
            encoder,
            o_track,
            mixed_frame,
        }
    }

    pub fn handle_flow_depacked(&mut self, flow: &Flow, packet: &ff::Packet, ts: i64) -> bool {
        if self.try_handle_audio(flow, packet) {
            true
        } else {
            match flow.codec.codec_id {
                CodecId::AAC => {
                    self.add_aac_flow(flow);
                    self.handle_flow_depacked(flow, packet, ts)
                },
                _ => false,
            }
        }
        // self.try_mix_video(ts);
    }

    fn get_elapsed_since_last_mix(&self, ts: i64) -> i64 {
        match self.last_mix_ts {
            Some(last) => ts - last,
            None => {
                Self::MIX_INTERVAL
            },
        }
    }

    const MIX_INTERVAL: i64 = 20;

    pub fn try_mix(&mut self, ts: i64, writer: &mut FFWriter) {

        
        while self.get_elapsed_since_last_mix(ts) >= Self::MIX_INTERVAL {
            self.mixed_frame.set_samples(self.frame_size);
            let buf = audio_frame_packed_i16_samples_mut(&mut self.mixed_frame);
            
            let mixed_ts = match self.timed.try_pull(ts, &mut self.mixer, buf) {
                Some(v) => v,
                None => break,
            };
            
            self.last_mix_ts = Some(mixed_ts);

            let pts = mixed_ts.rescale(milli_time_base(), self.encoder.get_time_base());
            self.mixed_frame.set_pts(Some(pts));

            // println!(
            //     "audio mixed_frame: rate {}, ch {}, format {:?}",
            //     self.mixed_frame.rate(),
            //     self.mixed_frame.channels(),
            //     self.mixed_frame.format(),
            // );

            let frame = self.mixed_resampler.resample_whole(&self.mixed_frame).unwrap();
            self.encoder.send_frame(&frame).unwrap();
            while let Some(mut packet) = self.encoder.receive_packet().unwrap() {
                writer.write_packet(&self.o_track, self.encoder.get_time_base(), &mut packet).unwrap();
            }
        }

        // if elapsed >= 40 {
            
        //     self.last_mix_ts = Some(ts);

        //     let mut image = self.timed.try_pull(ts, &mut self.mixer, buf)

        //     let ts = ts.rescale(milli_time_base(), self.encoder.get_time_base());
        //     image.frame_mut().set_pts(Some(ts));

        //     println!("mix video: decode pts {:?}", image.frame().pts());
        //     self.encoder.send_frame(image.frame()).unwrap();

        //     while let Some(mut packet) = self.encoder.receive_packet().unwrap() {
        //         println!("mix video: wrote pts {:?}, time_base {:?}", packet.pts(), self.encoder.get_time_base());
        //         writer.write_packet(&self.o_track, self.encoder.get_time_base(), &mut packet).unwrap();
        //     }
        // }
    }

    fn try_handle_audio(&mut self, flow: &Flow, packet: &ff::Packet) -> bool {
        if let Some(track) = self.tracks.get_mut(&flow.index) {
            // ff::util::frame::Video::new(format, width, height)
            // decoder.time_base();
            track.decoder.send_packet(packet).unwrap();
            while let Some(frame) = audio_decoder_receive_frame(&mut track.decoder).unwrap() {
                let frame = track.resampler.resample_whole(&frame).unwrap();
                let samples = audio_frame_packed_i16_samples(&frame);
                println!(
                    "audio mixer update: id {:?}, samples {}, rate {}, ch {}, format {:?}",
                    track.id,
                    samples.len(),
                    frame.rate(),
                    frame.channels(),
                    frame.format(),
                );

                self.mixer.update_ch(&track.id, samples).unwrap();
            }
            true
        } else {
            false
        }
    }

    fn add_aac_flow(&mut self, flow: &Flow) {
        // let args = parse_h264(&flow.codec).unwrap();
        let id = self.mixer.add_ch().unwrap();
        let decoder = make_audio_decoder(
            ff::codec::Id::AAC, 
            flow.codec.clock_rate as i32, 
            flow.codec.channels.unwrap_or(1) as i32, 
            milli_time_base(),
        ).unwrap();
        
        // let enc_params = self.encoder.get_parameters();
        
        let resampler = SResampler::get(
            decoder.format(),
            decoder.channel_layout(), 
            decoder.rate(), 
             self.mixed_frame.format(),
            ChannelLayout::default(self.mixed_frame.channels() as i32), 
            self.mixed_frame.rate() as u32,
        ).unwrap();

        self.tracks.insert(flow.index, MixAudioTrack {
            decoder,
            id,
            resampler,
        });
    }    
}


struct MixAudioTrack {
    decoder: ff::codec::decoder::Audio,
    resampler: SResampler,
    id: AChId,
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
    // otrack: Option<OTrack>,
    // depacker: Option<Box<dyn RtpCodecDepacker>>,
}

struct FlowExt {
    otrack_index: Option<usize>,
}



fn parse_h264(codec: &SdpCodec) -> Result<FFVideoArgs>{
    let fmtp: &str = codec.fmtps
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

    Ok(FFVideoArgs {
        codec_id: ff::codec::Id::H264,
        width: param.generic.pixel_dimensions.0 as i32,
        height: param.generic.pixel_dimensions.1 as i32,
        extra: spspps.freeze(),
        time_base: ff::Rational::new(1, 90000),
    })
}
