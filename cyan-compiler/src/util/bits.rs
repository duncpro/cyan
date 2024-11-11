pub trait Truncate<S> {
    fn truncate(self) -> S;
}

impl Truncate<u8> for u32 {
    fn truncate(self) -> u8 { return self as u8; }
}

