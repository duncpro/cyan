use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, RwLock};
use crate::util::str_list::{StrRef, StrList, StrListKey};

#[derive(Clone, Copy, Debug)]
struct Str { ptr: *const u8, len: usize }

impl Str {
    fn from_slice(slice: &[u8]) -> Self {
        return Self { ptr: slice.as_ptr(), len: slice.len() }
    }
    unsafe fn as_slice(&self) -> &[u8] {
        return std::slice::from_raw_parts(self.ptr, self.len);
    }
}


impl PartialEq for Str {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            return self.as_slice() == other.as_slice();
        }
    }
}

impl Eq for Str {}

impl Hash for Str {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            return self.as_slice().hash(state);
        }
    }
}

#[derive(Default, Debug)]
pub struct StrInterner {
    lut: Mutex<HashMap<Str, usize>>,
    str_list: StrList
}

impl StrInterner {  
    pub fn intern(&self, s: &[u8]) -> StrListKey {                
        let mut lut = self.lut.lock().unwrap();
        if let Some(id) = lut.get(&Str::from_slice(s)).copied() { return id; }
        let str_list_key = self.str_list.push(s);
        let intern = Str::from_slice(self.str_list.get(str_list_key).unwrap());
        assert!(lut.insert(intern, str_list_key).is_none());
        return str_list_key;
    }

    pub fn str_list(&self) -> &StrList { return &self.str_list; }
}

