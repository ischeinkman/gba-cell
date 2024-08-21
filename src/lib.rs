//! Provides the [`GbaCell`] type.
//! 
//! ## Safety
//! 
//! **This crate is intended to only be used for writing software on the
//! Nintendo Gameboy Advanced. Use on any other platform may lead to Undefined
//! Behaviour.**

use core::fmt::Debug;

/// Marker trait bound for the methods of [`GbaCell`].
///
/// When a type implements this trait it indicates that the type can be
/// atomically loaded/stored using a single volatile access.
///
/// ## Safety
/// The type must fit in a single register, and have an alignment equal to its
/// size. Generally that means it should be one of:
///
/// * an 8, 16, or 32 bit integer
/// * a function pointer
/// * a data pointer to a sized type
/// * an optional non-null pointer (to function or sized data)
/// * a `repr(transparent)` newtype over one of the above
/// 
/// Note that while the trait requirements are enforcable at the trait level,
/// the size & alignment requirements are enforced using `const` assertions
/// wherever a [`GbaCell`] is used.
pub unsafe trait GbaCellSafe: Copy {}

unsafe impl<T> GbaCellSafe for T where T: Copy {}

/// A "cell" type suitable to hold a global on the GBA.
#[repr(transparent)]
pub struct GbaCell<T>(core::cell::UnsafeCell<T>);

#[cfg(feature = "on_gba")]
impl<T> Debug for GbaCell<T>
where
    T: GbaCellSafe + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as Debug>::fmt(&self.read(), f)
    }
}
impl<T> Default for GbaCell<T>
where
    T: GbaCellSafe + Default,
{
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new(T::default())
    }
}
#[cfg(feature = "on_gba")]
impl<T> Clone for GbaCell<T>
where
    T: GbaCellSafe + Default,
{
    #[inline]
    #[must_use]
    fn clone(&self) -> Self {
        Self::new(self.read())
    }
}

#[cfg(feature = "on_gba")]
unsafe impl<T> Sync for GbaCell<T> {}

impl<T> GbaCell<T>
where
    T: GbaCellSafe,
{
    /// Helper to assert the size & alignment requirements of the wrapped value
    /// at compile time. 
    const _ASSERT_GBACELL_SAFE: () = {
        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();
        match (size, align) {
            (1, 1) | (2, 2) | (4, 4) => {}
            _ => {
                panic!("Provided type cannot be made GbaCell-safe! Expected a size & align of 1, 2, or 4.")
            }
        }
    };

    /// Constructs a new cell with the value given
    #[inline]
    #[must_use]
    pub const fn new(t: T) -> Self {
        Self(core::cell::UnsafeCell::new(t))
    }

    /// Read the value in the cell.
    #[inline]
    #[must_use]
    #[cfg(feature = "on_gba")]
    #[cfg_attr(feature = "track_caller", track_caller)]
    pub fn read(&self) -> T {
        // SAFETY: Guranteed to meet the size & alignment requirements of the
        // GBA's single-instruction reads because of Self::_ASSERT_GBACELL_SAFE.
        unsafe { self.0.get().read_volatile() }
    }

    /// Writes a new value to the cell.
    #[inline]
    #[cfg(feature = "on_gba")]
    #[cfg_attr(feature = "track_caller", track_caller)]
    pub fn write(&self, t: T) {
        // SAFETY: Guranteed to meet the size & alignment requirements of the
        // GBA's single-instruction reads because of Self::_ASSERT_GBACELL_SAFE.
        unsafe { self.0.get().write_volatile(t) }
    }
}
