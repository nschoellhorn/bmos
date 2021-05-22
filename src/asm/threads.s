.global __cleanup_thread
.extern cleanup_thread

__cleanup_thread:
    cli
    popq %rdi
    jmp cleanup_thread

test:
    hlt
