# Changelog

* **0.11.6:**
  * `on_gba` feature (default: enabled) that signals if the crate is running on a GBA.
    Limited portions of the crate *can* be used even when not on the GBA (such as in a build script).
  * `track_caller` added for fixed point math operations
* **0.11.5:**
  * Fixed the random number generator `next` method (https://github.com/rust-console/gba/issues/192).
  * Added optional support for the `critical-section` crate (https://github.com/rust-console/gba/pull/191)
  * Put new guidance on effective `build-std` settings in the README (https://github.com/rust-console/gba/issues/187)
  * The `mmio` module now publicly re-exports the necessary definitions used from other crates (https://github.com/rust-console/gba/issues/173)

* **0.11.3:**
  * **Soundness:** Fixed the definition of `VIDEO3_VRAM`, now it's the correct
  type and it will correctly stay in bounds.

* **0.11.1:**
  * Fixed incorrect timer 1/2/3 addresses (classic copy-paste error).
  * Fixed missing argument ordering swap in memset between the libc version and
    aeabi version.
  * Fixed incorrect handling of unaligned pointers in memset.
  * Added `.shstrtab` entry to the linker script so that it works with the new
    `rust-lld` used in Nightly 2023-01-01 (See
    https://github.com/rust-lang/rust/pull/109721)
  * Marked the linker script file (`mono_boot.ld`) as simply being
    [CC0](https://creativecommons.org/publicdomain/zero/1.0/legalcode). The
    reason being that it's the one file that often needs to be manually copied
    out of this repo and into the repo of a project using the crate (since
    libraries can't provide the linker script to binaries, petition the `cargo`
    folks if you want this). Accordingly, I'm giving it the most permissive
    license that I know of, to make things as simple as possible for everyone.
    The rest of the content in this repository still falls under the `license`
    entry specified in `Cargo.toml`.

* **0.11.0:**
  * **Breaking:** Once again the video memory interface has been updated. It
    should be simpler to follow now, because video memory consistently uses
    `VolGrid2dStrided`, rather than using special types for every screenblock
    style.

* **0.10.0:**
  * **Breaking:** Cleaned up the screenblock interface. Because they're in VRAM,
    they can't use `u8` access like they were defined to use before. Now the
    types use `u8x2`, similar to the Mode 4 bitmap.
  * Fixed the MMIO definition for the OBJ palette, so the OBJ palette should
    work now.
* **0.9.3:**
  * Added `as_u32_slice` and `as_u16_slice` to `Align4`.
  * *Removed* the requirement for inputs to `include_aligned_bytes!` to be a
    multiple of 4 bytes.
  * Added `as_usize` to all the screeblock address types.
* **0.9.2:**
  * Adds support for more BIOS functions, though not all functions are as
    clearly documented as I'd like.
  * Made much more of the `Fixed` type const friendly. Most ops now have an
    inherent method that is `const fn` as well as implementing the `core::ops`
    trait (the trait fn just calls the inherent fn). This means that you can't
    do `x + y` in a const context, but you can do `x.add(y)`. This is not the
    best system, but until const trait impls are stable this is the best middle
    ground.
* **0.9.1:**
  * Adds some randomization support directly into the crate.
  * Added more methods to the `Fixed` type.
  * Adds an `include_aligned_bytes!` macro to pull in compile time data that's
    aligned to 4.
* **0.9.0:**
  * **MSRV:** The crate now requires `compiler_builtins-0.1.81` to build. You
    will need a Nightly from 2022-10-15 or later.
  * **Break:** Quite a bit of the video interface has been replaced, but it
    should be much easier to use now.
  * **Break:** The timer interface has been updated so that fields more closely
    match the mGBA names, for ease of debugging.
* **0.8.0:**
  * **Break:** Removed the macros for `GbaCell` access in favor of just methods.
    I had at first thought that they'd assign registers and then inline, but it
    turns out that the inline phase happens way before the register assignment
    phase, so the macros were unnecessary (and clunky).
  * **Break:** The `IrqFn` type is changed to pass the function an `IrqBits`
    instead of a bare `u16`.
  * Adds functions to pick a screenblock location (one for each screenblock
    type).
  * Add `BitUnPack` BIOS function.
  * Add the `CGA_8X8_THICK` art data.
  * Greatly improved documentation.
* **0.7.4:** Adds mGBA logging support.
* **0.7.3:** Fixes "multiple definition" errors with the AEABI division functions.
  Filed a PR to fix this soon:
  https://github.com/rust-lang/compiler-builtins/pull/495
* **0.7.2:** First version that configures docs.rs properly!
