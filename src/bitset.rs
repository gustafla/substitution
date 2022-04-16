/// A set of N*64 bits that can be individually addressed
#[derive(Clone, Copy)]
pub struct BitSet64<const N: usize> {
    buf: [u64; N],
}

impl<const N: usize> BitSet64<N> {
    /// Creates a `BitSet64` with all zeroes
    pub fn new() -> Self {
        Self { buf: [0; N] }
    }

    /// Set a bit at index corresponding to `value` to 1
    pub fn insert<T: Into<u32> + Copy>(&mut self, value: T) {
        let idx = value.into() / u64::BITS;
        let off = value.into() % u64::BITS;
        self.buf[usize::try_from(idx).unwrap()] |= 1 << off;
    }

    /// Set a bit at index corresponding to `value` to 0
    pub fn remove<T: Into<u32> + Copy>(&mut self, value: T) {
        let idx = value.into() / u64::BITS;
        let off = value.into() % u64::BITS;
        self.buf[usize::try_from(idx).unwrap()] &= !(1 << off);
    }

    /// Query if the bit corresponding to `value` is 1
    pub fn contains<T: Into<u32> + Copy>(&self, value: T) -> bool {
        let idx = value.into() / u64::BITS;
        let off = value.into() % u64::BITS;
        (self.buf[usize::try_from(idx).unwrap()] & 1 << off) != 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bs64_mixed_operations() {
        let mut bs = BitSet64::<4>::new();
        for i in 0u32..100 {
            bs.insert(i);
        }
        for i in 4u32..8 {
            bs.remove(i);
        }

        for i in 0u32..4 {
            assert!(bs.contains(i));
        }
        for i in 4u32..8 {
            assert!(!bs.contains(i));
        }
        for i in 8u32..100 {
            assert!(bs.contains(i));
        }
        for i in 100u32..256 {
            assert!(!bs.contains(i));
        }
    }
}
