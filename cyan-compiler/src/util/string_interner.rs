use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU32;
use std::sync::RwLock;

#[derive(Clone, Copy, Debug)]
struct Intern { ptr: *const u8 }

impl Intern {
    fn from_slice(slice: &[u8]) -> Self {
        assert!(slice.ends_with(&[0]));
        return Self { ptr: slice.as_ptr() }
    }
    
    unsafe fn len(&self) -> usize {
        let mut next = self.ptr;
        while *next != 0 {
            next = next.add(1);
        }
        return (next as usize) - (self.ptr as usize);
    }

    unsafe fn as_slice(&self) -> &[u8] {
        return std::slice::from_raw_parts(self.ptr, self.len());
    }
}


#[derive(Clone, Copy, Debug)]
struct Ext {
    ptr: *const u8,
    len: usize
}

impl Ext {
    fn new(s: &[u8]) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len()
        }
    }
    
    unsafe fn as_slice(&self) -> &[u8] {
        return std::slice::from_raw_parts(self.ptr, self.len);
    }
}

#[derive(Clone, Copy, Debug)]
enum Str {
    Intern(Intern),
    Ext(Ext)
}

impl Str {  
    fn ext(s: &[u8]) -> Self {
        return Self::Ext(Ext::new(s));
    }
          
    unsafe fn as_slice(&self) -> &[u8] {
        return match self {
            Self::Intern(intern) => intern.as_slice(),
            Self::Ext(ext) => ext.as_slice(),
        }
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

#[derive(Debug)]
struct State {
    id_lut: HashMap<Str, NonZeroU32>,
    mem: Vec<u8>
}

#[derive(Debug)]
pub struct StringInterner {
    state: RwLock<State>
}

impl Default for StringInterner {
    fn default() -> Self {
        Self {
            state: RwLock::new(State {
                id_lut: HashMap::default(),
                mem: vec![0]
            })
        }
    }
}

impl StringInterner {
    fn lookup_id(&self, s: &[u8]) -> Option<NonZeroU32> {
        let state = self.state.read().unwrap();
        return state.id_lut.get(&Str::ext(s)).copied();
    }
    
    pub fn intern(&self, s: &[u8]) -> NonZeroU32 {
        assert!(s.len() > 0);
        assert!(!s.contains(&0));
        
        if let Some(id) = self.lookup_id(s) {
            return id;
        }

        let mut state = self.state.write().unwrap();
        let begin = state.mem.len();
        state.mem.extend_from_slice(s);
        state.mem.push(0);

        let intern = Intern::from_slice(&state.mem[begin..state.mem.len()]);

        let key = NonZeroU32::new(u32::try_from(begin).unwrap()).unwrap();
        assert!(state.id_lut.insert(Str::Intern(intern), key).is_none());
        return key;
    }

    pub fn lookup_str(&self, id: u32) -> &[u8] {
        let state = self.state.read().unwrap();
        let idx = usize::try_from(id).unwrap();
        assert!(0 < idx && idx < state.mem.len());
        assert!(state.mem[idx - 1] == 0);
        let len = state.mem[idx..].iter()
            .copied()
            .take_while(|b| *b != 0u8)
            .count();
        let s = &state.mem[idx..(idx + len)];

        // Typically, `s` cannot outlive `state`, because if it does, `s` is leaked outside the lock.
        // However, we actually *can* safely leak `s` since StringInterner does not support
        // deletions or reorderings. In otherwords, the memory is immutable once its written,
        // so it no longer needs to be protected by the lock. Here, we use `std::mem::transmute`
        // to replace the lifetime parameter with the lifetime of `self` not `state`.
        unsafe {
            return std::mem::transmute(s);
        }
    }
}
