
.section .text.idle
.globl sleep
sleep:
    addi a0, a0, -1
    bge a0, zero, sleep
    ret

.globl exec_at
exec_at:
    jr a0
