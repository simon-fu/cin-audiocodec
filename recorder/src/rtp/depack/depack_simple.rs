
use anyhow::{Result, anyhow};
use bytes::Bytes;
use super::RtpCodecDepacker;


#[derive(Default)]
pub struct RtpDepackerSimpleAudio {
    frame: Option<Bytes>,
}

impl RtpCodecDepacker for RtpDepackerSimpleAudio {

    fn push_rtp_slice(&mut self, rtp: &[u8]) -> Result<()>  {
        let rtp = rtp_rs::RtpReader::new(rtp)
        .map_err(|e|anyhow!("invalid rtp [{e:?}]"))?;
        self.frame = Some(Bytes::copy_from_slice(rtp.payload()));
        Ok(())
    }

    fn pull_frame(&mut self) -> Result<Option<Bytes>>  {
        Ok(self.frame.take())
    }
}
