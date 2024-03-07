pub struct FmtpParser<'a> {
    iter: std::str::Split<'a, char>,
}

impl<'a> FmtpParser<'a> {
    pub fn new(fmtp: &'a str) -> Self {
        Self {
            iter: fmtp.split(';'),
        }
    }
}

impl<'a> Iterator for FmtpParser<'a>  {
    type Item = KVStr<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x|KVStr(x))
    }
}

pub struct KVStr<'a>(pub &'a str);

impl<'a> KVStr<'a> {
    pub fn as_key_value(&self) -> Option<(&'a str, &str)> {
        self.0.trim().split_once('=')
    }
}


// pub fn parse_fmtp(fmtp: &str) {
//     let split = fmtp.split(';');

//     for p in fmtp.split(';') {
//         match p.trim().split_once('=') {
//             Some(("sprop-parameter-sets", value)) => sprop_parameter_sets = Some(value),
//             Some(("packetization-mode", value)) => pack_mode = Some(value),
//             None => return Err("key without value".into()),
//             _ => (),
//         }
//     }
// }
