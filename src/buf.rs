use std::mem::MaybeUninit;

pub trait Buf {
    fn dst(&mut self) -> &mut [MaybeUninit<u8>];
}

impl Buf for [MaybeUninit<u8>] {
    fn dst(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
}

impl Buf for [u8] {
    fn dst(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), self.len())
        }
    }
}
