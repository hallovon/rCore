use core::any::Any;

/// 用于和磁盘上指定数据块block交互
pub trait BlockDevice: Send + Sync + Any {
    // 读取编号为block_id的数据块中数据到buf中
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    // 将缓冲区buf中数据写入到block_id号数据块
    fn write_block(&self, block_id: usize, buf: &[u8]);
}