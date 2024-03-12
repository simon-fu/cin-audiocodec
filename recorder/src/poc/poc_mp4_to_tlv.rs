use std::{path::Path, sync::Arc};
use anyhow::{bail, Context, Result};


use chrono::Local;

use ffmpeg_next::{format::input_with_dictionary, media::Type as FFType};


// use rtp::{codecs::h264::H264Packet, packetizer::Depacketizer};

use crate::{ffeasy::{gen_sdp::gen_av_only_sdp, rtp_mem::{load_rtp_mem_sync, RtpMemCursor, RtpMemData, RtpMemPacket}}, tlv_custom::{ChInfo, TlvCustomFileWriter}};



#[test]
fn test_multi_mp4_to_tlv() {
    // let input = "/tmp/sample-data/sample.mp4";
    let output_tlv = "/tmp/output.tlv2";

    let inputs: Vec<&Path> = vec![
        "/tmp/sample-data/sample.mp4".as_ref(),
        "/tmp/ForBiggerBlazes.mp4".as_ref(),
        // "/Users/simon/Downloads/sample-5s.mp4".as_ref(),
    ];

    multi_mp4_to_tlv(inputs.iter(), output_tlv.as_ref(), None ).unwrap();
}

#[test]
fn test_single_mp4_to_tlv() {
    let input = "/tmp/sample-data/sample.mp4";
    let output_tlv = "/tmp/output.tlv2";

    single_mp4_to_tlv(input.as_ref(), output_tlv.as_ref(), None ).unwrap();

    // let output_mp4 = "/tmp/output.h264";
    // super::poc_tlv_to_h264::tlv_to_h264(output_tlv.as_ref(), output_mp4.as_ref()).unwrap();
}


fn multi_mp4_to_tlv<'a, I, P>( 
    inputs: I, 
    output: &Path, 
    max_frames: Option<u64>,
) -> Result<()> 
where 
    I: Iterator<Item = &'a P>,
    P: AsRef<Path> + 'static,
{

    let mut sources = Vec::new();

    let mut start_ch_id = 0;
    for input in inputs { 
        sources.push(RtpMemSource::open(input.as_ref(), max_frames, start_ch_id, 0)?);
        start_ch_id += 8;
        // sources.push(RtpMemSource::open(input.as_ref(), max_frames, start_ch_id, 100)?);
    }



    let mut ofile = TlvCustomFileWriter::open(&output)
        .with_context(||format!("failed open [{output:?}]"))?;
    println!("opened output {output:?}");


    ofile.write_header()
        .with_context(||"write file header failed")?;

    for src in sources.iter() {

        let info = ChInfo {
            name: format!("user-{}", src.start_ch_id),
            ch_id: src.start_ch_id,
            sdp: src.sdp.clone(),
        };

        println!("wrote ch  {info:?}");
        ofile.write_adding_ch(&info)?;
    }

    let mut reader = SourcesReader {
        packet_slots: vec![None; sources.len()],
        sources,
    };

    
    let mut first_pts: Option<i64> = None;
    let start_at_milli = Local::now().timestamp_millis();
    // let start_at_milli = 0;
    

    while let Some(rtp) = reader.read_next()? {

        let first_pts = match first_pts {
            Some(first_pts) => first_pts,
            None => {
                let pts = rtp.pts();
                first_pts = Some(pts);
                pts
            },
        };

        let ts = start_at_milli + (rtp.pts() - first_pts);

        let ch_id = rtp.ch_id() as u64;
        ofile.write_ch_data_with_ts(ch_id, rtp.data(), ts)?;
        // println!("wrote ch {ch_id}, ts {ts}, len {}", rtp.data().len());
    }

    ofile.write_file_end()?;

    Ok(())
}

struct RtpMemSource {
    start_ch_id: u64,
    time_offset: i64,
    que: Arc<RtpMemData>,
    cursor: RtpMemCursor,
    sdp: String,
}

impl RtpMemSource {
    pub fn open(input: &Path, max_frames: Option<u64>, start_ch_id: u64, time_offset: i64) -> Result<Self> {
        let ictx = input_with_dictionary(&input, Default::default())
        .with_context(||format!("failed open [{input:?}]"))?;
        println!("opened input {input:?}");

        let video_index = ictx.streams().best(FFType::Video)
            .map(|s|s.index());
        
        let audio_index = ictx.streams().best(FFType::Audio)
            .map(|s|s.index());

        if video_index.is_none() && audio_index.is_none() {
            bail!("Not found video or audio in [{:?}]", input)
        }

        let rtp_mem = load_rtp_mem_sync(input, max_frames.unwrap_or(u64::MAX))?;

        let (sdp, _octx) = gen_av_only_sdp(&ictx)?;

        Ok(Self {
            que: rtp_mem,
            start_ch_id,
            time_offset,
            cursor: RtpMemCursor::default(),
            sdp,
        })
    }

    pub fn read_next(&mut self) -> Result<Option<RtpMemPacket>> {
        match self.que.read_at(&mut self.cursor) {
            Some(mut packet) => {
                packet.set_ts(
                    packet.pts() + self.time_offset, 
                    packet.dts() + self.time_offset, 
                );
                packet.set_ch_id(packet.ch_id() + self.start_ch_id);
                Ok(Some(packet))
            },
            None => Ok(None),
        }
    }
}


struct SourcesReader {
    sources: Vec<RtpMemSource>, 
    packet_slots: Vec<Option<RtpMemPacket>>,
}

impl SourcesReader {
    pub fn read_next(&mut self) -> Result<Option<RtpMemPacket>> {
        match self.fill_slots()? {
            Some(index) => {
                Ok(self.packet_slots[index].take())
            },
            None => Ok(None),
        }
    }

    fn fill_slots(&mut self) -> Result<Option<usize>> {
        let mut min_index = None;
        let mut min_pts = i64::MAX;

        for (index, slot) in self.packet_slots.iter_mut().enumerate() {
            let pts = match slot {
                Some(packet) => {
                    packet.pts()
                },
                None => {
                    match self.sources[index].read_next()? {
                        Some(packet) => {
                            let pts = packet.pts();
                            *slot = Some(packet);
                            pts
                        },
                        None => i64::MAX,
                    }
                },
            };

            if pts < min_pts {
                min_index = Some(index);
                min_pts = pts;
            }
        }

        Ok(min_index)
    }
}

fn single_mp4_to_tlv(input: &Path, output: &Path, max_frames: Option<u64>) -> Result<Option<Vec<(Vec<u8>, Vec<Vec<u8>>)>>> {

    let ictx = input_with_dictionary(&input, Default::default())
        .with_context(||format!("failed open [{input:?}]"))?;
    println!("opened input {input:?}");

    let video_index = ictx.streams().best(FFType::Video)
        .map(|s|s.index());
    
    let audio_index = ictx.streams().best(FFType::Audio)
        .map(|s|s.index());

    if video_index.is_none() && audio_index.is_none() {
        bail!("Not found video or audio in [{:?}]", input)
    }

    let (sdp, _octx) = gen_av_only_sdp(&ictx)?;


    let mut ofile = TlvCustomFileWriter::open(&output)
        .with_context(||format!("failed open [{output:?}]"))?;
    println!("opened output {output:?}");


    ofile.write_header()
        .with_context(||"write file header failed")?;

    println!("wrote sdp len {}", sdp.len());
    ofile.write_adding_ch(&ChInfo {
        name: "first".into(),
        ch_id: 0,
        sdp,
    })?;
    

    let max_frames = max_frames.unwrap_or(u64::MAX);
    let rtp_mem = load_rtp_mem_sync(input, max_frames)?;
    let spspps = rtp_mem.video().as_ref().map(|track| {
        track.spspps().clone()
    });

    let mut first_pts: Option<i64> = None;
    let start_at_milli = Local::now().timestamp_millis();
    
    let mut cursor = RtpMemCursor::default();

    while let Some(rtp) = rtp_mem.read_at(&mut cursor) {

        let first_pts = match first_pts {
            Some(first_pts) => first_pts,
            None => {
                let pts = rtp.pts();
                first_pts = Some(pts);
                pts
            },
        };

        let ts = start_at_milli + (rtp.pts() - first_pts);

        let ch_id = rtp.ch_id() as u64;
        ofile.write_ch_data_with_ts(ch_id, rtp.data(), ts)?;
        println!("wrote ch {ch_id}, len {}", rtp.data().len());
    }

    ofile.write_file_end()?;

    Ok(spspps)
}
