mod iterate_with_endings {
    use crate::iterate_with_endings;

    #[test]
    fn trailing_newline() {
        let mut iter = iterate_with_endings("foo\nbar\r\nbaz\n");
        assert_eq!(iter.next(), Some("foo\n"));
        assert_eq!(iter.next(), Some("bar\r\n"));
        assert_eq!(iter.next(), Some("baz\n"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn without_trailing_newline() {
        let mut iter = iterate_with_endings("foo\nbar");
        assert_eq!(iter.next(), Some("foo\n"));
        assert_eq!(iter.next(), Some("bar"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn without_content() {
        let mut iter = iterate_with_endings("");
        assert_eq!(iter.next(), None);
    }
}

mod parse_fragment_tag {
    use crate::{
        parse_fragment_tag,
        test::{assert_matches, hashset},
        Error, StartTag, Tag, DEFAULT_TAG_MARKERS,
    };

    #[test]
    fn parse_fragment_tag_examples() {
        assert_eq!(
            parse_fragment_tag("  {% fragment foo %}", DEFAULT_TAG_MARKERS),
            Ok(Some(Tag::Start(StartTag {
                fragments: hashset!["foo"]
            })))
        );
        assert_eq!(
            parse_fragment_tag("  {% fragment foo bar %}", DEFAULT_TAG_MARKERS),
            Ok(Some(Tag::Start(StartTag {
                fragments: hashset!["foo", "bar"]
            })))
        );
        assert_matches!(
            parse_fragment_tag("  {% endfragment %}", DEFAULT_TAG_MARKERS),
            Ok(Some(Tag::End(_))),
        );
        assert_eq!(
            parse_fragment_tag("  {% fragment %}", DEFAULT_TAG_MARKERS),
            Err(Error::StartTagWithoutData)
        );
    }
}

mod parse_base {
    use crate::{parse_base, LineParts, DEFAULT_TAG_MARKERS};

    #[test]
    fn parse_base_examples() {
        assert_eq!(
            parse_base("abc{% fragment %}def", DEFAULT_TAG_MARKERS),
            Some(LineParts {
                head: "abc",
                start: true,
                data: "",
                tail: "def"
            })
        );
        assert_eq!(
            parse_base("abc{% endfragment %}def", DEFAULT_TAG_MARKERS),
            Some(LineParts {
                head: "abc",
                start: false,
                data: "",
                tail: "def"
            })
        );
        assert_eq!(
            parse_base("abc{% fragment 123 456 %}def", DEFAULT_TAG_MARKERS),
            Some(LineParts {
                head: "abc",
                start: true,
                data: "123 456 ",
                tail: "def"
            })
        );
        assert_eq!(
            parse_base("{% fragment %}", DEFAULT_TAG_MARKERS),
            Some(LineParts {
                head: "",
                start: true,
                data: "",
                tail: ""
            })
        );

        // missing space before
        assert_eq!(parse_base("abc{%fragment %}def", DEFAULT_TAG_MARKERS), None);
        // missing space after
        assert_eq!(parse_base("abc{% fragment%}def", DEFAULT_TAG_MARKERS), None);
        // invalid tag
        assert_eq!(parse_base("abc{% dummy %}def", DEFAULT_TAG_MARKERS), None);
    }
}

mod std_api {
    /// strip prefix with a char predicate only removes a single character
    #[test]
    fn strip_prefix() {
        assert_eq!("  ".strip_prefix(char::is_whitespace), Some(" "));
    }

    /// test that split_once does not include the separator itself
    #[test]
    fn split_once() {
        assert_eq!("abc{%def".split_once("{%"), Some(("abc", "def")));
    }

    #[test]
    fn split_whitespace() {
        let mut iter = "  ".split_whitespace();
        assert_eq!(iter.next(), None);

        let mut iter = "  foo    bar  ".split_whitespace();
        assert_eq!(iter.next(), Some("foo"));
        assert_eq!(iter.next(), Some("bar"));
        assert_eq!(iter.next(), None);
    }
}

mod is_valid_fragment_name {
    use crate::is_valid_fragment_name;

    #[test]
    fn examples() {
        assert_eq!(true, is_valid_fragment_name("hello"));
        assert_eq!(true, is_valid_fragment_name("--hello"));
        assert_eq!(true, is_valid_fragment_name("hello-foo"));
        assert_eq!(true, is_valid_fragment_name("hello-foo-bar"));
        assert_eq!(true, is_valid_fragment_name("hello-foo-bar-123"));
        assert_eq!(true, is_valid_fragment_name("123-hello-foo-bar"));
        assert_eq!(true, is_valid_fragment_name("123"));
        assert_eq!(false, is_valid_fragment_name("@hello"));
        assert_eq!(true, is_valid_fragment_name("hello_foo"));
    }

    #[test]
    fn reserved_names() {
        assert_eq!(false, is_valid_fragment_name("block"));
    }
}
