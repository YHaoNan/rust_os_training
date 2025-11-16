use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32], // 用户寄存器
    pub sstatus: Sstatus, // 控制寄存器，包括SPP（之前的特权级）、SIE（是否允许全局中断）、SPIE（异常发生前SIE的值）
    pub sepc: usize // ecall时自动被设置成当前被中断的用户指令地址（ecall指令地址） / 执行sret时会读取这个地址并跳转
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn app_init_context(entry: usize, sp: usize) -> Self { // 应用初始TrapContext
        println!("app_init_context: entry => {:#x}, sp => {:#x}", entry, sp);
        // entry = 应用程序入口, sp = 用户栈地址
        let mut sstatus = sstatus::read(); // CSR sstatus
        sstatus.set_spp(SPP::User); //previous privilege mode: user mode
        let mut cx = Self {
            x: [0; 32], // zero reg files
            sstatus,    // user mode previously
            sepc: entry // sepc to entry
        };
        cx.set_sp(sp);  // sp to user stack
        // when `ret` executed
        // 1. reg files will be recovered
        // 2. set pc to sepc (user program entry)
        // 3. set sp = $sp (user stack)
        cx
    }
}