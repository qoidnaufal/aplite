/// A simple bitset which can contains `size_of::<usize>() * 8` bits
/// This assume little endian, in which `0b01001` means the bitset contains id `[0, 3]`
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitset(usize);

impl Bitset {
    const SIZE: usize = size_of::<Self>() * 8;

    pub fn new(num: usize) -> Self {
        Self(num)
    }

    /// say you have this:     0b000000000
    /// and you want to add index 4: ^
    /// the bits are indexed from right to left: 0b0000(1)0000
    /// the bit you want to add is at index:       8765(4)3210
    /// so you pass 4 to [`Bitset::add_bit`]
    pub fn add_bit(&mut self, index: usize) {
        self.0 |= 1 << index
    }

    /// say you have this: 0b111010001
    /// and you want to remove ^
    /// the bits are indexed from right to left: 0b1110(1)0001
    /// the bit you want to remove is at index:    8765(4)3210
    /// so you pass 4 to [`Bitset::remove_bit`]
    pub fn remove_bit(&mut self, index: usize) {
        self.0 ^= 1 << index
    }

    pub fn contains(&self, other: &Bitset) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn contains_index(&self, id: usize) -> bool {
        self.0 & (1 << id) == (1 << id)
    }

    /// this assume little endian
    pub fn from_array(input: &[usize]) -> Self {
        debug_assert!(input.len() <= Self::SIZE);

        Self(input.iter().map(|i| 1 << i).sum())
    }

    /// this assume little endian
    pub fn as_bytes(&self) -> Box<[u8]> {
        self.iter().collect()
    }

    #[inline(always)]
    /// this assume little endian
    pub fn iter(&self) -> impl Iterator<Item = u8> {
        (0..Self::SIZE).filter_map(|i| {
            let b = (((self.0 >> i) & 0x1) * i) as u8;
            if i > 0 && b == 0 {
                None
            } else {
                Some(b)
            }
        })
    }
}

impl std::hash::Hash for Bitset {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0);
    }
}

impl std::fmt::Debug for Bitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl std::fmt::Display for Bitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod bitset_test {
    use super::*;

    #[test]
    fn as_bytes() {
        let bitset = Bitset(0b1001010001);

        assert!(bitset.contains_index(0));
        assert!(bitset.contains_index(4));
        assert!(bitset.contains_index(6));
        assert!(bitset.contains_index(9));

        assert_eq!(bitset.as_bytes().as_ref(), &[0, 4, 6, 9]);
    }
}
