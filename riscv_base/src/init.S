.section .vectors, "ax"
.global _start
.globl __main
.weak  __main
.equ   __main, wait_forever

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

    lui sp, %hi(_stack_base)
    add sp, sp, %lo(_stack_base)
    jal zero, __main

    .cfi_endproc

    
_start_trap:
wait_forever:
    j wait_forever
    
.section .init
.globl abort
abort:
    j abort
