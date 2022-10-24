use crate::{split_templates, ErrorWithLine};

use super::super::{filter_template, Error};

macro_rules! assert_matches {
    ($left:expr, $pattern:pat,) => {{
        let left = $left;
        if !matches!(left, $pattern) {
            panic!("{:?} does not match {}", left, stringify!($pattern));
        }
    }};
}

#[test]
fn test_unbalanced_tags_no_end() {
    const SOURCE: &'static str = r#"
        {% fragment foo %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::UnbalancedTags)),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::UnbalancedTags)),
    );
}

#[test]
fn test_unbalanced_tags_to_many_ends() {
    const SOURCE: &'static str = r#"
        {% fragment foo %}
        {% endfragment %}
        {% endfragment %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::UnbalancedTags)),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::UnbalancedTags)),
    );
}

#[test]
fn test_start_without_data() {
    const SOURCE: &'static str = r#"
        {% fragment %}
        {% endfragment %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::StartTagWithoutData)),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::StartTagWithoutData)),
    );
}

#[test]
fn test_end_with_data() {
    const SOURCE: &'static str = r#"
        {% fragment foo %}
        {% endfragment foo %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::EndTagWithData)),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::EndTagWithData)),
    );
}

#[test]
fn test_leading_data() {
    const SOURCE: &'static str = r#"
        invalid {% fragment foo %}
        {% endfragment foo %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::LeadingContent)),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::LeadingContent)),
    );
}

#[test]
fn test_trailing_data() {
    const SOURCE: &'static str = r#"
        {% fragment foo %} invalid
        {% endfragment foo %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::TrailingContent)),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::TrailingContent)),
    );
}
