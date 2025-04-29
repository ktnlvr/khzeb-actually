type DirtyFlagBlock = u64;
const DIRTY_FLAG_BITS_PER_BLOCK: usize = 64;

#[derive(Clone, Copy)]
pub struct DirtyFlags<const N: usize> {
    blocks: [DirtyFlagBlock; N],
}

impl<const N: usize> Default for DirtyFlags<N> {
    fn default() -> Self {
        Self { blocks: [0; N] }
    }
}

impl<const N: usize> DirtyFlags<N> {
    pub const SIZE: usize = DIRTY_FLAG_BITS_PER_BLOCK * N;

    pub fn new() -> Self {
        Self::default()
    }

    // TODO: error-check the bounds
    pub fn mark(&mut self, idx: usize) {
        let block_idx = idx / DIRTY_FLAG_BITS_PER_BLOCK;
        let bit_idx = idx % DIRTY_FLAG_BITS_PER_BLOCK;
        self.blocks[block_idx] |= 1 << bit_idx;
    }

    pub fn iter_bits(&self) -> impl Iterator<Item = bool> + '_ {
        self.blocks
            .iter()
            .flat_map(|b| (0..DIRTY_FLAG_BITS_PER_BLOCK).map(move |i| (b >> i) & 1 != 0))
    }

    pub fn iter_marked(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter_bits()
            .enumerate()
            .filter_map(|(i, b)| Some(i).filter(|_| b))
    }

    pub fn clear(&mut self) {
        for block in self.blocks.iter_mut() {
            *block = 0u64;
        }
    }
}
