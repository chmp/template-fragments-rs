mod errors;
mod examples;
mod internals;

macro_rules! hashset {
    ($($part:expr),*) => {
        {
            let mut res = ::std::collections::HashSet::new();
            $(res.insert($part);)*
            res
        }
    };
}

pub(crate) use hashset;

macro_rules! assert_matches {
    ($left:expr, $pattern:pat,) => {{
        let left = $left;
        if !matches!(left, $pattern) {
            panic!("{:?} does not match {}", left, stringify!($pattern));
        }
    }};
}

pub(crate) use assert_matches;
