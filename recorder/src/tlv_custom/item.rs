

use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct ChInfo {
    pub name: String,
    pub ch_id: u64,
    pub sdp: String,
}
