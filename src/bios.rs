#![allow(non_snake_case)]

//! Module for calls to BIOS functions.
//!
//! The BIOS functions aren't called using a normal foreign function call (eg:
//! using the `extern "C"` ABI). Instead, a special instruction `swi`
//! (software-interrupt) is executed, and an immediate data byte in the
//! instruction tells the BIOS what function to execute. Because of this, the
//! BIOS functions have a rather high calls overhead compared to a normal
//! foreign function.

use crate::IrqBits;

/// `0x04`: Waits for a specific interrupt type(s) to happen.
///
/// Pauses the CPU until any of the interrupt types set in `target_irqs` to
/// occur. This can create a significant savings of the battery while you're
/// waiting, so use this function when possible.
///
/// **Important:** This function forces [`IME`](crate::mmio::IME) on.
///
/// Your interrupt handler (if any) will be run before this function returns.
///
/// If none of the interrupts specified in `target_irqs` are properly configured
/// to fire then this function will loop forever without returning.
///
/// This function uses a special BIOS variable to track what interrupts have
/// occured recently.
/// * If `ignore_existing` is set, then any previous interrupts (since
///   `IntrWait` was last called) that match `target_irqs` are *ignored* and
///   this function will wait for a new target interrupt to occur.
/// * Otherwise, any previous interrupts that match `target_irqs` will cause the
///   function to return immediately without waiting for a new interrupt.
#[inline]
#[cfg_attr(feature = "on_gba", instruction_set(arm::t32))]
pub fn IntrWait(ignore_existing: bool, target_irqs: IrqBits) {
  on_gba_or_unimplemented!(
    unsafe {
      core::arch::asm! {
        "swi #0x04",
        inout("r0") ignore_existing as u32 => _,
        inout("r1") target_irqs.0 => _,
        out("r3") _,
        options(preserves_flags),
      }
    };
  );
}

/// `0x05`: Builtin shorthand for [`IntrWait(true, IrqBits::VBLANK)`](IntrWait)
#[inline]
#[cfg_attr(feature = "on_gba", instruction_set(arm::t32))]
pub fn VBlankIntrWait() {
  on_gba_or_unimplemented!(
    unsafe {
      core::arch::asm! {
        "swi #0x05",
        out("r0") _,
        out("r1") _,
        out("r3") _,
        options(preserves_flags),
      }
    };
  );
}
