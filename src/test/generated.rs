use super::super::{filter_template, split_templates};

macro_rules! build_string_map {
    ($($key:expr => $value:expr,)*) => {
        {
            let mut res = ::std::collections::HashMap::<String, String>::new();
            $(res.insert(String::from($key), String::from($value));)*
            res
        }
    };
}
#[test]
fn reentrant_fragment() {
    let template = concat!(
        "{% fragment dummy %}\n",
        "{% fragment dummy %}\n",
        "{% endfragment %}\n",
        "{% endfragment %}\n",
    );

    assert!(filter_template(template, "").is_err());
    assert!(filter_template(template, "dummy").is_err());
    assert!(split_templates(template).is_err());
}

#[test]
fn missing_name() {
    let template = concat!(
        "{% fragment %}\n",
        "{% endfragment %}\n",
    );

    assert!(filter_template(template, "").is_err());
    assert!(filter_template(template, "dummy").is_err());
    assert!(split_templates(template).is_err());
}

#[test]
fn named_end() {
    let template = concat!(
        "{% fragment dummy %}\n",
        "{% endfragment dummy %}\n",
    );

    assert!(filter_template(template, "").is_err());
    assert!(filter_template(template, "dummy").is_err());
    assert!(split_templates(template).is_err());
}

#[test]
fn missing_end() {
    let template = concat!(
        "{% fragment example %}\n",
    );

    assert!(filter_template(template, "").is_err());
    assert!(filter_template(template, "dummy").is_err());
    assert!(split_templates(template).is_err());
}

#[test]
fn repeated_ends() {
    let template = concat!(
        "{% fragment example %}\n",
        "{% endfragment %}\n",
        "{% endfragment %}\n",
    );

    assert!(filter_template(template, "").is_err());
    assert!(filter_template(template, "dummy").is_err());
    assert!(split_templates(template).is_err());
}

#[test]
fn trailing_content() {
    let template = concat!(
        "{% fragment example %} invalid\n",
        "{% endfragment %}\n",
    );

    assert!(filter_template(template, "").is_err());
    assert!(filter_template(template, "dummy").is_err());
    assert!(split_templates(template).is_err());
}

#[test]
fn example_1() {
    let template = concat!(
        "<body>\n",
        "<ul>\n",
        "{% fragment listing %}\n",
        "    {% for item in listing %}\n",
        "    <li>{{ item }}</li>\n",
        "    {% endfor %}\n",
        "{% endfragment %}\n",
        "</ul>\n",
        "{% fragment content %}\n",
        "<div>\n",
        "    {% for item in content %}\n",
        "    {% fragment content-item %}\n",
        "    <div>{{ item }}</div>\n",
        "    {% endfragment %}\n",
        "    {% endfor %}\n",
        "</div>\n",
        "{% endfragment %}\n",
        "</body>\n",
    );

    let expected = build_string_map! {
        "" => concat!(
            "<body>\n",
            "<ul>\n",
            "    {% for item in listing %}\n",
            "    <li>{{ item }}</li>\n",
            "    {% endfor %}\n",
            "</ul>\n",
            "<div>\n",
            "    {% for item in content %}\n",
            "    <div>{{ item }}</div>\n",
            "    {% endfor %}\n",
            "</div>\n",
            "</body>\n",
        ),
        "listing" => concat!(
            "    {% for item in listing %}\n",
            "    <li>{{ item }}</li>\n",
            "    {% endfor %}\n",
        ),
        "content" => concat!(
            "<div>\n",
            "    {% for item in content %}\n",
            "    <div>{{ item }}</div>\n",
            "    {% endfor %}\n",
            "</div>\n",
        ),
        "content-item" => concat!(
            "    <div>{{ item }}</div>\n",
        ),
    };
    assert_eq!(filter_template(template, "").as_ref(), Ok(&expected[""]));
    assert_eq!(filter_template(template, "listing").as_ref(), Ok(&expected["listing"]));
    assert_eq!(filter_template(template, "content").as_ref(), Ok(&expected["content"]));
    assert_eq!(filter_template(template, "content-item").as_ref(), Ok(&expected["content-item"]));
    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
}

#[test]
fn example_2() {
    let template = concat!(
        "<body>\n",
        "    {% for item in items %}\n",
        "    {% fragment item %}\n",
        "    <div>\n",
        "        {{ item }}\n",
        "    </div>\n",
        "    {% endfragment %}\n",
        "    {% endfor %}\n",
        "<body>\n",
    );

    let expected = build_string_map! {
        "" => concat!(
            "<body>\n",
            "    {% for item in items %}\n",
            "    <div>\n",
            "        {{ item }}\n",
            "    </div>\n",
            "    {% endfor %}\n",
            "<body>\n",
        ),
        "item" => concat!(
            "    <div>\n",
            "        {{ item }}\n",
            "    </div>\n",
        ),
    };
    assert_eq!(filter_template(template, "").as_ref(), Ok(&expected[""]));
    assert_eq!(filter_template(template, "item").as_ref(), Ok(&expected["item"]));
    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
}

#[test]
fn block_fragments() {
    let template = concat!(
        "<body>\n",
        "  {% for item in items %}\n",
        "  {% fragment-block item %}\n",
        "    <div>\n",
        "      {{ item }}\n",
        "    </div>\n",
        "  {% endfragment-block %}\n",
        "  {% endfor %}\n",
        "<body>\n",
    );

    let expected = build_string_map! {
        "" => concat!(
            "<body>\n",
            "  {% for item in items %}\n",
            "  {% block item %}\n",
            "    <div>\n",
            "      {{ item }}\n",
            "    </div>\n",
            "  {% endblock %}\n",
            "  {% endfor %}\n",
            "<body>\n",
        ),
        "item" => concat!(
            "  {% block item %}\n",
            "    <div>\n",
            "      {{ item }}\n",
            "    </div>\n",
            "  {% endblock %}\n",
        ),
    };
    assert_eq!(filter_template(template, "").as_ref(), Ok(&expected[""]));
    assert_eq!(filter_template(template, "item").as_ref(), Ok(&expected["item"]));
    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
}

#[test]
fn nested_block_fragments() {
    let template = concat!(
        "<body>\n",
        "  {% fragment-block outer %}\n",
        "  {% for item in items %}\n",
        "  {% fragment-block item %}\n",
        "    <div>\n",
        "      {{ item }}\n",
        "    </div>\n",
        "  {% endfragment-block %}\n",
        "  {% endfor %}\n",
        "  {% endfragment-block %}\n",
        "<body>\n",
    );

    let expected = build_string_map! {
        "" => concat!(
            "<body>\n",
            "  {% block outer %}\n",
            "  {% for item in items %}\n",
            "  {% block item %}\n",
            "    <div>\n",
            "      {{ item }}\n",
            "    </div>\n",
            "  {% endblock %}\n",
            "  {% endfor %}\n",
            "  {% endblock %}\n",
            "<body>\n",
        ),
        "item" => concat!(
            "  {% block item %}\n",
            "    <div>\n",
            "      {{ item }}\n",
            "    </div>\n",
            "  {% endblock %}\n",
        ),
        "outer" => concat!(
            "  {% block outer %}\n",
            "  {% for item in items %}\n",
            "  {% block item %}\n",
            "    <div>\n",
            "      {{ item }}\n",
            "    </div>\n",
            "  {% endblock %}\n",
            "  {% endfor %}\n",
            "  {% endblock %}\n",
        ),
    };
    assert_eq!(filter_template(template, "").as_ref(), Ok(&expected[""]));
    assert_eq!(filter_template(template, "item").as_ref(), Ok(&expected["item"]));
    assert_eq!(filter_template(template, "outer").as_ref(), Ok(&expected["outer"]));
    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
}

#[test]
fn repeated_fragment() {
    let template = concat!(
        "{% fragment foo bar %}\n",
        "    <common>\n",
        "{% endfragment %}\n",
        "{% fragment foo %}\n",
        "    <foo>\n",
        "{% endfragment %}\n",
        "{% fragment bar %}\n",
        "    <bar>\n",
        "{% endfragment %}\n",
    );

    let expected = build_string_map! {
        "" => concat!(
            "    <common>\n",
            "    <foo>\n",
            "    <bar>\n",
        ),
        "foo" => concat!(
            "    <common>\n",
            "    <foo>\n",
        ),
        "bar" => concat!(
            "    <common>\n",
            "    <bar>\n",
        ),
    };
    assert_eq!(filter_template(template, "").as_ref(), Ok(&expected[""]));
    assert_eq!(filter_template(template, "foo").as_ref(), Ok(&expected["foo"]));
    assert_eq!(filter_template(template, "bar").as_ref(), Ok(&expected["bar"]));
    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
}

