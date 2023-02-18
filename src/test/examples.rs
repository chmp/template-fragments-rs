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
fn split_templates_example() {
    let template = concat!(
        "<body>\n",
        "  {% for item in items %}\n",
        "  {% fragment item %}\n",
        "    <div>\n",
        "      {{ item }}\n",
        "    </div>\n",
        "  {% endfragment %}\n",
        "  {% endfor %}\n",
        "<body>\n",
    );
    let expected = build_string_map! {
        "" => concat!(
            "<body>\n",
            "  {% for item in items %}\n",
            "    <div>\n",
            "      {{ item }}\n",
            "    </div>\n",
            "  {% endfor %}\n",
            "<body>\n",
        ),
        "item" => concat!(
            "    <div>\n",
            "      {{ item }}\n",
            "    </div>\n",
        ),
    };

    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
    assert_eq!(filter_template(template, "").as_ref(), Ok(&expected[""]));
    assert_eq!(
        filter_template(template, "item").as_ref(),
        Ok(&expected["item"])
    );
}

#[test]
fn block_fragments() {
    let template = concat!(
        "<body>\n",
        "  {% for item in items %}\n",
        "  {% fragment block item %}\n",
        "    <div>\n",
        "      {{ item }}\n",
        "    </div>\n",
        "  {% endfragment block %}\n",
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
    assert_eq!(
        filter_template(template, "item").as_ref(),
        Ok(&expected["item"])
    );
    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
}

#[test]
fn nested_block_fragments() {
    let template = concat!(
        "<body>\n",
        "  {% fragment block outer %}\n",
        "  {% for item in items %}\n",
        "  {% fragment block item %}\n",
        "    <div>\n",
        "      {{ item }}\n",
        "    </div>\n",
        "  {% endfragment block %}\n",
        "  {% endfor %}\n",
        "  {% endfragment block %}\n",
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
    assert_eq!(
        filter_template(template, "outer").as_ref(),
        Ok(&expected["outer"])
    );
    assert_eq!(
        filter_template(template, "item").as_ref(),
        Ok(&expected["item"])
    );
    assert_eq!(split_templates(template).as_ref(), Ok(&expected));
}
