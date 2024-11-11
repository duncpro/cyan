macro_rules! assert_matches {
    ($e:expr, $p:pat) => {
        let $p = $e else { panic!("pattern did not match"); };
    };
}

pub(crate) use assert_matches;
