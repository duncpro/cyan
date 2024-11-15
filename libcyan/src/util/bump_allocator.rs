use std::alloc::{handle_alloc_error, GlobalAlloc, Layout};
use std::marker::PhantomData;
use std::num::NonZeroU32;

// -- Handle -------------------------------------------------------------------------------------

#[repr(transparent)]
pub struct Handle<T> {
    key: NonZeroU32,
    pd: PhantomData<T>
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self { return *self; }
}
impl<T> Copy for Handle<T> {}

// -- BumpAllocator ------------------------------------------------------------------------------

pub struct BumpAllocator {
    layout: Layout,
    ptr: *mut u8,
    pos: usize
}

impl BumpAllocator {    
    /// Allocates a new [`BumpAllocator`] arena with capacity `size` via the system allocator.
    pub fn new(size: usize, align: usize) -> Self {
        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { std::alloc::System.alloc(layout) };
        assert_ne!(ptr, std::ptr::null_mut());
        return Self { layout, ptr, pos: 0 };
    }
    
    /// Moves `value` into the arena and returns a reference to it in the form of a [`Handle`].
    /// Moves the cursor forward to the next unused byte.
    ///
    /// This procedure will panic if `value` does not fit into the remaining free space in the arena.
    pub fn bump<T>(&mut self, value: T) -> Handle<T> {
        assert_eq!(align_of::<T>(), self.layout.align(), "{}'s alignment ({}) does not match the \
            allocator's ({}).", std::any::type_name::<T>(), align_of::<T>(), self.layout.align());
        assert!(!std::mem::needs_drop::<T>());
        assert!(self.layout.size().saturating_sub(self.pos) >= size_of::<T>());
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

    /// Given a handle to an object in the allocation, returns a constant pointer to that object.
    ///
    /// Safety: This procedure is unsafe because invoking it with a foreign handle may
    /// produce an inconsistently typed pointer (pointer is type *T but T is not at the address).
    /// Moreover, this procedure can be misued to produce multiple exclusive references to the
    /// same memory address.
    pub unsafe fn get<T>(&self, handle: Handle<T>) -> *const T {
        return self.get_mut(handle);
    }
    
    /// Given a handle to an object in the allocation, returns a mutable pointer to that object.
    ///
    /// Safety: This procedure is unsafe because invoking it with a foreign handle may
    /// produce an inconsistently typed pointer (pointer is type *T but T is not at the address).
    /// Moreover, this procedure can be misued to produce multiple exclusive references to the
    /// same memory address.
    pub unsafe fn get_mut<T>(&self, handle: Handle<T>) -> *mut T {
        assert_eq!(align_of::<T>(), self.layout.align(), "{} does not have align {}, \
            it could never have been allocated in this arena", std::any::type_name::<T>(),
            self.layout.align());
        let offset = usize::try_from(handle.key.get() - 1).unwrap();
        assert!(offset < self.layout.size());
        let ptr = self.ptr.add(offset);
        return std::mem::transmute(ptr);
    }

    pub fn shrink_to_fit(&mut self) {
        let new_layout = Layout::from_size_align(self.pos, self.layout.align()).unwrap();
        let ptr = unsafe { std::alloc::System.realloc(self.ptr, self.layout, self.pos) };
        if ptr.is_null() { handle_alloc_error(new_layout); };
        self.layout = new_layout;
        self.ptr = ptr;
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        unsafe {
            std::alloc::System.dealloc(self.ptr, self.layout);
        }
    }
}

// -- Linked List Support ------------------------------------------------------------------------

pub trait LLNode: Sized {
    fn next_mut(&mut self) -> &mut Option<Handle<Self>>;
}

pub fn extend_ll<T: LLNode>(mem: &mut BumpAllocator, tail: &mut &mut Option<Handle<T>>, with: T) {
    let handle = mem.bump(with);
    **tail = Some(handle);
    *tail = unsafe { (*mem.get_mut(handle)).next_mut() };
}
