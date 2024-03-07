use base64::Engine;
use bytes::{BufMut, Bytes, BytesMut};
use h264_reader::nal::UnitType;


// refer from retina::codec::h264::InternalParameters
#[derive(Clone)]
pub struct RtpH264Parameters {
    pub generic: VideoParameters,

    /// The (single) SPS NAL.
    pub sps_nal: Bytes,

    /// The (single) PPS NAL.
    pub pps_nal: Bytes,

    pub packetization_mode: u8,
}

impl std::fmt::Debug for RtpH264Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H264Parameters")
            .field("generic", &self.generic)
            .field(
                "sps",
                &LimitedHex::new(&self.sps_nal, 16),
            )
            .field(
                "pps",
                &LimitedHex::new(&self.pps_nal, 16),
            )
            .finish()
    }
}

impl RtpH264Parameters {

    pub fn parse_from_str(fmtp: &str) -> Result<Self, String> {
        let mut sprop_parameter_sets = None;
        let mut pack_mode = None;

        for p in fmtp.split(';') {
            match p.trim().split_once('=') {
                Some(("sprop-parameter-sets", value)) => sprop_parameter_sets = Some(value),
                Some(("packetization-mode", value)) => pack_mode = Some(value),
                None => return Err("key without value".into()),
                _ => (),
            }
        }

        let sprop_parameter_sets = sprop_parameter_sets
            .ok_or_else(|| "no sprop-parameter-sets in H.264 format-specific-params".to_string())?;

        let pack_mode = pack_mode.unwrap_or("1");
        let pack_mode: u8 = pack_mode.parse().map_err(|_e|"invalid packetization-mode".to_string())?;
    
        let mut sps_nal = None;
        let mut pps_nal = None;
        for nal in sprop_parameter_sets.split(',') {
            let nal = base64::engine::general_purpose::STANDARD
                .decode(nal)
                .map_err(|_| {
                    "bad sprop-parameter-sets: NAL has invalid base64 encoding".to_string()
                })?;
            if nal.is_empty() {
                return Err("bad sprop-parameter-sets: empty NAL".into());
            }
            let header = h264_reader::nal::NalHeader::new(nal[0])
                .map_err(|_| format!("bad sprop-parameter-sets: bad NAL header {:0x}", nal[0]))?;
            match header.nal_unit_type() {
                UnitType::SeqParameterSet => {
                    if sps_nal.is_some() {
                        return Err("multiple SPSs".into());
                    }
                    sps_nal = Some(nal);
                }
                UnitType::PicParameterSet => {
                    if pps_nal.is_some() {
                        return Err("multiple PPSs".into());
                    }
                    pps_nal = Some(nal);
                }
                _ => return Err("only SPS and PPS expected in parameter sets".into()),
            }
        }
        let sps_nal = sps_nal.ok_or_else(|| "no sps".to_string())?;
        let pps_nal = pps_nal.ok_or_else(|| "no pps".to_string())?;
        Self::parse_sps_and_pps(&sps_nal, &pps_nal, pack_mode)
    }
    
    fn parse_sps_and_pps(sps_nal: &[u8], pps_nal: &[u8], packetization_mode: u8) -> Result<Self, String> {
        let sps_rbsp = h264_reader::rbsp::decode_nal(sps_nal).map_err(|_| "bad sps")?;
        if sps_rbsp.len() < 5 {
            return Err("bad sps".into());
        }
        let rfc6381_codec = format!(
            "avc1.{:02X}{:02X}{:02X}",
            sps_rbsp[0], sps_rbsp[1], sps_rbsp[2]
        );
        let sps = h264_reader::nal::sps::SeqParameterSet::from_bits(
            h264_reader::rbsp::BitReader::new(&*sps_rbsp),
        )
        .map_err(|e| format!("Bad SPS: {e:?}"))?;
        // debug!("sps: {:#?}", &sps);

        let pixel_dimensions = sps
            .pixel_dimensions()
            .map_err(|e| format!("SPS has invalid pixel dimensions: {e:?}"))?;

        // Create the AVCDecoderConfiguration, ISO/IEC 14496-15 section 5.2.4.1.
        // The beginning of the AVCDecoderConfiguration takes a few values from
        // the SPS (ISO/IEC 14496-10 section 7.3.2.1.1).
        let mut avc_decoder_config = BytesMut::with_capacity(11 + sps_nal.len() + pps_nal.len());
        avc_decoder_config.put_u8(1); // configurationVersion
        avc_decoder_config.extend(&sps_rbsp[0..=2]); // profile_idc . AVCProfileIndication
                                                     // ...misc bits... . profile_compatibility
                                                     // level_idc . AVCLevelIndication

        // Hardcode lengthSizeMinusOne to 3, matching TransformSampleData's 4-byte
        // lengths.
        avc_decoder_config.put_u8(0xff);

        // Only support one SPS and PPS.
        // ffmpeg's ff_isom_write_avcc has the same limitation, so it's probably
        // fine. This next byte is a reserved 0b111 + a 5-bit # of SPSs (1).
        avc_decoder_config.put_u8(0xe1);
        avc_decoder_config.extend(
            &u16::try_from(sps_nal.len())
                .map_err(|_| format!("SPS NAL is {} bytes long; must fit in u16", sps_nal.len()))?
                .to_be_bytes()[..],
        );
        let sps_nal_start = avc_decoder_config.len();
        avc_decoder_config.extend_from_slice(sps_nal);
        let sps_nal_end = avc_decoder_config.len();
        avc_decoder_config.put_u8(1); // # of PPSs.
        avc_decoder_config.extend(
            &u16::try_from(pps_nal.len())
                .map_err(|_| format!("PPS NAL is {} bytes long; must fit in u16", pps_nal.len()))?
                .to_be_bytes()[..],
        );
        let pps_nal_start = avc_decoder_config.len();
        avc_decoder_config.extend_from_slice(pps_nal);
        let pps_nal_end = avc_decoder_config.len();
        assert_eq!(avc_decoder_config.len(), 11 + sps_nal.len() + pps_nal.len());

        let (pixel_aspect_ratio, frame_rate);
        match sps.vui_parameters {
            Some(ref vui) => {
                pixel_aspect_ratio = vui
                    .aspect_ratio_info
                    .as_ref()
                    .and_then(|a| a.clone().get())
                    .map(|(h, v)| (u32::from(h), (u32::from(v))));

                // TODO: study H.264, (E-34). This quick'n'dirty calculation isn't always right.
                frame_rate = vui.timing_info.as_ref().and_then(|t| {
                    t.num_units_in_tick
                        .checked_mul(2)
                        .map(|doubled| (doubled, t.time_scale))
                });
            }
            None => {
                pixel_aspect_ratio = None;
                frame_rate = None;
            }
        }
        let avc_decoder_config = avc_decoder_config.freeze();
        let sps_nal = avc_decoder_config.slice(sps_nal_start..sps_nal_end);
        let pps_nal = avc_decoder_config.slice(pps_nal_start..pps_nal_end);
        Ok(Self {
            generic: VideoParameters {
                rfc6381_codec,
                pixel_dimensions,
                pixel_aspect_ratio,
                frame_rate,
                extra_data: avc_decoder_config,
            },
            sps_nal,
            pps_nal,
            packetization_mode,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VideoParameters {
    pub pixel_dimensions: (u32, u32),
    pub rfc6381_codec: String,
    pub pixel_aspect_ratio: Option<(u32, u32)>,
    pub frame_rate: Option<(u32, u32)>,
    pub extra_data: Bytes,
}

impl std::fmt::Debug for VideoParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoParameters")
            .field("rfc6381_codec", &self.rfc6381_codec)
            .field("pixel_dimensions", &self.pixel_dimensions)
            .field("pixel_aspect_ratio", &self.pixel_aspect_ratio)
            .field("frame_rate", &self.frame_rate)
            .field(
                "extra_data",
                &LimitedHex::new(&self.extra_data, 16),
            )
            .finish()
    }
}

use pretty_hex::PrettyHex;

pub struct LimitedHex<'a> {
    inner: &'a [u8],
    max_bytes: usize,
}

impl<'a> LimitedHex<'a> {
    pub fn new(inner: &'a [u8], max_bytes: usize) -> Self {
        Self { inner, max_bytes }
    }
}

impl<'a> std::fmt::Debug for LimitedHex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:#?}]",
            self.inner.hex_conf( {
                let mut cfg = pretty_hex::HexConfig::simple();
                cfg.max_bytes = self.max_bytes;
                cfg
            })
        )
    }
}


