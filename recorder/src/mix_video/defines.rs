
use ffmpeg_next as ff;

use std::collections::BTreeMap;

use enumflags2::{bitflags, BitFlags};

use crate::ffeasy::video::{image::FFYuvImage, scaler::FFAutoScaler};

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VChFlag {
    ShareScreen     = 0b_0000_0001,
    Talker          ,
}

pub type VChFlags = BitFlags<VChFlag>;

pub(super) type VChannels = BTreeMap<VChId, VChannel>;

pub(super) struct VChannel {
    pub(super) image: Option<FFYuvImage>,
    pub(super) flags: VChFlags,
    pub(super) scaler: FFAutoScaler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VChId(pub(super) u64);

pub type Result<T> = std::result::Result<T, ff::Error>;

pub(super) trait LayoutOp {
    // fn add_ch(&mut self, ch_id: VChId, flags: VChFlags) -> Result<()> {
    //     Ok(())
    // }
    // fn remove_ch(&mut self, ch_id: &VChId) -> Result<()> {
    //     Ok(())
    // }
    // fn update_ch(&mut self, ch_id: &VChId, image: &FFYuvImage) -> Result<()> {
    //     Ok(())
    // }

    fn get_output(&mut self, channels: &mut VChannels, image: &mut FFYuvImage) -> Result<()>;
}

