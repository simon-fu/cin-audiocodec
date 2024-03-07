
use std::num::{NonZeroU16, NonZeroU32};

use anyhow::{Result, anyhow};
use bytes::{Buf, BufMut, Bytes};

use super::super::depack::RtpCodecDepacker;

pub type RetinaDepackH264 = RetinaDepack<PostPullAnnexb>;

pub fn make_retina_depack_h264(fmtp: Option<&str>) -> Result<RetinaDepackH264> {
    let clock_rate = 90000;
    let depack = retina::codec::Depacketizer::new(
        "video", 
        "h264", 
        clock_rate, 
        None, 
        fmtp,
    ).map_err(anyhow::Error::msg)?;

    Ok(RetinaDepack {
        depack,
        ctx: retina::PacketContext::dummy(),
        clock_rate,
        post: PostPullAnnexb(),
    } )
}

pub type RetinaDepackAAC = RetinaDepack<PostPullAudio>;

pub fn make_retina_depack_aac(
    clock_rate: u32,
    channels: Option<NonZeroU16>,
    fmtp: Option<&str>,
) -> Result<RetinaDepackAAC> {
    let depack = retina::codec::Depacketizer::new(
        "audio", 
        "mpeg4-generic", 
        clock_rate, 
        channels, 
        fmtp,
    ).map_err(anyhow::Error::msg)?;

    Ok( RetinaDepack {
        depack,
        ctx: retina::PacketContext::dummy(),
        clock_rate,
        post: PostPullAudio(),
    } )
}


pub trait PostPullOp {
    fn post_pull(&mut self, item: retina::codec::CodecItem) -> Result<Option<Bytes>>;
}

pub struct PostPullAnnexb();
impl PostPullOp for PostPullAnnexb {
    fn post_pull(&mut self, item: retina::codec::CodecItem) -> Result<Option<Bytes>> {
        match item {
            retina::codec::CodecItem::VideoFrame(frame) => {
                let mut frame = frame.into_data();
                let mut data = &mut frame[..];
                while data.len() >= 4 {
                    let unit_len = (&data[..4]).get_u32() as usize;
                    data.put(&[0, 0, 0, 1][..]);
                    data = &mut data[unit_len..];
                }

                Ok(Some(frame.into()))
            }
            _ => Ok(None)
        }
    }
}

pub struct PostPullAudio();
impl PostPullOp for PostPullAudio {
    fn post_pull(&mut self, item: retina::codec::CodecItem) -> Result<Option<Bytes>> {
        match item {
            retina::codec::CodecItem::AudioFrame(frame) => {
                let frame = Bytes::copy_from_slice(frame.data()) ;
                Ok(Some(frame.into()))
            }
            _ => Ok(None)
        }
    }
}


pub struct RetinaDepack<P> {
    ctx: retina::PacketContext,
    depack: retina::codec::Depacketizer,
    clock_rate: u32,
    post: P,
}


impl<P: PostPullOp + 'static> RetinaDepack<P> {

    pub fn into_box(self) -> Box<dyn RtpCodecDepacker> {
        Box::new(self)
    }

    pub fn input_slice(&mut self, data: &[u8]) -> Result<()> {
        let rtp = rtp_rs::RtpReader::new(data)
        .map_err(|e|anyhow!("invalid rtp [{e:?}]"))?;

        let packet = retina::rtp::ReceivedPacketBuilder {
            ctx: self.ctx.clone(),
            stream_id: 0,
            sequence_number: rtp.sequence_number().into(),
            timestamp: retina::Timestamp::new(0, NonZeroU32::new(self.clock_rate).unwrap(), 0).unwrap(),
            payload_type: rtp.payload_type(),
            ssrc: rtp.ssrc(),
            mark: rtp.mark(),
            loss: 0,
        }
        .build(Bytes::copy_from_slice(rtp.payload()))
        .map_err(|e|anyhow!("{e}"))?;

        self.depack.push(packet)
        .map_err(|e|anyhow!("{e}"))?;

        Ok(())
    }
}

impl<P: PostPullOp + 'static> RtpCodecDepacker for RetinaDepack<P> {
    fn push_rtp_slice(&mut self, rtp: &[u8]) -> Result<()>  {
        self.input_slice(rtp)?;
        Ok(())
    }

    fn pull_frame(&mut self) -> Result<Option<Bytes>>  {
        
        let r = self.depack.pull(
            &retina::ConnectionContext::dummy(),
            &retina::StreamContext::dummy(),
        )?;

        match r {
            Some(item) => self.post.post_pull(item),
            None => Ok(None),
        }
    }
}
