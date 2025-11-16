// public code
.section .text._start

// fn _start()
_start:
.L_parking_loop:
        wfe // wait for events
        b           .L_parking_loop

.size _start, . - _start
.type _start, function
.global _start
