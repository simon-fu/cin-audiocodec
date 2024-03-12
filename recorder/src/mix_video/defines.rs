
use ffmpeg_next as ff;

use std::collections::HashMap;

use enumflags2::{bitflags, BitFlags};

use crate::ffeasy::video::image::FFYuvImage;

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VChFlag {
    ShareScreen     = 0b_0000_0001,
    Talker          ,
}

pub type VChFlags = BitFlags<VChFlag>;

pub(super) type VChannels = HashMap<VChId, VChannel>;

pub(super) struct VChannel {
    pub(super) image: Option<FFYuvImage>,
    pub(super) flags: VChFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VChId(pub(super) u64);

pub type Result<T> = std::result::Result<T, ff::Error>;

pub(super) trait LayoutOp {
    fn add_ch(&mut self, ch_id: VChId, flags: VChFlags) -> Result<()>;
    fn remove_ch(&mut self, ch_id: &VChId) -> Result<()>;
    fn update_ch(&mut self, ch_id: &VChId, image: &FFYuvImage) -> Result<()>;
    fn get_output(&mut self, channels: &VChannels) -> Result<&FFYuvImage>;
}

