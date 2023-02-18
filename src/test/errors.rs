use crate::{filter_template, split_templates, test::assert_matches, Error, ErrorWithLine};

#[test]
fn unbalanced_tags_no_end() {
    const SOURCE: &'static str = r#"
        {% fragment foo %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::UnclosedTag(_))),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::UnclosedTag(_))),
    );
}

#[test]
fn unbalanced_tags_to_many_ends() {
    const SOURCE: &'static str = r#"
        {% fragment foo %}
        {% endfragment %}
        {% endfragment %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::UnbalancedEndTag)),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::UnbalancedEndTag)),
    );
}

#[test]
fn start_without_data() {
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
fn end_with_data() {
    const SOURCE: &'static str = r#"
        {% fragment foo %}
        {% endfragment foo %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::EndTagWithData(_))),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::EndTagWithData(_))),
    );
}

#[test]
fn leading_data() {
    const SOURCE: &'static str = r#"
        invalid {% fragment foo %}
        {% endfragment %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::LeadingContent(_))),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::LeadingContent(_))),
    );
}

#[test]
fn trailing_data() {
    const SOURCE: &'static str = r#"
        {% fragment foo %} invalid
        {% endfragment %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::TrailingContent(_))),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::TrailingContent(_))),
    );
}

#[test]
fn invalid_tag_name() {
    const SOURCE: &'static str = r#"
        {% fragment foo block %}
        {% endfragment %}
    "#;

    assert_matches!(
        filter_template(SOURCE, "foo"),
        Err(ErrorWithLine(_, Error::InvalidFragmentName(_))),
    );
    assert_matches!(
        split_templates(SOURCE),
        Err(ErrorWithLine(_, Error::InvalidFragmentName(_))),
    );
}
