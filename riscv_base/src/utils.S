
.section .text.idle
.globl sleep
sleep:
    addi a0, a0, -1
    bge a0, zero, sleep
    ret

.globl exec_at
exec_at:
    lui     s1,0x420
    li      a1,0x87
    sw      a1,32(s1)
    sw      a0,36(s1)
    li      a1,0xff
    sw      a1,40(s1)
    sw      zero,64(s1)
    li a1, 0x100000
exec_loop:
    addi a1, a1, -1
    bge a1, zero, sleep
    jr a0
