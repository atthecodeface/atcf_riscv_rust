.section .vectors, "ax"
.global _start
.globl __main
.weak  __main
.equ   __main, wait_forever

//.equ SLEEP_AMOUNT, 0x400000
.equ SLEEP_AMOUNT, 0x40

_start:
    .cfi_startproc
    .cfi_undefined ra

    // Set trap handler
    la t0, _start_trap
    csrw mtvec, t0

    // Set global pointer
    .option push
    .option norelax
    //la gp, __global_pointer$
    .option pop

    lui     s3, 0x470
    li      s4, 0xc0
    sw      s4,8(s3) # 470008 - FB SRAM control

    lui     a0,0x190
    li      a1,53
    sw      a1,4(a0)

    li a0, SLEEP_AMOUNT
start_pause:    
    addi a0, a0, -1
    bge a0, zero, start_pause

    lui     s1,0x420
    li      a0,0x0a0d0a0d
    sw      a0,32(s1)
    li      a0,0x52656164
    sw      a0,36(s1)
    li      a0,0x792021ff
    sw      a0,40(s1)
    sw      zero,44(s1)
    sw      zero,64(s1)

    lui sp, %hi(_stack_base)
    add sp, sp, %lo(_stack_base)

    lui     s1,0x420
    li      a0,0x4d656d87 # Mem
    sw      a0,32(s1)
    sw      a0, 0(sp)
    li      a0,-1

    sw      a0,40(s1)
    lw      a1, 0(sp)
    sw      a1,36(s1)
    sw      zero,64(s1)
    
    li a0, SLEEP_AMOUNT
start_pause2:    
    addi a0, a0, -1
    bge a0, zero, start_pause2

    jal zero, __main

    .cfi_endproc

    
_start_trap:
wait_forever:
    j wait_forever
    
.section .init
.globl abort
abort:
    j abort
