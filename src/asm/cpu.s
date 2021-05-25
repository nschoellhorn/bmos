.att_syntax prefix

.section .text

.global __switch_context
.global __init_switch

.align 16
__switch_context:
    pushfq
    pushq %rbx
    pushq %rbp
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    movq %rsp, (%rdi)
    movq (%rsi), %rsp
    mov %cr0, %rax
    or $8, %rax
    mov %rax, %cr0
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbp
    popq %rbx
    popfq
    movq %rsi, %rdi
    retq

.align 16
__init_switch:
    movq (%rdi), %rsp
    mov %cr0, %rax
    or $8, %rax
    mov %rax, %cr0
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbp
    popq %rbx
    popfq
    retq
