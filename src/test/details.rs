use super::super::{iterate_with_endings, parse_base, parse_fragment_tag, Error, LineParts, Tag};

macro_rules! hashset {
    ($($part:expr),*) => {
        {
            let mut res = ::std::collections::HashSet::new();
            $(res.insert($part);)*
            res
        }
    };
}

#[test]
fn iterate_with_endings_example_with_trailing_newline() {
    let mut iter = iterate_with_endings("foo\nbar\r\nbaz\n");
    assert_eq!(iter.next(), Some("foo\n"));
    assert_eq!(iter.next(), Some("bar\r\n"));
    assert_eq!(iter.next(), Some("baz\n"));
    assert_eq!(iter.next(), None);
}

#[test]
fn iterate_with_endings_example_without_trailing_newline() {
    let mut iter = iterate_with_endings("foo\nbar");
    assert_eq!(iter.next(), Some("foo\n"));
    assert_eq!(iter.next(), Some("bar"));
    assert_eq!(iter.next(), None);
}

#[test]
fn iterate_with_endings_example_without_content() {
    let mut iter = iterate_with_endings("");
    assert_eq!(iter.next(), None);
}

#[test]
fn parse_fragment_tag_examples() {
    assert_eq!(
        parse_fragment_tag("  {% fragment foo %}"),
        Ok(Some(Tag::Start(hashset!["foo"])))
    );
    assert_eq!(
        parse_fragment_tag("  {% fragment foo bar %}"),
        Ok(Some(Tag::Start(hashset!["foo", "bar"])))
    );
    assert_eq!(
        parse_fragment_tag("  {% endfragment %}"),
        Ok(Some(Tag::End))
    );
    assert_eq!(
        parse_fragment_tag("  {% fragment %}"),
        Err(Error::StartTagWithoutData)
    );
}

#[test]
fn parse_base_examples() {
    assert_eq!(
        parse_base("abc{% fragment %}def"),
        Some(LineParts {
            head: "abc",
            start: true,
            data: "",
            tail: "def"
        })
    );
    assert_eq!(
        parse_base("abc{% endfragment %}def"),
        Some(LineParts {
            head: "abc",
            start: false,
            data: "",
            tail: "def"
        })
    );
    assert_eq!(
        parse_base("abc{% fragment 123 456 %}def"),
        Some(LineParts {
            head: "abc",
            start: true,
            data: "123 456 ",
            tail: "def"
        })
    );
    assert_eq!(
        parse_base("{% fragment %}"),
        Some(LineParts {
            head: "",
            start: true,
            data: "",
            tail: ""
        })
    );

    // missing space before
    assert_eq!(parse_base("abc{%fragment %}def"), None);
    // missing space after
    assert_eq!(parse_base("abc{% fragment%}def"), None);
    // invalid tag
    assert_eq!(parse_base("abc{% dummy %}def"), None);
}

/// strip prefix with a char predicate only removes a single character
#[test]
fn strip_prefix_api() {
    assert_eq!("  ".strip_prefix(char::is_whitespace), Some(" "));
}

/// test that split_once does not include the separator itself
#[test]
fn split_once_api() {
    assert_eq!("abc{%def".split_once("{%"), Some(("abc", "def")));
}

#[test]
fn split_whitespace_api() {
    let mut iter = "  ".split_whitespace();
    assert_eq!(iter.next(), None);

    let mut iter = "  foo    bar  ".split_whitespace();
    assert_eq!(iter.next(), Some("foo"));
    assert_eq!(iter.next(), Some("bar"));
    assert_eq!(iter.next(), None);
}
