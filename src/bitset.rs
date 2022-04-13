pub struct U64BitSet<const N: usize> {
    buf: [u64; N],
}

impl<const N: usize> U64BitSet<N> {
    pub fn new() -> Self {
        Self { buf: [0; N] }
    }

    pub fn insert<T: Into<u32> + Copy>(&mut self, value: T) {
        let idx = value.into() / u64::BITS;
        let off = value.into() % u64::BITS;
        self.buf[usize::try_from(idx).unwrap()] |= 1 << off;
    }

    pub fn contains<T: Into<u32> + Copy>(&self, value: T) -> bool {
        let idx = value.into() / u64::BITS;
        let off = value.into() % u64::BITS;
        (self.buf[usize::try_from(idx).unwrap()] & 1 << off) != 0
    }

    pub fn clear(&mut self) {
        self.buf.fill(0);
    }
}
