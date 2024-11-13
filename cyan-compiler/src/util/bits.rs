pub trait Truncate<S> {
    fn truncate(self) -> S;
}

impl Truncate<u8> for u32 {
    fn truncate(self) -> u8 { return self as u8; }
}

/// Fast but insecure hash function taken from the JDK String#toString() method.
pub fn fast_hash(s: &[u8]) -> usize {
    let mut hash: usize = 0;
    for i in 0..s.len() {
        hash += 31usize.wrapping_pow((s.len() - i + 1) as u32)
            .wrapping_mul(usize::from(s[i]));
        
    }
    return hash;
}
