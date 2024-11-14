macro_rules! assert_matches {
    ($e:expr, $p:pat) => {
        let $p = $e else { panic!("pattern did not match"); };
    };
}

pub(crate) use assert_matches;

pub const fn max_of_usizes<const K: usize>(arr: [usize; K]) -> usize {
    assert!(K > 0);
    let mut i: usize = 0;
    let mut max: usize = usize::MIN;
    while i < arr.len() {
        if arr[i] > max {
            max = arr[i];
        }
        i += 1;
    }
    return max;
}
