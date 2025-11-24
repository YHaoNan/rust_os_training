.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # sp -> UserStack, sscratch -> 进入trap之前执行任务的TrapContext（在restore中返回前被设置给sscratch）
    csrrw sp, sscratch, sp

    # 现在 sscratch -> UserStack, sp -> TrapContext

    # 保存通用寄存器到TrapContext
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # 保存sstatus和sepc，供restore中返回使用。现在我们可以使用t0和t1，因为已经被存在TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # 将user stack位置存储到TrapContext第2个通用寄存器（即sp）
    csrr t2, sscratch
    sd t2, 2*8(sp)
    
    # 加载内核栈，内核页表，跳转到trap_handler
    ld t0, 34*8(sp) # satp(kernel page table)
    ld t1, 36*8(sp) # trap_handler
    ld sp, 35*8(sp) # 内核栈位置

    csrw satp, t0   # 至此，切换至内核页表
    sfence.vma # 清除TLB

    jr t1 # 进入trap_handler
    # 这里不直接使用call trap_handler，因为它会被编译成两条相对寻址指令（相对当前pc加上偏移量），而当前我们的执行VA实际上在Trampoline中，而不在内核alltraps的实际位置，但trap_handler并不在trampoline
    # 这里直接使用绝对地址

__restore:
    # 代码中进入__restore时携带的参数，由trap_return设置
    # in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
    # in("a1") user_satp,        // a1 = phy addr of usr page table

    # 核心任务：恢复TrapContext、设置sscratch、设置sp => 用户页表、设置sepc/sstatus，最后sret
    
    # 切换到用户地址空间，然后才能操作并恢复TrapContext
    csrw satp, a1
    sfence.vma

    # 设置正确的sscratch -> trap_cx_ptr
    csrw sscratch, a0

    # 将sp设置成a0
    mv sp, a0

    # 恢复sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # 恢复通用寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    # 恢复用户栈
    ld sp, 2*8(sp)
    sret
