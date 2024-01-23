use core::num::NonZeroUsize;

#[derive(Debug, Clone, Copy)]
pub struct BitSet(u64);

impl BitSet {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, off: usize) {
        self.0 |= 1 << off
    }

    pub fn difference(&self, Self(other_bits): Self) -> Self {
        Self(self.0 & !other_bits)
    }
}

impl std::iter::IntoIterator for BitSet {
    type Item = NonZeroUsize;
    type IntoIter = BitSetIter;

    fn into_iter(self) -> Self::IntoIter {
        BitSetIter(self.0)
    }
}

#[derive(Debug)]
pub struct BitSetIter(u64);

impl std::iter::Iterator for BitSetIter {
    type Item = NonZeroUsize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0 => None,
            bits => {
                let idx = bits.trailing_zeros() as usize;
                self.0 ^= 1 << idx;
                NonZeroUsize::new(idx)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
    }
}

impl std::iter::ExactSizeIterator for BitSetIter {}
impl std::iter::FusedIterator for BitSetIter {}
