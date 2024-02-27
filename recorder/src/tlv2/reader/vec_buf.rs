
/// Buffer for read operation
#[derive(Debug, Default, Clone)]
pub struct VecBuf {
    vec: Vec<u8>,
    pos: usize,
}

impl VecBuf {
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self { vec, pos: 0 }
    }

    pub fn clear(&mut self) {
        self.pos = 0;
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.vec[..self.pos]
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [u8] {
        let limit = self.vec.len();
        &mut self.vec[self.pos..limit]
    }

    pub fn spare_mut<'a>(&'a mut self, additional: usize) -> Spare<'a> {
        let limit = self.pos + additional;
        
        if limit > self.vec.len() {
            self.vec.resize(limit, 0);
        }
        
        Spare { owner: self, limit }
    }
}

pub struct Spare<'a> {
    owner: &'a mut VecBuf,
    limit: usize,
}

impl<'a> Spare<'a> {
    pub fn buf(&mut self) -> &mut [u8] {
        &mut self.owner.vec[self.owner.pos..self.limit]
    }

    pub fn take_up(&mut self, n: usize) {
        self.owner.pos += n;
    }

    pub fn is_full(&self) -> bool {
        self.owner.pos >= self.limit
    }
}
