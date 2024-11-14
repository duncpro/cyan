use std::alloc::{GlobalAlloc, Layout};
use std::marker::PhantomData;
use std::num::NonZeroU32;

#[repr(transparent)]
pub struct Handle<T> {
    key: NonZeroU32,
    pd: PhantomData<T>
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self { return *self; }
}
impl<T> Copy for Handle<T> {}


pub struct BumpAllocator {
    align: usize,
    size: usize,
    ptr: *mut u8,
    pos: usize
}

impl BumpAllocator {
    pub fn bump<T>(&mut self, value: T) -> Handle<T> {
        assert_eq!(align_of::<T>(), self.align);
        assert!(!std::mem::needs_drop::<T>());
        assert!(self.size.saturating_sub(self.pos) >= size_of::<T>());
        let key = self.pos.checked_add(1).unwrap();
        let key = u32::try_from(key).ok().and_then(NonZeroU32::new).unwrap();
        let handle = Handle { key, pd: PhantomData };
        unsafe {
            let ptr = self.ptr.add(self.pos);
            *(self.ptr as *mut T) = value;
        }
        self.pos += size_of::<T>();
        return handle;
    }

    pub unsafe fn deref<'a, T>(&'a self, handle: Handle<T>) -> &'a T {
        let offset = usize::try_from(handle.key.get() - 1).unwrap();
        let ptr = self.ptr.add(offset);
        return std::mem::transmute(ptr);
    }
    
    pub fn new(size: usize, align: usize) -> Self {
        assert!(size > 0);
        assert!(align.is_power_of_two());
        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { std::alloc::System.alloc(layout) };
        assert_ne!(ptr, std::ptr::null_mut());
        return Self { ptr, align, size, pos: 0 };
    }
}
