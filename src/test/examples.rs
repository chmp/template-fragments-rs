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
