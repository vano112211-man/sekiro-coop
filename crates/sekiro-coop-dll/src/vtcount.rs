//! Per-slot call counter for PlayerIns virtual methods (run 57).
//!
//! Every hooked method gets its own tiny assembly stub that counts the call
//! when `rcx` is the Hero or the clone, then jumps to the original (all
//! registers and the stack untouched, so any signature is safe). Comparing the
//! two count tables shows which per-frame methods run for the Hero but never
//! for the clone.

#![allow(non_upper_case_globals)]

pub const N: usize = 256;

#[no_mangle]
pub static mut VTC_HERO: usize = 0;
#[no_mangle]
pub static mut VTC_CLONE: usize = 0;
#[no_mangle]
pub static mut VTC_ORIG: [usize; N] = [0; N];
#[no_mangle]
pub static mut VTC_H: [u64; N] = [0; N];
#[no_mangle]
pub static mut VTC_C: [u64; N] = [0; N];
#[no_mangle]
pub static mut VTC_ALL: [u64; N] = [0; N];

core::arch::global_asm!(r#"
    .text
    .globl vtc_stub_0
vtc_stub_0:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 0]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 0]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 0]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 0]
    .globl vtc_stub_1
vtc_stub_1:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 8]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 8]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 8]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 8]
    .globl vtc_stub_2
vtc_stub_2:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 16]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 16]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 16]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 16]
    .globl vtc_stub_3
vtc_stub_3:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 24]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 24]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 24]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 24]
    .globl vtc_stub_4
vtc_stub_4:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 32]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 32]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 32]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 32]
    .globl vtc_stub_5
vtc_stub_5:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 40]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 40]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 40]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 40]
    .globl vtc_stub_6
vtc_stub_6:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 48]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 48]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 48]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 48]
    .globl vtc_stub_7
vtc_stub_7:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 56]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 56]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 56]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 56]
    .globl vtc_stub_8
vtc_stub_8:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 64]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 64]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 64]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 64]
    .globl vtc_stub_9
vtc_stub_9:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 72]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 72]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 72]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 72]
    .globl vtc_stub_10
vtc_stub_10:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 80]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 80]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 80]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 80]
    .globl vtc_stub_11
vtc_stub_11:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 88]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 88]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 88]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 88]
    .globl vtc_stub_12
vtc_stub_12:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 96]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 96]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 96]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 96]
    .globl vtc_stub_13
vtc_stub_13:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 104]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 104]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 104]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 104]
    .globl vtc_stub_14
vtc_stub_14:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 112]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 112]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 112]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 112]
    .globl vtc_stub_15
vtc_stub_15:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 120]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 120]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 120]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 120]
    .globl vtc_stub_16
vtc_stub_16:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 128]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 128]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 128]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 128]
    .globl vtc_stub_17
vtc_stub_17:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 136]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 136]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 136]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 136]
    .globl vtc_stub_18
vtc_stub_18:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 144]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 144]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 144]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 144]
    .globl vtc_stub_19
vtc_stub_19:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 152]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 152]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 152]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 152]
    .globl vtc_stub_20
vtc_stub_20:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 160]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 160]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 160]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 160]
    .globl vtc_stub_21
vtc_stub_21:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 168]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 168]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 168]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 168]
    .globl vtc_stub_22
vtc_stub_22:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 176]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 176]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 176]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 176]
    .globl vtc_stub_23
vtc_stub_23:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 184]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 184]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 184]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 184]
    .globl vtc_stub_24
vtc_stub_24:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 192]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 192]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 192]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 192]
    .globl vtc_stub_25
vtc_stub_25:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 200]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 200]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 200]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 200]
    .globl vtc_stub_26
vtc_stub_26:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 208]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 208]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 208]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 208]
    .globl vtc_stub_27
vtc_stub_27:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 216]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 216]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 216]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 216]
    .globl vtc_stub_28
vtc_stub_28:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 224]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 224]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 224]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 224]
    .globl vtc_stub_29
vtc_stub_29:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 232]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 232]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 232]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 232]
    .globl vtc_stub_30
vtc_stub_30:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 240]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 240]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 240]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 240]
    .globl vtc_stub_31
vtc_stub_31:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 248]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 248]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 248]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 248]
    .globl vtc_stub_32
vtc_stub_32:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 256]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 256]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 256]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 256]
    .globl vtc_stub_33
vtc_stub_33:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 264]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 264]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 264]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 264]
    .globl vtc_stub_34
vtc_stub_34:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 272]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 272]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 272]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 272]
    .globl vtc_stub_35
vtc_stub_35:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 280]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 280]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 280]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 280]
    .globl vtc_stub_36
vtc_stub_36:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 288]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 288]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 288]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 288]
    .globl vtc_stub_37
vtc_stub_37:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 296]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 296]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 296]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 296]
    .globl vtc_stub_38
vtc_stub_38:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 304]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 304]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 304]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 304]
    .globl vtc_stub_39
vtc_stub_39:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 312]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 312]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 312]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 312]
    .globl vtc_stub_40
vtc_stub_40:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 320]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 320]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 320]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 320]
    .globl vtc_stub_41
vtc_stub_41:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 328]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 328]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 328]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 328]
    .globl vtc_stub_42
vtc_stub_42:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 336]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 336]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 336]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 336]
    .globl vtc_stub_43
vtc_stub_43:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 344]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 344]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 344]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 344]
    .globl vtc_stub_44
vtc_stub_44:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 352]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 352]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 352]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 352]
    .globl vtc_stub_45
vtc_stub_45:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 360]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 360]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 360]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 360]
    .globl vtc_stub_46
vtc_stub_46:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 368]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 368]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 368]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 368]
    .globl vtc_stub_47
vtc_stub_47:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 376]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 376]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 376]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 376]
    .globl vtc_stub_48
vtc_stub_48:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 384]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 384]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 384]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 384]
    .globl vtc_stub_49
vtc_stub_49:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 392]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 392]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 392]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 392]
    .globl vtc_stub_50
vtc_stub_50:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 400]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 400]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 400]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 400]
    .globl vtc_stub_51
vtc_stub_51:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 408]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 408]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 408]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 408]
    .globl vtc_stub_52
vtc_stub_52:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 416]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 416]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 416]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 416]
    .globl vtc_stub_53
vtc_stub_53:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 424]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 424]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 424]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 424]
    .globl vtc_stub_54
vtc_stub_54:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 432]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 432]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 432]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 432]
    .globl vtc_stub_55
vtc_stub_55:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 440]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 440]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 440]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 440]
    .globl vtc_stub_56
vtc_stub_56:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 448]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 448]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 448]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 448]
    .globl vtc_stub_57
vtc_stub_57:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 456]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 456]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 456]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 456]
    .globl vtc_stub_58
vtc_stub_58:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 464]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 464]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 464]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 464]
    .globl vtc_stub_59
vtc_stub_59:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 472]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 472]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 472]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 472]
    .globl vtc_stub_60
vtc_stub_60:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 480]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 480]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 480]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 480]
    .globl vtc_stub_61
vtc_stub_61:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 488]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 488]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 488]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 488]
    .globl vtc_stub_62
vtc_stub_62:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 496]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 496]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 496]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 496]
    .globl vtc_stub_63
vtc_stub_63:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 504]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 504]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 504]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 504]
    .globl vtc_stub_64
vtc_stub_64:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 512]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 512]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 512]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 512]
    .globl vtc_stub_65
vtc_stub_65:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 520]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 520]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 520]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 520]
    .globl vtc_stub_66
vtc_stub_66:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 528]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 528]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 528]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 528]
    .globl vtc_stub_67
vtc_stub_67:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 536]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 536]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 536]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 536]
    .globl vtc_stub_68
vtc_stub_68:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 544]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 544]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 544]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 544]
    .globl vtc_stub_69
vtc_stub_69:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 552]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 552]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 552]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 552]
    .globl vtc_stub_70
vtc_stub_70:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 560]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 560]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 560]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 560]
    .globl vtc_stub_71
vtc_stub_71:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 568]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 568]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 568]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 568]
    .globl vtc_stub_72
vtc_stub_72:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 576]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 576]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 576]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 576]
    .globl vtc_stub_73
vtc_stub_73:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 584]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 584]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 584]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 584]
    .globl vtc_stub_74
vtc_stub_74:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 592]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 592]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 592]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 592]
    .globl vtc_stub_75
vtc_stub_75:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 600]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 600]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 600]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 600]
    .globl vtc_stub_76
vtc_stub_76:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 608]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 608]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 608]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 608]
    .globl vtc_stub_77
vtc_stub_77:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 616]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 616]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 616]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 616]
    .globl vtc_stub_78
vtc_stub_78:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 624]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 624]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 624]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 624]
    .globl vtc_stub_79
vtc_stub_79:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 632]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 632]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 632]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 632]
    .globl vtc_stub_80
vtc_stub_80:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 640]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 640]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 640]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 640]
    .globl vtc_stub_81
vtc_stub_81:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 648]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 648]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 648]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 648]
    .globl vtc_stub_82
vtc_stub_82:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 656]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 656]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 656]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 656]
    .globl vtc_stub_83
vtc_stub_83:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 664]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 664]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 664]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 664]
    .globl vtc_stub_84
vtc_stub_84:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 672]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 672]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 672]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 672]
    .globl vtc_stub_85
vtc_stub_85:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 680]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 680]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 680]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 680]
    .globl vtc_stub_86
vtc_stub_86:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 688]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 688]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 688]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 688]
    .globl vtc_stub_87
vtc_stub_87:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 696]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 696]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 696]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 696]
    .globl vtc_stub_88
vtc_stub_88:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 704]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 704]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 704]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 704]
    .globl vtc_stub_89
vtc_stub_89:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 712]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 712]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 712]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 712]
    .globl vtc_stub_90
vtc_stub_90:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 720]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 720]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 720]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 720]
    .globl vtc_stub_91
vtc_stub_91:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 728]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 728]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 728]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 728]
    .globl vtc_stub_92
vtc_stub_92:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 736]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 736]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 736]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 736]
    .globl vtc_stub_93
vtc_stub_93:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 744]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 744]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 744]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 744]
    .globl vtc_stub_94
vtc_stub_94:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 752]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 752]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 752]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 752]
    .globl vtc_stub_95
vtc_stub_95:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 760]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 760]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 760]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 760]
    .globl vtc_stub_96
vtc_stub_96:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 768]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 768]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 768]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 768]
    .globl vtc_stub_97
vtc_stub_97:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 776]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 776]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 776]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 776]
    .globl vtc_stub_98
vtc_stub_98:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 784]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 784]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 784]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 784]
    .globl vtc_stub_99
vtc_stub_99:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 792]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 792]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 792]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 792]
    .globl vtc_stub_100
vtc_stub_100:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 800]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 800]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 800]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 800]
    .globl vtc_stub_101
vtc_stub_101:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 808]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 808]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 808]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 808]
    .globl vtc_stub_102
vtc_stub_102:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 816]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 816]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 816]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 816]
    .globl vtc_stub_103
vtc_stub_103:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 824]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 824]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 824]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 824]
    .globl vtc_stub_104
vtc_stub_104:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 832]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 832]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 832]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 832]
    .globl vtc_stub_105
vtc_stub_105:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 840]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 840]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 840]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 840]
    .globl vtc_stub_106
vtc_stub_106:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 848]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 848]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 848]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 848]
    .globl vtc_stub_107
vtc_stub_107:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 856]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 856]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 856]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 856]
    .globl vtc_stub_108
vtc_stub_108:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 864]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 864]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 864]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 864]
    .globl vtc_stub_109
vtc_stub_109:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 872]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 872]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 872]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 872]
    .globl vtc_stub_110
vtc_stub_110:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 880]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 880]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 880]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 880]
    .globl vtc_stub_111
vtc_stub_111:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 888]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 888]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 888]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 888]
    .globl vtc_stub_112
vtc_stub_112:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 896]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 896]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 896]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 896]
    .globl vtc_stub_113
vtc_stub_113:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 904]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 904]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 904]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 904]
    .globl vtc_stub_114
vtc_stub_114:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 912]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 912]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 912]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 912]
    .globl vtc_stub_115
vtc_stub_115:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 920]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 920]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 920]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 920]
    .globl vtc_stub_116
vtc_stub_116:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 928]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 928]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 928]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 928]
    .globl vtc_stub_117
vtc_stub_117:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 936]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 936]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 936]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 936]
    .globl vtc_stub_118
vtc_stub_118:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 944]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 944]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 944]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 944]
    .globl vtc_stub_119
vtc_stub_119:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 952]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 952]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 952]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 952]
    .globl vtc_stub_120
vtc_stub_120:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 960]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 960]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 960]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 960]
    .globl vtc_stub_121
vtc_stub_121:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 968]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 968]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 968]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 968]
    .globl vtc_stub_122
vtc_stub_122:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 976]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 976]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 976]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 976]
    .globl vtc_stub_123
vtc_stub_123:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 984]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 984]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 984]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 984]
    .globl vtc_stub_124
vtc_stub_124:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 992]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 992]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 992]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 992]
    .globl vtc_stub_125
vtc_stub_125:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1000]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1000]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1000]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1000]
    .globl vtc_stub_126
vtc_stub_126:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1008]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1008]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1008]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1008]
    .globl vtc_stub_127
vtc_stub_127:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1016]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1016]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1016]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1016]
    .globl vtc_stub_128
vtc_stub_128:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1024]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1024]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1024]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1024]
    .globl vtc_stub_129
vtc_stub_129:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1032]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1032]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1032]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1032]
    .globl vtc_stub_130
vtc_stub_130:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1040]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1040]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1040]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1040]
    .globl vtc_stub_131
vtc_stub_131:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1048]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1048]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1048]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1048]
    .globl vtc_stub_132
vtc_stub_132:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1056]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1056]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1056]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1056]
    .globl vtc_stub_133
vtc_stub_133:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1064]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1064]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1064]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1064]
    .globl vtc_stub_134
vtc_stub_134:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1072]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1072]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1072]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1072]
    .globl vtc_stub_135
vtc_stub_135:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1080]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1080]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1080]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1080]
    .globl vtc_stub_136
vtc_stub_136:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1088]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1088]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1088]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1088]
    .globl vtc_stub_137
vtc_stub_137:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1096]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1096]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1096]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1096]
    .globl vtc_stub_138
vtc_stub_138:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1104]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1104]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1104]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1104]
    .globl vtc_stub_139
vtc_stub_139:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1112]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1112]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1112]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1112]
    .globl vtc_stub_140
vtc_stub_140:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1120]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1120]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1120]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1120]
    .globl vtc_stub_141
vtc_stub_141:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1128]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1128]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1128]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1128]
    .globl vtc_stub_142
vtc_stub_142:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1136]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1136]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1136]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1136]
    .globl vtc_stub_143
vtc_stub_143:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1144]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1144]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1144]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1144]
    .globl vtc_stub_144
vtc_stub_144:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1152]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1152]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1152]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1152]
    .globl vtc_stub_145
vtc_stub_145:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1160]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1160]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1160]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1160]
    .globl vtc_stub_146
vtc_stub_146:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1168]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1168]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1168]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1168]
    .globl vtc_stub_147
vtc_stub_147:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1176]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1176]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1176]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1176]
    .globl vtc_stub_148
vtc_stub_148:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1184]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1184]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1184]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1184]
    .globl vtc_stub_149
vtc_stub_149:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1192]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1192]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1192]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1192]
    .globl vtc_stub_150
vtc_stub_150:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1200]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1200]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1200]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1200]
    .globl vtc_stub_151
vtc_stub_151:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1208]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1208]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1208]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1208]
    .globl vtc_stub_152
vtc_stub_152:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1216]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1216]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1216]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1216]
    .globl vtc_stub_153
vtc_stub_153:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1224]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1224]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1224]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1224]
    .globl vtc_stub_154
vtc_stub_154:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1232]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1232]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1232]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1232]
    .globl vtc_stub_155
vtc_stub_155:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1240]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1240]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1240]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1240]
    .globl vtc_stub_156
vtc_stub_156:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1248]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1248]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1248]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1248]
    .globl vtc_stub_157
vtc_stub_157:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1256]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1256]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1256]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1256]
    .globl vtc_stub_158
vtc_stub_158:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1264]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1264]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1264]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1264]
    .globl vtc_stub_159
vtc_stub_159:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1272]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1272]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1272]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1272]
    .globl vtc_stub_160
vtc_stub_160:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1280]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1280]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1280]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1280]
    .globl vtc_stub_161
vtc_stub_161:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1288]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1288]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1288]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1288]
    .globl vtc_stub_162
vtc_stub_162:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1296]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1296]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1296]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1296]
    .globl vtc_stub_163
vtc_stub_163:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1304]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1304]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1304]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1304]
    .globl vtc_stub_164
vtc_stub_164:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1312]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1312]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1312]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1312]
    .globl vtc_stub_165
vtc_stub_165:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1320]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1320]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1320]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1320]
    .globl vtc_stub_166
vtc_stub_166:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1328]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1328]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1328]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1328]
    .globl vtc_stub_167
vtc_stub_167:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1336]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1336]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1336]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1336]
    .globl vtc_stub_168
vtc_stub_168:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1344]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1344]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1344]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1344]
    .globl vtc_stub_169
vtc_stub_169:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1352]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1352]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1352]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1352]
    .globl vtc_stub_170
vtc_stub_170:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1360]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1360]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1360]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1360]
    .globl vtc_stub_171
vtc_stub_171:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1368]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1368]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1368]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1368]
    .globl vtc_stub_172
vtc_stub_172:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1376]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1376]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1376]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1376]
    .globl vtc_stub_173
vtc_stub_173:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1384]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1384]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1384]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1384]
    .globl vtc_stub_174
vtc_stub_174:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1392]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1392]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1392]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1392]
    .globl vtc_stub_175
vtc_stub_175:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1400]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1400]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1400]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1400]
    .globl vtc_stub_176
vtc_stub_176:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1408]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1408]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1408]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1408]
    .globl vtc_stub_177
vtc_stub_177:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1416]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1416]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1416]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1416]
    .globl vtc_stub_178
vtc_stub_178:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1424]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1424]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1424]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1424]
    .globl vtc_stub_179
vtc_stub_179:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1432]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1432]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1432]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1432]
    .globl vtc_stub_180
vtc_stub_180:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1440]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1440]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1440]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1440]
    .globl vtc_stub_181
vtc_stub_181:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1448]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1448]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1448]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1448]
    .globl vtc_stub_182
vtc_stub_182:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1456]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1456]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1456]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1456]
    .globl vtc_stub_183
vtc_stub_183:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1464]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1464]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1464]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1464]
    .globl vtc_stub_184
vtc_stub_184:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1472]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1472]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1472]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1472]
    .globl vtc_stub_185
vtc_stub_185:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1480]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1480]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1480]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1480]
    .globl vtc_stub_186
vtc_stub_186:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1488]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1488]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1488]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1488]
    .globl vtc_stub_187
vtc_stub_187:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1496]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1496]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1496]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1496]
    .globl vtc_stub_188
vtc_stub_188:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1504]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1504]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1504]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1504]
    .globl vtc_stub_189
vtc_stub_189:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1512]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1512]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1512]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1512]
    .globl vtc_stub_190
vtc_stub_190:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1520]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1520]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1520]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1520]
    .globl vtc_stub_191
vtc_stub_191:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1528]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1528]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1528]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1528]
    .globl vtc_stub_192
vtc_stub_192:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1536]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1536]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1536]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1536]
    .globl vtc_stub_193
vtc_stub_193:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1544]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1544]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1544]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1544]
    .globl vtc_stub_194
vtc_stub_194:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1552]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1552]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1552]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1552]
    .globl vtc_stub_195
vtc_stub_195:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1560]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1560]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1560]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1560]
    .globl vtc_stub_196
vtc_stub_196:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1568]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1568]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1568]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1568]
    .globl vtc_stub_197
vtc_stub_197:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1576]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1576]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1576]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1576]
    .globl vtc_stub_198
vtc_stub_198:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1584]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1584]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1584]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1584]
    .globl vtc_stub_199
vtc_stub_199:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1592]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1592]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1592]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1592]
    .globl vtc_stub_200
vtc_stub_200:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1600]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1600]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1600]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1600]
    .globl vtc_stub_201
vtc_stub_201:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1608]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1608]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1608]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1608]
    .globl vtc_stub_202
vtc_stub_202:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1616]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1616]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1616]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1616]
    .globl vtc_stub_203
vtc_stub_203:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1624]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1624]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1624]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1624]
    .globl vtc_stub_204
vtc_stub_204:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1632]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1632]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1632]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1632]
    .globl vtc_stub_205
vtc_stub_205:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1640]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1640]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1640]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1640]
    .globl vtc_stub_206
vtc_stub_206:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1648]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1648]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1648]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1648]
    .globl vtc_stub_207
vtc_stub_207:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1656]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1656]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1656]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1656]
    .globl vtc_stub_208
vtc_stub_208:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1664]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1664]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1664]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1664]
    .globl vtc_stub_209
vtc_stub_209:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1672]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1672]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1672]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1672]
    .globl vtc_stub_210
vtc_stub_210:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1680]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1680]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1680]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1680]
    .globl vtc_stub_211
vtc_stub_211:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1688]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1688]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1688]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1688]
    .globl vtc_stub_212
vtc_stub_212:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1696]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1696]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1696]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1696]
    .globl vtc_stub_213
vtc_stub_213:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1704]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1704]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1704]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1704]
    .globl vtc_stub_214
vtc_stub_214:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1712]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1712]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1712]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1712]
    .globl vtc_stub_215
vtc_stub_215:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1720]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1720]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1720]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1720]
    .globl vtc_stub_216
vtc_stub_216:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1728]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1728]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1728]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1728]
    .globl vtc_stub_217
vtc_stub_217:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1736]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1736]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1736]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1736]
    .globl vtc_stub_218
vtc_stub_218:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1744]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1744]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1744]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1744]
    .globl vtc_stub_219
vtc_stub_219:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1752]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1752]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1752]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1752]
    .globl vtc_stub_220
vtc_stub_220:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1760]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1760]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1760]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1760]
    .globl vtc_stub_221
vtc_stub_221:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1768]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1768]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1768]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1768]
    .globl vtc_stub_222
vtc_stub_222:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1776]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1776]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1776]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1776]
    .globl vtc_stub_223
vtc_stub_223:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1784]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1784]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1784]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1784]
    .globl vtc_stub_224
vtc_stub_224:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1792]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1792]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1792]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1792]
    .globl vtc_stub_225
vtc_stub_225:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1800]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1800]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1800]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1800]
    .globl vtc_stub_226
vtc_stub_226:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1808]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1808]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1808]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1808]
    .globl vtc_stub_227
vtc_stub_227:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1816]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1816]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1816]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1816]
    .globl vtc_stub_228
vtc_stub_228:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1824]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1824]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1824]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1824]
    .globl vtc_stub_229
vtc_stub_229:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1832]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1832]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1832]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1832]
    .globl vtc_stub_230
vtc_stub_230:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1840]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1840]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1840]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1840]
    .globl vtc_stub_231
vtc_stub_231:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1848]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1848]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1848]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1848]
    .globl vtc_stub_232
vtc_stub_232:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1856]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1856]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1856]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1856]
    .globl vtc_stub_233
vtc_stub_233:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1864]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1864]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1864]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1864]
    .globl vtc_stub_234
vtc_stub_234:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1872]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1872]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1872]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1872]
    .globl vtc_stub_235
vtc_stub_235:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1880]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1880]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1880]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1880]
    .globl vtc_stub_236
vtc_stub_236:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1888]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1888]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1888]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1888]
    .globl vtc_stub_237
vtc_stub_237:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1896]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1896]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1896]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1896]
    .globl vtc_stub_238
vtc_stub_238:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1904]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1904]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1904]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1904]
    .globl vtc_stub_239
vtc_stub_239:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1912]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1912]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1912]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1912]
    .globl vtc_stub_240
vtc_stub_240:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1920]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1920]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1920]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1920]
    .globl vtc_stub_241
vtc_stub_241:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1928]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1928]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1928]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1928]
    .globl vtc_stub_242
vtc_stub_242:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1936]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1936]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1936]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1936]
    .globl vtc_stub_243
vtc_stub_243:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1944]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1944]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1944]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1944]
    .globl vtc_stub_244
vtc_stub_244:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1952]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1952]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1952]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1952]
    .globl vtc_stub_245
vtc_stub_245:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1960]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1960]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1960]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1960]
    .globl vtc_stub_246
vtc_stub_246:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1968]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1968]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1968]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1968]
    .globl vtc_stub_247
vtc_stub_247:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1976]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1976]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1976]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1976]
    .globl vtc_stub_248
vtc_stub_248:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1984]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1984]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1984]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1984]
    .globl vtc_stub_249
vtc_stub_249:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 1992]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 1992]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 1992]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 1992]
    .globl vtc_stub_250
vtc_stub_250:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 2000]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 2000]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 2000]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 2000]
    .globl vtc_stub_251
vtc_stub_251:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 2008]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 2008]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 2008]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 2008]
    .globl vtc_stub_252
vtc_stub_252:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 2016]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 2016]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 2016]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 2016]
    .globl vtc_stub_253
vtc_stub_253:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 2024]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 2024]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 2024]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 2024]
    .globl vtc_stub_254
vtc_stub_254:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 2032]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 2032]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 2032]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 2032]
    .globl vtc_stub_255
vtc_stub_255:
    push rax
    lock inc qword ptr [rip + VTC_ALL + 2040]
    mov rax, qword ptr [rip + VTC_CLONE]
    cmp rcx, rax
    jne 5f
    lock inc qword ptr [rip + VTC_C + 2040]
5:
    mov rax, qword ptr [rip + VTC_HERO]
    cmp rcx, rax
    jne 6f
    lock inc qword ptr [rip + VTC_H + 2040]
6:
    pop rax
    jmp qword ptr [rip + VTC_ORIG + 2040]
"#);

extern "C" {
    fn vtc_stub_0();
    fn vtc_stub_1();
    fn vtc_stub_2();
    fn vtc_stub_3();
    fn vtc_stub_4();
    fn vtc_stub_5();
    fn vtc_stub_6();
    fn vtc_stub_7();
    fn vtc_stub_8();
    fn vtc_stub_9();
    fn vtc_stub_10();
    fn vtc_stub_11();
    fn vtc_stub_12();
    fn vtc_stub_13();
    fn vtc_stub_14();
    fn vtc_stub_15();
    fn vtc_stub_16();
    fn vtc_stub_17();
    fn vtc_stub_18();
    fn vtc_stub_19();
    fn vtc_stub_20();
    fn vtc_stub_21();
    fn vtc_stub_22();
    fn vtc_stub_23();
    fn vtc_stub_24();
    fn vtc_stub_25();
    fn vtc_stub_26();
    fn vtc_stub_27();
    fn vtc_stub_28();
    fn vtc_stub_29();
    fn vtc_stub_30();
    fn vtc_stub_31();
    fn vtc_stub_32();
    fn vtc_stub_33();
    fn vtc_stub_34();
    fn vtc_stub_35();
    fn vtc_stub_36();
    fn vtc_stub_37();
    fn vtc_stub_38();
    fn vtc_stub_39();
    fn vtc_stub_40();
    fn vtc_stub_41();
    fn vtc_stub_42();
    fn vtc_stub_43();
    fn vtc_stub_44();
    fn vtc_stub_45();
    fn vtc_stub_46();
    fn vtc_stub_47();
    fn vtc_stub_48();
    fn vtc_stub_49();
    fn vtc_stub_50();
    fn vtc_stub_51();
    fn vtc_stub_52();
    fn vtc_stub_53();
    fn vtc_stub_54();
    fn vtc_stub_55();
    fn vtc_stub_56();
    fn vtc_stub_57();
    fn vtc_stub_58();
    fn vtc_stub_59();
    fn vtc_stub_60();
    fn vtc_stub_61();
    fn vtc_stub_62();
    fn vtc_stub_63();
    fn vtc_stub_64();
    fn vtc_stub_65();
    fn vtc_stub_66();
    fn vtc_stub_67();
    fn vtc_stub_68();
    fn vtc_stub_69();
    fn vtc_stub_70();
    fn vtc_stub_71();
    fn vtc_stub_72();
    fn vtc_stub_73();
    fn vtc_stub_74();
    fn vtc_stub_75();
    fn vtc_stub_76();
    fn vtc_stub_77();
    fn vtc_stub_78();
    fn vtc_stub_79();
    fn vtc_stub_80();
    fn vtc_stub_81();
    fn vtc_stub_82();
    fn vtc_stub_83();
    fn vtc_stub_84();
    fn vtc_stub_85();
    fn vtc_stub_86();
    fn vtc_stub_87();
    fn vtc_stub_88();
    fn vtc_stub_89();
    fn vtc_stub_90();
    fn vtc_stub_91();
    fn vtc_stub_92();
    fn vtc_stub_93();
    fn vtc_stub_94();
    fn vtc_stub_95();
    fn vtc_stub_96();
    fn vtc_stub_97();
    fn vtc_stub_98();
    fn vtc_stub_99();
    fn vtc_stub_100();
    fn vtc_stub_101();
    fn vtc_stub_102();
    fn vtc_stub_103();
    fn vtc_stub_104();
    fn vtc_stub_105();
    fn vtc_stub_106();
    fn vtc_stub_107();
    fn vtc_stub_108();
    fn vtc_stub_109();
    fn vtc_stub_110();
    fn vtc_stub_111();
    fn vtc_stub_112();
    fn vtc_stub_113();
    fn vtc_stub_114();
    fn vtc_stub_115();
    fn vtc_stub_116();
    fn vtc_stub_117();
    fn vtc_stub_118();
    fn vtc_stub_119();
    fn vtc_stub_120();
    fn vtc_stub_121();
    fn vtc_stub_122();
    fn vtc_stub_123();
    fn vtc_stub_124();
    fn vtc_stub_125();
    fn vtc_stub_126();
    fn vtc_stub_127();
    fn vtc_stub_128();
    fn vtc_stub_129();
    fn vtc_stub_130();
    fn vtc_stub_131();
    fn vtc_stub_132();
    fn vtc_stub_133();
    fn vtc_stub_134();
    fn vtc_stub_135();
    fn vtc_stub_136();
    fn vtc_stub_137();
    fn vtc_stub_138();
    fn vtc_stub_139();
    fn vtc_stub_140();
    fn vtc_stub_141();
    fn vtc_stub_142();
    fn vtc_stub_143();
    fn vtc_stub_144();
    fn vtc_stub_145();
    fn vtc_stub_146();
    fn vtc_stub_147();
    fn vtc_stub_148();
    fn vtc_stub_149();
    fn vtc_stub_150();
    fn vtc_stub_151();
    fn vtc_stub_152();
    fn vtc_stub_153();
    fn vtc_stub_154();
    fn vtc_stub_155();
    fn vtc_stub_156();
    fn vtc_stub_157();
    fn vtc_stub_158();
    fn vtc_stub_159();
    fn vtc_stub_160();
    fn vtc_stub_161();
    fn vtc_stub_162();
    fn vtc_stub_163();
    fn vtc_stub_164();
    fn vtc_stub_165();
    fn vtc_stub_166();
    fn vtc_stub_167();
    fn vtc_stub_168();
    fn vtc_stub_169();
    fn vtc_stub_170();
    fn vtc_stub_171();
    fn vtc_stub_172();
    fn vtc_stub_173();
    fn vtc_stub_174();
    fn vtc_stub_175();
    fn vtc_stub_176();
    fn vtc_stub_177();
    fn vtc_stub_178();
    fn vtc_stub_179();
    fn vtc_stub_180();
    fn vtc_stub_181();
    fn vtc_stub_182();
    fn vtc_stub_183();
    fn vtc_stub_184();
    fn vtc_stub_185();
    fn vtc_stub_186();
    fn vtc_stub_187();
    fn vtc_stub_188();
    fn vtc_stub_189();
    fn vtc_stub_190();
    fn vtc_stub_191();
    fn vtc_stub_192();
    fn vtc_stub_193();
    fn vtc_stub_194();
    fn vtc_stub_195();
    fn vtc_stub_196();
    fn vtc_stub_197();
    fn vtc_stub_198();
    fn vtc_stub_199();
    fn vtc_stub_200();
    fn vtc_stub_201();
    fn vtc_stub_202();
    fn vtc_stub_203();
    fn vtc_stub_204();
    fn vtc_stub_205();
    fn vtc_stub_206();
    fn vtc_stub_207();
    fn vtc_stub_208();
    fn vtc_stub_209();
    fn vtc_stub_210();
    fn vtc_stub_211();
    fn vtc_stub_212();
    fn vtc_stub_213();
    fn vtc_stub_214();
    fn vtc_stub_215();
    fn vtc_stub_216();
    fn vtc_stub_217();
    fn vtc_stub_218();
    fn vtc_stub_219();
    fn vtc_stub_220();
    fn vtc_stub_221();
    fn vtc_stub_222();
    fn vtc_stub_223();
    fn vtc_stub_224();
    fn vtc_stub_225();
    fn vtc_stub_226();
    fn vtc_stub_227();
    fn vtc_stub_228();
    fn vtc_stub_229();
    fn vtc_stub_230();
    fn vtc_stub_231();
    fn vtc_stub_232();
    fn vtc_stub_233();
    fn vtc_stub_234();
    fn vtc_stub_235();
    fn vtc_stub_236();
    fn vtc_stub_237();
    fn vtc_stub_238();
    fn vtc_stub_239();
    fn vtc_stub_240();
    fn vtc_stub_241();
    fn vtc_stub_242();
    fn vtc_stub_243();
    fn vtc_stub_244();
    fn vtc_stub_245();
    fn vtc_stub_246();
    fn vtc_stub_247();
    fn vtc_stub_248();
    fn vtc_stub_249();
    fn vtc_stub_250();
    fn vtc_stub_251();
    fn vtc_stub_252();
    fn vtc_stub_253();
    fn vtc_stub_254();
    fn vtc_stub_255();
}

pub fn stubs() -> [unsafe extern "C" fn(); N] {
    [
    vtc_stub_0 as unsafe extern "C" fn(),
    vtc_stub_1 as unsafe extern "C" fn(),
    vtc_stub_2 as unsafe extern "C" fn(),
    vtc_stub_3 as unsafe extern "C" fn(),
    vtc_stub_4 as unsafe extern "C" fn(),
    vtc_stub_5 as unsafe extern "C" fn(),
    vtc_stub_6 as unsafe extern "C" fn(),
    vtc_stub_7 as unsafe extern "C" fn(),
    vtc_stub_8 as unsafe extern "C" fn(),
    vtc_stub_9 as unsafe extern "C" fn(),
    vtc_stub_10 as unsafe extern "C" fn(),
    vtc_stub_11 as unsafe extern "C" fn(),
    vtc_stub_12 as unsafe extern "C" fn(),
    vtc_stub_13 as unsafe extern "C" fn(),
    vtc_stub_14 as unsafe extern "C" fn(),
    vtc_stub_15 as unsafe extern "C" fn(),
    vtc_stub_16 as unsafe extern "C" fn(),
    vtc_stub_17 as unsafe extern "C" fn(),
    vtc_stub_18 as unsafe extern "C" fn(),
    vtc_stub_19 as unsafe extern "C" fn(),
    vtc_stub_20 as unsafe extern "C" fn(),
    vtc_stub_21 as unsafe extern "C" fn(),
    vtc_stub_22 as unsafe extern "C" fn(),
    vtc_stub_23 as unsafe extern "C" fn(),
    vtc_stub_24 as unsafe extern "C" fn(),
    vtc_stub_25 as unsafe extern "C" fn(),
    vtc_stub_26 as unsafe extern "C" fn(),
    vtc_stub_27 as unsafe extern "C" fn(),
    vtc_stub_28 as unsafe extern "C" fn(),
    vtc_stub_29 as unsafe extern "C" fn(),
    vtc_stub_30 as unsafe extern "C" fn(),
    vtc_stub_31 as unsafe extern "C" fn(),
    vtc_stub_32 as unsafe extern "C" fn(),
    vtc_stub_33 as unsafe extern "C" fn(),
    vtc_stub_34 as unsafe extern "C" fn(),
    vtc_stub_35 as unsafe extern "C" fn(),
    vtc_stub_36 as unsafe extern "C" fn(),
    vtc_stub_37 as unsafe extern "C" fn(),
    vtc_stub_38 as unsafe extern "C" fn(),
    vtc_stub_39 as unsafe extern "C" fn(),
    vtc_stub_40 as unsafe extern "C" fn(),
    vtc_stub_41 as unsafe extern "C" fn(),
    vtc_stub_42 as unsafe extern "C" fn(),
    vtc_stub_43 as unsafe extern "C" fn(),
    vtc_stub_44 as unsafe extern "C" fn(),
    vtc_stub_45 as unsafe extern "C" fn(),
    vtc_stub_46 as unsafe extern "C" fn(),
    vtc_stub_47 as unsafe extern "C" fn(),
    vtc_stub_48 as unsafe extern "C" fn(),
    vtc_stub_49 as unsafe extern "C" fn(),
    vtc_stub_50 as unsafe extern "C" fn(),
    vtc_stub_51 as unsafe extern "C" fn(),
    vtc_stub_52 as unsafe extern "C" fn(),
    vtc_stub_53 as unsafe extern "C" fn(),
    vtc_stub_54 as unsafe extern "C" fn(),
    vtc_stub_55 as unsafe extern "C" fn(),
    vtc_stub_56 as unsafe extern "C" fn(),
    vtc_stub_57 as unsafe extern "C" fn(),
    vtc_stub_58 as unsafe extern "C" fn(),
    vtc_stub_59 as unsafe extern "C" fn(),
    vtc_stub_60 as unsafe extern "C" fn(),
    vtc_stub_61 as unsafe extern "C" fn(),
    vtc_stub_62 as unsafe extern "C" fn(),
    vtc_stub_63 as unsafe extern "C" fn(),
    vtc_stub_64 as unsafe extern "C" fn(),
    vtc_stub_65 as unsafe extern "C" fn(),
    vtc_stub_66 as unsafe extern "C" fn(),
    vtc_stub_67 as unsafe extern "C" fn(),
    vtc_stub_68 as unsafe extern "C" fn(),
    vtc_stub_69 as unsafe extern "C" fn(),
    vtc_stub_70 as unsafe extern "C" fn(),
    vtc_stub_71 as unsafe extern "C" fn(),
    vtc_stub_72 as unsafe extern "C" fn(),
    vtc_stub_73 as unsafe extern "C" fn(),
    vtc_stub_74 as unsafe extern "C" fn(),
    vtc_stub_75 as unsafe extern "C" fn(),
    vtc_stub_76 as unsafe extern "C" fn(),
    vtc_stub_77 as unsafe extern "C" fn(),
    vtc_stub_78 as unsafe extern "C" fn(),
    vtc_stub_79 as unsafe extern "C" fn(),
    vtc_stub_80 as unsafe extern "C" fn(),
    vtc_stub_81 as unsafe extern "C" fn(),
    vtc_stub_82 as unsafe extern "C" fn(),
    vtc_stub_83 as unsafe extern "C" fn(),
    vtc_stub_84 as unsafe extern "C" fn(),
    vtc_stub_85 as unsafe extern "C" fn(),
    vtc_stub_86 as unsafe extern "C" fn(),
    vtc_stub_87 as unsafe extern "C" fn(),
    vtc_stub_88 as unsafe extern "C" fn(),
    vtc_stub_89 as unsafe extern "C" fn(),
    vtc_stub_90 as unsafe extern "C" fn(),
    vtc_stub_91 as unsafe extern "C" fn(),
    vtc_stub_92 as unsafe extern "C" fn(),
    vtc_stub_93 as unsafe extern "C" fn(),
    vtc_stub_94 as unsafe extern "C" fn(),
    vtc_stub_95 as unsafe extern "C" fn(),
    vtc_stub_96 as unsafe extern "C" fn(),
    vtc_stub_97 as unsafe extern "C" fn(),
    vtc_stub_98 as unsafe extern "C" fn(),
    vtc_stub_99 as unsafe extern "C" fn(),
    vtc_stub_100 as unsafe extern "C" fn(),
    vtc_stub_101 as unsafe extern "C" fn(),
    vtc_stub_102 as unsafe extern "C" fn(),
    vtc_stub_103 as unsafe extern "C" fn(),
    vtc_stub_104 as unsafe extern "C" fn(),
    vtc_stub_105 as unsafe extern "C" fn(),
    vtc_stub_106 as unsafe extern "C" fn(),
    vtc_stub_107 as unsafe extern "C" fn(),
    vtc_stub_108 as unsafe extern "C" fn(),
    vtc_stub_109 as unsafe extern "C" fn(),
    vtc_stub_110 as unsafe extern "C" fn(),
    vtc_stub_111 as unsafe extern "C" fn(),
    vtc_stub_112 as unsafe extern "C" fn(),
    vtc_stub_113 as unsafe extern "C" fn(),
    vtc_stub_114 as unsafe extern "C" fn(),
    vtc_stub_115 as unsafe extern "C" fn(),
    vtc_stub_116 as unsafe extern "C" fn(),
    vtc_stub_117 as unsafe extern "C" fn(),
    vtc_stub_118 as unsafe extern "C" fn(),
    vtc_stub_119 as unsafe extern "C" fn(),
    vtc_stub_120 as unsafe extern "C" fn(),
    vtc_stub_121 as unsafe extern "C" fn(),
    vtc_stub_122 as unsafe extern "C" fn(),
    vtc_stub_123 as unsafe extern "C" fn(),
    vtc_stub_124 as unsafe extern "C" fn(),
    vtc_stub_125 as unsafe extern "C" fn(),
    vtc_stub_126 as unsafe extern "C" fn(),
    vtc_stub_127 as unsafe extern "C" fn(),
    vtc_stub_128 as unsafe extern "C" fn(),
    vtc_stub_129 as unsafe extern "C" fn(),
    vtc_stub_130 as unsafe extern "C" fn(),
    vtc_stub_131 as unsafe extern "C" fn(),
    vtc_stub_132 as unsafe extern "C" fn(),
    vtc_stub_133 as unsafe extern "C" fn(),
    vtc_stub_134 as unsafe extern "C" fn(),
    vtc_stub_135 as unsafe extern "C" fn(),
    vtc_stub_136 as unsafe extern "C" fn(),
    vtc_stub_137 as unsafe extern "C" fn(),
    vtc_stub_138 as unsafe extern "C" fn(),
    vtc_stub_139 as unsafe extern "C" fn(),
    vtc_stub_140 as unsafe extern "C" fn(),
    vtc_stub_141 as unsafe extern "C" fn(),
    vtc_stub_142 as unsafe extern "C" fn(),
    vtc_stub_143 as unsafe extern "C" fn(),
    vtc_stub_144 as unsafe extern "C" fn(),
    vtc_stub_145 as unsafe extern "C" fn(),
    vtc_stub_146 as unsafe extern "C" fn(),
    vtc_stub_147 as unsafe extern "C" fn(),
    vtc_stub_148 as unsafe extern "C" fn(),
    vtc_stub_149 as unsafe extern "C" fn(),
    vtc_stub_150 as unsafe extern "C" fn(),
    vtc_stub_151 as unsafe extern "C" fn(),
    vtc_stub_152 as unsafe extern "C" fn(),
    vtc_stub_153 as unsafe extern "C" fn(),
    vtc_stub_154 as unsafe extern "C" fn(),
    vtc_stub_155 as unsafe extern "C" fn(),
    vtc_stub_156 as unsafe extern "C" fn(),
    vtc_stub_157 as unsafe extern "C" fn(),
    vtc_stub_158 as unsafe extern "C" fn(),
    vtc_stub_159 as unsafe extern "C" fn(),
    vtc_stub_160 as unsafe extern "C" fn(),
    vtc_stub_161 as unsafe extern "C" fn(),
    vtc_stub_162 as unsafe extern "C" fn(),
    vtc_stub_163 as unsafe extern "C" fn(),
    vtc_stub_164 as unsafe extern "C" fn(),
    vtc_stub_165 as unsafe extern "C" fn(),
    vtc_stub_166 as unsafe extern "C" fn(),
    vtc_stub_167 as unsafe extern "C" fn(),
    vtc_stub_168 as unsafe extern "C" fn(),
    vtc_stub_169 as unsafe extern "C" fn(),
    vtc_stub_170 as unsafe extern "C" fn(),
    vtc_stub_171 as unsafe extern "C" fn(),
    vtc_stub_172 as unsafe extern "C" fn(),
    vtc_stub_173 as unsafe extern "C" fn(),
    vtc_stub_174 as unsafe extern "C" fn(),
    vtc_stub_175 as unsafe extern "C" fn(),
    vtc_stub_176 as unsafe extern "C" fn(),
    vtc_stub_177 as unsafe extern "C" fn(),
    vtc_stub_178 as unsafe extern "C" fn(),
    vtc_stub_179 as unsafe extern "C" fn(),
    vtc_stub_180 as unsafe extern "C" fn(),
    vtc_stub_181 as unsafe extern "C" fn(),
    vtc_stub_182 as unsafe extern "C" fn(),
    vtc_stub_183 as unsafe extern "C" fn(),
    vtc_stub_184 as unsafe extern "C" fn(),
    vtc_stub_185 as unsafe extern "C" fn(),
    vtc_stub_186 as unsafe extern "C" fn(),
    vtc_stub_187 as unsafe extern "C" fn(),
    vtc_stub_188 as unsafe extern "C" fn(),
    vtc_stub_189 as unsafe extern "C" fn(),
    vtc_stub_190 as unsafe extern "C" fn(),
    vtc_stub_191 as unsafe extern "C" fn(),
    vtc_stub_192 as unsafe extern "C" fn(),
    vtc_stub_193 as unsafe extern "C" fn(),
    vtc_stub_194 as unsafe extern "C" fn(),
    vtc_stub_195 as unsafe extern "C" fn(),
    vtc_stub_196 as unsafe extern "C" fn(),
    vtc_stub_197 as unsafe extern "C" fn(),
    vtc_stub_198 as unsafe extern "C" fn(),
    vtc_stub_199 as unsafe extern "C" fn(),
    vtc_stub_200 as unsafe extern "C" fn(),
    vtc_stub_201 as unsafe extern "C" fn(),
    vtc_stub_202 as unsafe extern "C" fn(),
    vtc_stub_203 as unsafe extern "C" fn(),
    vtc_stub_204 as unsafe extern "C" fn(),
    vtc_stub_205 as unsafe extern "C" fn(),
    vtc_stub_206 as unsafe extern "C" fn(),
    vtc_stub_207 as unsafe extern "C" fn(),
    vtc_stub_208 as unsafe extern "C" fn(),
    vtc_stub_209 as unsafe extern "C" fn(),
    vtc_stub_210 as unsafe extern "C" fn(),
    vtc_stub_211 as unsafe extern "C" fn(),
    vtc_stub_212 as unsafe extern "C" fn(),
    vtc_stub_213 as unsafe extern "C" fn(),
    vtc_stub_214 as unsafe extern "C" fn(),
    vtc_stub_215 as unsafe extern "C" fn(),
    vtc_stub_216 as unsafe extern "C" fn(),
    vtc_stub_217 as unsafe extern "C" fn(),
    vtc_stub_218 as unsafe extern "C" fn(),
    vtc_stub_219 as unsafe extern "C" fn(),
    vtc_stub_220 as unsafe extern "C" fn(),
    vtc_stub_221 as unsafe extern "C" fn(),
    vtc_stub_222 as unsafe extern "C" fn(),
    vtc_stub_223 as unsafe extern "C" fn(),
    vtc_stub_224 as unsafe extern "C" fn(),
    vtc_stub_225 as unsafe extern "C" fn(),
    vtc_stub_226 as unsafe extern "C" fn(),
    vtc_stub_227 as unsafe extern "C" fn(),
    vtc_stub_228 as unsafe extern "C" fn(),
    vtc_stub_229 as unsafe extern "C" fn(),
    vtc_stub_230 as unsafe extern "C" fn(),
    vtc_stub_231 as unsafe extern "C" fn(),
    vtc_stub_232 as unsafe extern "C" fn(),
    vtc_stub_233 as unsafe extern "C" fn(),
    vtc_stub_234 as unsafe extern "C" fn(),
    vtc_stub_235 as unsafe extern "C" fn(),
    vtc_stub_236 as unsafe extern "C" fn(),
    vtc_stub_237 as unsafe extern "C" fn(),
    vtc_stub_238 as unsafe extern "C" fn(),
    vtc_stub_239 as unsafe extern "C" fn(),
    vtc_stub_240 as unsafe extern "C" fn(),
    vtc_stub_241 as unsafe extern "C" fn(),
    vtc_stub_242 as unsafe extern "C" fn(),
    vtc_stub_243 as unsafe extern "C" fn(),
    vtc_stub_244 as unsafe extern "C" fn(),
    vtc_stub_245 as unsafe extern "C" fn(),
    vtc_stub_246 as unsafe extern "C" fn(),
    vtc_stub_247 as unsafe extern "C" fn(),
    vtc_stub_248 as unsafe extern "C" fn(),
    vtc_stub_249 as unsafe extern "C" fn(),
    vtc_stub_250 as unsafe extern "C" fn(),
    vtc_stub_251 as unsafe extern "C" fn(),
    vtc_stub_252 as unsafe extern "C" fn(),
    vtc_stub_253 as unsafe extern "C" fn(),
    vtc_stub_254 as unsafe extern "C" fn(),
    vtc_stub_255 as unsafe extern "C" fn()
    ]
}
