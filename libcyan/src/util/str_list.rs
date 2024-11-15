use std::sync::RwLock;
use std::num::NonZeroU32;

/// A list of strings of heterogenous lengths arranged contiguously in memory.
///
/// Abstractly this data-structure is simply a `Vec<[u8]>` but optimized for iteration speed.
/// Since `StrList` elements are stored contiguously in memory, iterating over all the bytes of
/// all the elements in the list in order has far better locality than the default `Vec<String>`. 
///
/// The other use-case for [`StrList`] is as a sort-of typed bump-allocator. The internal
/// `Vec<u8>` redoubles its capacity when it is exhausted, leaving ample unused space for
/// future strings. This can significantly reduce the number of allocations needed when compared to
/// `Vec::new()`-ing each time a new string appears.
#[derive(Default, Debug)]
pub struct StrList { state: RwLock<Vec<u8>> }

pub type StrListKey = NonZeroU32;

impl StrList {
    pub fn push(&self, str: &[u8]) -> StrListKey {
        let mut arr = self.state.write().unwrap();
        let key = u32::try_from(arr.len() + 1).ok().and_then(NonZeroU32::new).unwrap();
        arr.extend_from_slice(&str.len().to_ne_bytes());
        arr.extend_from_slice(str);
        return key;
    }

    pub fn get(&self, key: StrListKey) -> &[u8] {
        let idx = usize::try_from(key.get() - 1).unwrap();
        let arr = self.state.read().unwrap();
        let content_begin_idx = idx + size_of::<usize>();
        let mut header = [0u8; size_of::<usize>()];
        header.copy_from_slice(&arr[idx..content_begin_idx]);
        let len = usize::from_ne_bytes(header);
        let s = &arr[content_begin_idx..(content_begin_idx + len)];
        return unsafe { std::mem::transmute(s) };
    }

    pub fn shrink_to_fit(&self) {
        let mut arr = self.state.write().unwrap();
        arr.shrink_to_fit();
    }
}


#[derive(Clone, Copy)]
pub struct StrListRef<'a> {
    table: &'a StrList,
    key: StrListKey
}

impl<'a> StrListRef<'a> {
    pub fn new(table: &'a StrList, key: StrListKey) -> Self {
        return Self { table, key };
    }
}

impl<'a> StrListRef<'a> {
    pub fn get(&self) -> &'a [u8] { return self.table.get(self.key); }
}

impl<'a> std::fmt::Debug for StrListRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.get(), f)
    }
}


#[derive(Clone, Copy, Debug)]
pub enum StrRef<'a> {
    List(StrListRef<'a>),
    Slice(&'a [u8])
}

impl<'a> StrRef<'a> {
    pub fn get(&self) -> &'a [u8] {
        match self {
            StrRef::List(ptr) => ptr.get(),
            StrRef::Slice(slice) => slice,
        }
    }
}
