// from webrtc_rs


use anyhow::{anyhow, bail, Result};
use bytes::{BufMut, Bytes, BytesMut};
use rtp_rs::RtpReader;
use super::super::depack::RtpCodecDepacker;


#[derive(Default)]
pub struct RtpDepackerWebrtcRsH264 {
    is_avc: bool,
    fua_buffer: Option<BytesMut>,
    frame: Option<Bytes>,
    // depack: H264Packet,
}

impl RtpCodecDepacker for RtpDepackerWebrtcRsH264 {
    fn push_rtp_slice(&mut self, payload: &[u8]) -> Result<()>  {
        // self.depack.depacketize(b);

        let rtp = RtpReader::new(payload)
        .map_err(|e|anyhow!("invalid rtp {e:?}"))?;

        let payload = rtp.payload();

        if payload.len() <= 2 {
            // return Err(Error::ErrShortPacket);
            bail!("short packet")
        }

        let mut frame_buf = BytesMut::new();

        // NALU Types
        // https://tools.ietf.org/html/rfc6184#section-5.4
        let b0 = payload[0];
        let nalu_type = b0 & NALU_TYPE_BITMASK;

        match nalu_type {
            1..=23 => {
                if self.is_avc {
                    frame_buf.put_u32(payload.len() as u32);
                } else {
                    frame_buf.put(&*ANNEXB_NALUSTART_CODE);
                }
                // frame_buf.put(&*packet.clone());
                frame_buf.put(payload);

                self.frame = Some(frame_buf.freeze());
                Ok(())
            }
            STAPA_NALU_TYPE => {
                let mut curr_offset = STAPA_HEADER_SIZE;
                while curr_offset < payload.len() {
                    let nalu_size =
                        ((payload[curr_offset] as usize) << 8) | payload[curr_offset + 1] as usize;
                    curr_offset += STAPA_NALU_LENGTH_SIZE;

                    if payload.len() < curr_offset + nalu_size {
                        bail!("StapA size too large ")
                        // return Err(Error::StapASizeLargerThanBuffer(
                        //     nalu_size,
                        //     packet.len() - curr_offset,
                        // ));
                    }

                    if self.is_avc {
                        frame_buf.put_u32(nalu_size as u32);
                    } else {
                        frame_buf.put(&*ANNEXB_NALUSTART_CODE);
                    }
                    // frame_buf.put(&*packet.slice(curr_offset..curr_offset + nalu_size));
                    frame_buf.put(&payload[curr_offset..curr_offset + nalu_size]);
                    curr_offset += nalu_size;
                }

                self.frame = Some(frame_buf.freeze());
                Ok(())
            }
            FUA_NALU_TYPE => {
                if payload.len() < FUA_HEADER_SIZE {
                    // return Err(Error::ErrShortPacket);
                    bail!("short packet")
                }

                if self.fua_buffer.is_none() {
                    self.fua_buffer = Some(BytesMut::new());
                }

                if let Some(fua_buffer) = &mut self.fua_buffer {
                    // fua_buffer.put(&*packet.slice(FUA_HEADER_SIZE..));
                    fua_buffer.put(&payload[FUA_HEADER_SIZE..]);
                }

                let b1 = payload[1];
                if b1 & FU_END_BITMASK != 0 {
                    let nalu_ref_idc = b0 & NALU_REF_IDC_BITMASK;
                    let fragmented_nalu_type = b1 & NALU_TYPE_BITMASK;

                    if let Some(fua_buffer) = self.fua_buffer.take() {
                        if self.is_avc {
                            frame_buf.put_u32((fua_buffer.len() + 1) as u32);
                        } else {
                            frame_buf.put(&*ANNEXB_NALUSTART_CODE);
                        }
                        frame_buf.put_u8(nalu_ref_idc | fragmented_nalu_type);
                        frame_buf.put(fua_buffer);
                    }

                    self.frame = Some(frame_buf.freeze());
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => {
                bail!("unknown nalu type [{nalu_type}]")
                // Err(Error::NaluTypeIsNotHandled(nalu_type))
            },
        }
    }

    fn pull_frame(&mut self) -> Result<Option<Bytes>>  {
        Ok(self.frame.take())
    }
}

pub const STAPA_NALU_TYPE: u8 = 24;
pub const FUA_NALU_TYPE: u8 = 28;
// pub const FUB_NALU_TYPE: u8 = 29;
// pub const SPS_NALU_TYPE: u8 = 7;
// pub const PPS_NALU_TYPE: u8 = 8;
// pub const AUD_NALU_TYPE: u8 = 9;
// pub const FILLER_NALU_TYPE: u8 = 12;

pub const FUA_HEADER_SIZE: usize = 2;
pub const STAPA_HEADER_SIZE: usize = 1;
pub const STAPA_NALU_LENGTH_SIZE: usize = 2;

pub const NALU_TYPE_BITMASK: u8 = 0x1F;
pub const NALU_REF_IDC_BITMASK: u8 = 0x60;
// pub const FU_START_BITMASK: u8 = 0x80;
pub const FU_END_BITMASK: u8 = 0x40;

// pub const OUTPUT_STAP_AHEADER: u8 = 0x78;

pub static ANNEXB_NALUSTART_CODE: Bytes = Bytes::from_static(&[0x00, 0x00, 0x00, 0x01]);
