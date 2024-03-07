use std::{fs::File, io::Write, path::Path, str::FromStr};
use anyhow::{anyhow, Context, Result};

use bytes::Bytes;


use crate::{rtp::{codec::h264::{RtpDepackerH264, RtpH264Parameters}, depack::RtpCodecDepacker}, tlv2::{TlvFileSyncReader, Type, VecBuf}, tlv_custom::{ChInfo, TlvType, TLV_MAGIC}};

#[test]
fn test_tlv_to_h264() {

    let output_tlv = "/tmp/output.tlv2";
    let output_h264 = "/tmp/output.h264";

    super::poc_tlv_to_h264::tlv_to_h264(output_tlv.as_ref(), output_h264.as_ref()).unwrap();
}

pub fn tlv_to_h264(input: &Path, output: &Path) -> Result<()> {
    // spspps: &Option<Vec<(Vec<u8>, Vec<Vec<u8>>)>>

    let mut reader = TlvFileSyncReader::open_with_magic(&input, Some(TLV_MAGIC))
        .with_context(||format!("failed open [{input:?}]"))?;
    println!("opened input {input:?}");

    let mut writer = File::create(&output)
        .with_context(||format!("failed open [{output:?}]"))?;
    println!("opened output {output:?}");

    // if let Some(spspps) = spspps {
    //     for item in spspps.iter() {
    //         let sps = &item.0;
    //         let pps_vec = &item.1;
    //         writer.write_all(&[0, 0, 1])?;
    //         writer.write_all(&sps)?;
    //         for pps in pps_vec.iter() {
    //             writer.write_all(&[0, 0, 1])?;
    //             writer.write_all(&pps)?;
    //         }
    //     }
    // }

    let mut buf = VecBuf::default();
    let buf  = &mut buf;


    let mut h264_deapck = RtpDepackerH264::new(Some("packetization-mode=1; sprop-parameter-sets=Z0LAFdoCAJbARAAAAwAEAAADAPA8WLqA,aM4yyA==; profile-level-id=42C015"))?;


    // let mut h264_deapck = RtpDepackerH264::default();

    // let mut h264_depack = H264Packet::default();
    // h264_depack.is_avc = true;

    

    loop {
        let tag = reader.read_tag(buf)
            .with_context(||"read next tlv failed")?;

        let rtype = tag.rtype();
        if rtype.is_build_in() {
            match rtype {
                Type::ATTACH_END => {}
                Type::FILE_END => {
                    println!("got file end\n");
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
                        println!("read: add_ch, ts [{ts}], content [{}]-[{content}]\n", content.len());

                        let info: ChInfo = serde_json::from_str(content)?;

                        let sdp = sdp_rs::SessionDescription::from_str(&info.sdp)?; 
                        // let sdp = webrtc_sdp::parse_sdp(&info.sdp, false)?;
                        println!("  {sdp:?}\n");

                        
                        let (pt, fmtp) = parse_h264_fmtp_from_sdp(&info.sdp).unwrap().unwrap();
                        println!("  found h264: [{pt}]-[{fmtp}]\n");

                        let parames = RtpH264Parameters::parse_from_str(&fmtp).unwrap();
                        println!("  h264 param: {parames:?}\n");

                        writer.write_all(&[0, 0, 1])?;
                        writer.write_all(&parames.sps_nal)?;

                        writer.write_all(&[0, 0, 1])?;
                        writer.write_all(&parames.pps_nal)?;

                    }
                    TlvType::ChData => {
                        let mut value = tag.value();
                        let ts = value.cut_var_i64()?;
                        let ch_id = value.cut_var_u64()?;
                        let data = value.as_slice();
                        println!("read: ch_data, ts {ts}, ch {ch_id}, len {}", data.len());
                        if ch_id == 0 {
                            // let payload = {
                            //     let rtp = rtp_rs::RtpReader::new(data)
                            //     .map_err(|e|anyhow!("invalid rtp [{e:?}]"))?;
                            //     Bytes::copy_from_slice(rtp.payload())
                            // };

                            // let frame = h264_depack.depacketize(&payload)?;

                            h264_deapck.push_rtp_slice(&data)?;
                            let frame = h264_deapck.pull_frame().unwrap().unwrap_or(Bytes::new());

                            if frame.len() > 0 {
                                let h = h264_reader::nal::NalHeader::new(frame[4])
                                .map_err(|e|anyhow!("invalid nalu header [{e:?}]"))?;
                                println!("  nalu={}, {h:?}", (frame[4]& 0x1F));
                                writer.write_all(&frame)?;
                            }
                        }
                    },
                    _ => {
                        println!("read: unhandle {rtype:?}");
                    }
                }
            },
            Err(_e) => {
                println!("unknown tlv type [{rtype:?}]");
            },
        }
    }

    Ok(())
}

fn parse_h264_fmtp_from_sdp(sdp: &str) -> Result<Option<(u8, String)>> {
    let sdp = sdp_rs::SessionDescription::from_str(sdp)?; 
    for media in sdp.media_descriptions.iter() {
        let mut h264_payload_type = None;
        for attr in media.attributes.iter() {
            match attr {
                sdp_rs::lines::Attribute::Rtpmap(rtpmap)  => {
                    if rtpmap.encoding_name.eq_ignore_ascii_case("H264") {
                        if h264_payload_type.is_none() {
                            h264_payload_type = Some(rtpmap.payload_type as u8);
                        }
                    }
                }
                sdp_rs::lines::Attribute::Other(name, value) => {
                    if name == "fmtp" {
                        if let Some(value) = value {
                            let r = value.split_once(' ');
                            if let Some((num_str, fmtp)) = r {
                                let payload_type: Option<u8> = num_str.parse().ok();
                                if payload_type == h264_payload_type {
                                    return Ok(payload_type.map(|x|(x, fmtp.to_string())))
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

        }
    }

    Ok(None)
}
