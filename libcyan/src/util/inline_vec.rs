use std::mem::MaybeUninit;

pub struct InlineVec<T, const SIZE: usize> {
    arr: [MaybeUninit<T>; SIZE],
    len: usize
}

impl<T, const SIZE: usize> InlineVec<T, SIZE> {
    pub fn new() -> Self {
        return Self { 
            arr: [const { MaybeUninit::uninit() }; SIZE],
            len: 0
        };
    }

    pub fn push(&mut self, value: T) {
        assert!(self.len < SIZE, "unable to push because {} is full", std::any::type_name::<Self>());
        let mut slot = &mut self.arr[self.len];
        slot.write(value);
        self.len += 1;
    }
}

impl<T, const SIZE: usize> Drop for InlineVec<T, SIZE> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.arr[i].assume_init_drop();
            }
        }
    }
}
