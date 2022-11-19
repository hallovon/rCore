use spin::Mutex;

use crate::BLOCK_SZ;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::*;

use crate::BlockDevice;

/// 与磁盘中数据块映射的内存中的数据块
pub struct BlockCache {
    // 用于存储数据块的数据
    cache: Vec<u8>,
    // 对应于磁盘中数据块编号
    block_id: usize,
    // 获取blockdevice引用，便于内存和磁盘交互
    block_device: Arc<dyn BlockDevice>,
    // 标记该磁盘缓存是否被修改
    modified: bool,
}

impl BlockCache {
    /// 从磁盘中加载一个数据块到内存中
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
        let mut cache = vec![0u8; BLOCK_SZ];
        block_device.read_block(block_id, &mut cache);
        Self {
            cache,
            block_id,
            block_device,
            modified: false,
        }
    }

    /// 获取数据块中指定偏移量的物理地址
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    /// 从指定偏移量位置获取T类型数据引用
    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    /// 从指定偏移量位置获取T类型数据可变引用
    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        self.modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }

    /// 获取数据块中offset偏移量的数据引用，将其传入闭包f中作为输入参数，
    /// 执行内在逻辑，并得到一个返回值
    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    /// 获取数据块中offset偏移量的数据`可变引用`，将其传入闭包f中作为输入参数，
    /// 执行内在逻辑，并得到一个返回值
    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }

    /// 将内存中修改过的数据块刷新回磁盘中
    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            self.block_device.write_block(self.block_id, &self.cache);
        }
    }
}

impl Drop for BlockCache {
    /// 满足RAII，释放blockcache资源时，将修改过的内容刷回磁盘
    fn drop(&mut self) {
        self.sync();
    }
}

const BLOCK_CACHE_SIZE: usize = 16;

/// blockcache管理器，内部维护一个队列
pub struct BlockCacheManager {
    // 队列保存对应的block_id和BlockCache的引用
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// 获取指定block_id的数据块的blockcache
    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        block_device: Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BlockCache>> {
        // 如果队列有对应的blockcache,直接返回
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Arc::clone(&pair.1)
        } else {
            // 如果缓存队列已满，检查是否有不使用的blockcache，若有，删除，否则，直接报错
            if self.queue.len() == BLOCK_CACHE_SIZE {
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.queue.drain(idx..=idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }

            // 创建新的blockcache并加入到缓存队列中
            let block_cache = Arc::new(Mutex::new(BlockCache::new(block_id, block_device.clone())));
            self.queue.push_back((block_id, block_cache.clone()));
            block_cache
        }
    }
}

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::new());
}

/// 对外部接口，获取指定block_id的数据块
pub fn get_block_cache(
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
) -> Arc<Mutex<BlockCache>> {
    BLOCK_CACHE_MANAGER
        .lock()
        .get_block_cache(block_id, block_device)
}

/// 对外部接口，将所有修改过的数据块缓存刷新回磁盘
pub fn block_cache_sync_all() {
    let manager = BLOCK_CACHE_MANAGER.lock();
    for (_, cache) in manager.queue.iter() {
        cache.lock().sync();
    }
}
