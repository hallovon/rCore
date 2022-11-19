use crate::trap::trap_return;

#[repr(C)]
pub struct TaskContext {
    /// trap_return函数的地址
    ra: usize,
    /// 内核栈地址
    sp: usize,
    /// 通用寄存器
    s: [usize; 12],
}

impl TaskContext {
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
