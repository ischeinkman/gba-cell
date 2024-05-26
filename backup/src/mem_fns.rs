//! Module for direct memory operations.
//!
//! Generally you don't need to call these yourself. Instead, the compiler will
//! insert calls to the functions defined here as necessary.

use core::ffi::c_void;

/// `u16` copy between exclusive regions.
///
/// * If the `byte_count` is odd then a single byte copy will happen at the end.
///
/// ## Safety
/// * If `byte_count` is zero then the pointers are not used and they can be any
///   value.
/// * If `byte_count` is non-zero then:
///   * Both pointers must be valid for the span used and aligned to 2.
///   * The two regions must either be *entirely* disjoint or *entirely*
///     overlapping. Partial overlap is not allowed.
#[inline]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.__aeabi_memcpy2"]
pub unsafe extern "C" fn __aeabi_memcpy2(
  mut dest: *mut u16, mut src: *const u16, mut byte_count: usize,
) {
  core::arch::asm! {
    "1:",
    "subs    {count}, {count}, #2",
    "ldrhge  {temp}, [{src}], #2",
    "strhge  {temp}, [{dest}], #2",
    "bgt     1b",
    temp = out(reg) _,
    count = inout(reg) byte_count,
    src = inout(reg) src,
    dest = inout(reg) dest,
    options(nostack)
  }
  if byte_count != 0 {
    let dest = dest.cast::<u8>();
    let src = src.cast::<u8>();
    dest.write_volatile(src.read_volatile());
  }
}

/// Word copy between exclusive regions.
///
/// * If `byte_count` is not a multiple of 4 then a halfword and/or byte copy
///   will happen at the end.
///
/// ## Safety
/// * If `byte_count` is zero then the pointers are not used and they can be any
///   value.
/// * If `byte_count` is non-zero then:
///   * Both pointers must be valid for the span used and aligned to 4.
///   * The two regions must either be *entirely* disjoint or *entirely*
///     overlapping. Partial overlap is not allowed.
#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.__aeabi_memcpy4"]
pub unsafe extern "C" fn __aeabi_memcpy4(
  dest: *mut u32, src: *const u32, byte_count: usize,
) {
  core::arch::asm! {
    bracer::when!( "r2" >=u "#32" [label_id=2] {
      bracer::with_pushed_registers!("{{r4-r9}}", {
        "1:",
        "subs   r2, r2, #32",
        "ldmge  r1!, {{r3-r9, r12}}",
        "stmge  r0!, {{r3-r9, r12}}",
        "bgt    1b",
      }),
      "bxeq   lr",
    }),

    // copy 4 words, two at a time
    "tst    r2, #0b10000",
    "ldmne  r1!, {{r3, r12}}",
    "stmne  r0!, {{r3, r12}}",
    "ldmne  r1!, {{r3, r12}}",
    "stmne  r0!, {{r3, r12}}",
    "bics   r2, r2, #0b10000",
    "bxeq   lr",

    // copy 2 and/or 1 words
    "lsls   r3, r2, #29",
    "ldmcs  r1!, {{r3, r12}}",
    "stmcs  r0!, {{r3, r12}}",
    "ldrmi  r3, [r1], #4",
    "strmi  r3, [r0], #4",
    "bics   r2, r2, #0b1100",
    "bxeq   lr",

    // copy halfword and/or byte
    "lsls   r3, r2, #31",
    "ldrhcs r3, [r1], #2",
    "strhcs r3, [r0], #2",
    "ldrbmi r3, [r1], #1",
    "strbmi r3, [r0], #1",
    "bx     lr",
    options(noreturn),
  }
}

/// Just call [`__aeabi_memcpy4`] instead.
///
/// This function is provided only for API completeness, because in some cases
/// the compiler might automatically generate a call to this function.
#[inline]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.__aeabi_memcpy8"]
pub unsafe extern "C" fn __aeabi_memcpy8(
  dest: *mut u32, src: *const u32, byte_count: usize,
) {
  __aeabi_memcpy4(dest, src, byte_count);
}

/// Arbitrary-width copy between exclusive regions.
///
/// ## Safety
/// * If `byte_count` is zero then the pointers are not used and they can be any
///   value.
/// * If `byte_count` is non-zero then:
///   * Both pointers must be valid for the span used (no required alignment).
///   * The two regions must either be *entirely* disjoint or *entirely*
///     overlapping. Partial overlap is not allowed.
#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.__aeabi_memcpy"]
pub unsafe extern "C" fn __aeabi_memcpy(
  dest: *mut u8, src: *const u8, byte_count: usize,
) {
  core::arch::asm! {
    "cmp    r2, #7", // if count <= (fix+word): just byte copy
    "ble    {__aeabi_memcpy1}",

    // check max coalign
    "eor    r3, r0, r1",
    "lsls   r3, r3, #31",
    "bmi    {__aeabi_memcpy1}",
    "bcs    2f",

    // max coalign4, possible fixup and jump
    "lsls   r3, r0, #31",
    "submi  r2, r2, #1",
    "ldrbmi r3, [r1], #1",
    "strbmi r3, [r0], #1",
    "subcs  r2, r2, #2",
    "ldrhcs r3, [r1], #2",
    "strhcs r3, [r0], #2",
    "b      {__aeabi_memcpy4}",

    // max coalign2, possible fixup and jump
    "2:",
    "lsls   r3, r0, #31",
    "submi  r2, r2, #1",
    "ldrbmi r3, [r1], #1",
    "strbmi r3, [r0], #1",
    "b      {__aeabi_memcpy2}",

    //
    __aeabi_memcpy4 = sym __aeabi_memcpy4,
    __aeabi_memcpy2 = sym __aeabi_memcpy2,
    __aeabi_memcpy1 = sym __aeabi_memcpy1,
    options(noreturn)
  }
}

/// Copy between exclusive regions, prefer [`__aeabi_memcpy`] if possible.
///
/// This is the libc version of a memory copy. It's required to return the
/// `dest` pointer at the end of the call, which makes it need an extra
/// push/pop compared to a direct call to `__aeabi_memcpy`.
///
/// * **Returns:** The `dest` pointer.
#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.memcpy"]
pub unsafe extern "C" fn memcpy(
  dest: *mut u8, src: *const u8, byte_count: usize,
) -> *mut u8 {
  // I've seen a standard call to `__aeabi_memcpy` give weird codegen,
  // so we (currently) do the call manually.
  core::arch::asm! {
    bracer::with_pushed_registers!("{{r0, lr}}", {
      "bl {__aeabi_memcpy}",
    }),
    "bx lr",
    __aeabi_memcpy = sym __aeabi_memcpy,
    options(noreturn)
  }
}

// MOVE

// used by `__aeabi_memmove` in some cases
#[inline]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.reverse_copy_u8"]
unsafe extern "C" fn reverse_copy_u8(
  dest: *mut u8, src: *const u8, byte_count: usize,
) {
  core::arch::asm! {
    "1:",
    "subs    {count}, {count}, #1",
    "ldrbge  {temp}, [{src}, #-1]!",
    "strbge  {temp}, [{dest}, #-1]!",
    "bgt     1b",
    temp = out(reg) _,
    count = inout(reg) byte_count => _,
    src = inout(reg) src => _,
    dest = inout(reg) dest => _,
    options(nostack)
  }
}

// used by `__aeabi_memmove` in some cases
#[inline]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.reverse_copy_u16"]
unsafe extern "C" fn reverse_copy_u16(
  mut dest: *mut u16, mut src: *const u16, mut byte_count: usize,
) {
  core::arch::asm! {
    "1:",
    "subs    {count}, {count}, #2",
    "ldrhge  {temp}, [{src}, #-2]!",
    "strhge  {temp}, [{dest}, #-2]!",
    "bgt     1b",
    temp = out(reg) _,
    count = inout(reg) byte_count,
    src = inout(reg) src,
    dest = inout(reg) dest,
    options(nostack)
  }
  if byte_count != 0 {
    let dest = dest.cast::<u8>().sub(1);
    let src = src.cast::<u8>().sub(1);
    dest.write_volatile(src.read_volatile());
  }
}

// used by `__aeabi_memmove` in some cases
#[naked]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.reverse_copy_u32"]
unsafe extern "C" fn reverse_copy_u32(
  dest: *mut u32, src: *const u32, byte_count: usize,
) {
  core::arch::asm! {
    bracer::when!( "r2" >=u "#32" [label_id=2] {
      bracer::with_pushed_registers!("{{r4-r9}}", {
        "1:",
        "subs    r2, r2, #32",
        "ldmdbcs r1!, {{r3-r9, r12}}",
        "stmdbcs r0!, {{r3-r9, r12}}",
        "bgt     1b",
      }),
      "bxeq   lr",
    }),

    // copy 4 words, two at a time
    "tst     r2, #0b10000",
    "ldmdbne r1!, {{r3, r12}}",
    "stmdbne r0!, {{r3, r12}}",
    "ldmdbne r1!, {{r3, r12}}",
    "stmdbne r0!, {{r3, r12}}",
    "bics    r2, r2, #0b10000",
    "bxeq    lr",

    // copy 2 and/or 1 words
    "lsls    r3, r2, #29",
    "ldmdbcs r1!, {{r3, r12}}",
    "stmdbcs r0!, {{r3, r12}}",
    "ldrmi   r3, [r1, #-4]!",
    "strmi   r3, [r0, #-4]!",
    "bxeq    lr",

    // copy halfword and/or byte
    "lsls    r2, r2, #31",
    "ldrhcs  r3, [r1, #-2]!",
    "strhcs  r3, [r0, #-2]!",
    "ldrbmi  r3, [r1, #-1]!",
    "strbmi  r3, [r0, #-1]!",
    "bx      lr",
    options(noreturn),
  }
}

/// Copy between non-exclusive regions, prefer [`__aeabi_memmove`] if possible.
///
/// This function is provided only for API completeness, because in some cases
/// the compiler might automatically generate a call to this function.
#[inline]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.__aeabi_memmove4"]
pub unsafe extern "C" fn __aeabi_memmove4(
  dest: *mut u32, src: *const u32, byte_count: usize,
) {
  __aeabi_memmove(dest.cast(), src.cast(), byte_count)
}

/// Copy between non-exclusive regions, prefer [`__aeabi_memmove`] if possible.
///
/// This function is provided only for API completeness, because in some cases
/// the compiler might automatically generate a call to this function.
#[inline]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.__aeabi_memmove8"]
pub unsafe extern "C" fn __aeabi_memmove8(
  dest: *mut u32, src: *const u32, byte_count: usize,
) {
  __aeabi_memmove(dest.cast(), src.cast(), byte_count)
}

/// Copy between non-exclusive regions.
///
/// * The pointers do not have a minimum alignment. The function will
///   automatically detect the best type of copy to perform.
#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.__aeabi_memmove"]
pub unsafe extern "C" fn __aeabi_memmove(
  dest: *mut u8, src: *const u8, byte_count: usize,
) {
  core::arch::asm! {
    // when d > s we need to copy back-to-front
    bracer::when!("r0" >=u "r1" [label_id=1] {
      "add     r0, r0, r2",
      "add     r1, r1, r2",
      "eor     r3, r0, r1",
      "lsls    r3, r3, #31",
      "bmi     {reverse_copy_u8}",
      "bcs     2f",

      // max coalign4, possible fixup and jump
      "lsls    r3, r0, #31",
      "submi   r2, r2, #1",
      "ldrbmi  r3, [r1, #-1]!",
      "strbmi  r3, [r0, #-1]!",
      "subcs   r2, r2, #2",
      "ldrhcs  r3, [r1, #-2]!",
      "strhcs  r3, [r0, #-2]!",
      "b       {reverse_copy_u32}",

      // max coalign2, possible fixup and jump
      "2:",
      "tst     r0, #1",
      "sub     r2, r2, #1",
      "ldrb    r3, [r1, #-1]!",
      "strb    r3, [r0, #-1]!",
      "b       {reverse_copy_u16}",
    }),
    // forward copy is a normal memcpy
    "b      {__aeabi_memcpy}",
    __aeabi_memcpy = sym __aeabi_memcpy,
    reverse_copy_u8 = sym reverse_copy_u8,
    reverse_copy_u16 = sym reverse_copy_u16,
    reverse_copy_u32 = sym reverse_copy_u32,
    options(noreturn),
  }
}

/// Copy between non-exclusive regions, prefer [`__aeabi_memmove`] if possible.
///
/// This is the libc version of a memory move. It's required to return the
/// `dest` pointer at the end of the call, which makes it need an extra
/// push/pop compared to a direct call to `__aeabi_memmove`.
///
/// * **Returns:** The `dest` pointer.
#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.memmove"]
pub unsafe extern "C" fn memmove(
  dest: *mut u8, src: *const u8, byte_count: usize,
) -> *mut u8 {
  core::arch::asm! {
    bracer::with_pushed_registers!("{{r0, lr}}", {
      "bl {__aeabi_memmove}",
    }),
    "bx lr",
    __aeabi_memmove = sym __aeabi_memmove,
    options(noreturn)
  }
}
