.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # 用户栈到内核栈的切换， sp -> 内核栈, sscratch -> 用户栈
    addi sp, sp, -34*8
    # 准备TrapContext的空间，32个用户寄存器 + 2个控制寄存器
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    csrr t0, sstatus
    csrr t1, sepc

    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    csrr t2, sscratch
    sd t2, 2*8(sp)

    mv a0, sp
    # set TrapContext address to trap_handler argument
    call trap_handler

__restore:
    mv sp, a0
    # 将返回值给sp，返回值就是TrapContext的地址

    # 将控制寄存器恢复
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    

    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    # 将通用寄存器恢复
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    addi sp, sp, 34*8
    # 归还栈空间
    csrrw sp, sscratch, sp
    # sp->user stack, sscratch->kernel stack
    sret