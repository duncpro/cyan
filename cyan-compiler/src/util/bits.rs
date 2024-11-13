pub trait Truncate<S> {
    fn truncate(self) -> S;
}

impl Truncate<u8> for u32 {
    fn truncate(self) -> u8 { return self as u8; }
}

pub fn fast_hash(s: &[u8]) -> usize {
    let mut hash: usize = 0;
    for i in 0..s.len() {
        hash += (s.len() - i + 1).wrapping_mul(usize::from(s[i]));
    }
    return hash;
}
