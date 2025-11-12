use std::mem::MaybeUninit;

/// Private sealing trait - prevents external implementations
mod private {
    pub trait Sealed {}

    impl Sealed for [std::mem::MaybeUninit<u8>] {}
    impl Sealed for [u8] {}
}

/// Public trait for types that can be used as hex encoding/decoding buffers.
///
/// This trait is sealed and cannot be implemented outside this crate.
/// It is implemented for `[MaybeUninit<u8>]` and `[u8]`.
pub trait Buf: private::Sealed {
    /// Returns a mutable reference to the buffer as `MaybeUninit<u8>` slices.
    ///
    /// # Safety
    ///
    /// The caller MUST only write fully initialized bytes through the returned
    /// reference. Writing uninitialized bytes when the underlying buffer is
    /// `&mut [u8]` is undefined behavior.
    ///
    /// This method is not directly callable by external code.
    #[doc(hidden)]
    unsafe fn dst(&mut self) -> &mut [MaybeUninit<u8>];
}

impl Buf for [MaybeUninit<u8>] {
    unsafe fn dst(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
}

impl Buf for [u8] {
    unsafe fn dst(&mut self) -> &mut [MaybeUninit<u8>] {
        // SAFETY: This is sound because:
        // 1. T and MaybeUninit<T> are ABI-compatible
        // 2. The lifetime of the reference is tied to &mut self, preventing aliasing
        // 3. The caller is responsible for only writing initialized bytes (per trait contract)
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), self.len())
        }
    }
}
