use alloc::sync::Arc;

use crate::{block_cache::get_block_cache, BlockDevice, BLOCK_SZ};

type BitmapBlock = [u64; 64];

const BLOCK_BITS: usize = BLOCK_SZ * 8;

pub struct Bitmap {
    // 磁盘中位图的开始序号
    start_block_id: usize,
    // 位图使用block数目
    blocks: usize,
}

impl Bitmap {
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    /// 位图分配一个block，返回block的id,该id为位图内的相对id，不是磁盘上的绝对id
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            let pos = get_block_cache(block_id + self.start_block_id, block_device.clone())
                .lock()
                .modify(0, |bitmap_block: &mut BitmapBlock| {
                    if let Some((bits64_pos, inner_pos)) = bitmap_block
                        .iter()
                        .enumerate()
                        .find(|(_, bits64)| **bits64 != u64::MAX)
                        .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                    {
                        bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                        Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos)
                    } else {
                        None
                    }
                });

            if pos.is_some() {
                return pos;
            }
        }

        None
    }

    /// 位图中根据bit删除标记
    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        get_block_cache(block_pos + self.start_block_id, block_device.clone())
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                // 判断指定的block_id是否存在
                assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
                bitmap_block[bits64_pos] -= 1u64 << inner_pos;
            });
    }

    /// 返回最多可使用的block数
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}

/// 将得到的位图中的block的相对id，解构成(block_pos, bits64_pos, inner_pos)
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    bit %= BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}
