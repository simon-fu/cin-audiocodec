use std::path::Path;
use anyhow::{bail, Context, Result};


use chrono::Local;

use ffmpeg_next::{format::input_with_dictionary, media::Type as FFType};


// use rtp::{codecs::h264::H264Packet, packetizer::Depacketizer};

use crate::{ffeasy::{gen_sdp::gen_av_only_sdp, rtp_mem::{load_rtp_mem_sync, RtpMemCursor}}, tlv_custom::{ChInfo, TlvCustomFileWriter}};

#[test]
fn test_mp4_to_tlv() {
    let input = "/tmp/sample-data/sample.mp4";
    let output_tlv = "/tmp/output.tlv2";

    let _spspps = mp4_to_tlv(input.as_ref(), output_tlv.as_ref(), None ).unwrap();

    // let output_h264 = "/tmp/output.h264";
    // tlv_to_h264(output_tlv.as_ref(), output_h264.as_ref()).await.unwrap();

    let output_mp4 = "/tmp/output.h264";
    super::poc_tlv_to_h264::tlv_to_h264(output_tlv.as_ref(), output_mp4.as_ref()).unwrap();
}




fn mp4_to_tlv(input: &Path, output: &Path, max_frames: Option<u64>) -> Result<Option<Vec<(Vec<u8>, Vec<Vec<u8>>)>>> {

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
