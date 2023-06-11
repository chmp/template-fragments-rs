"""Generate rust tests"""
import functools as ft
import json
import tomli

from pathlib import Path

self_path = Path(__file__).parent.resolve()

def main():
    dst = self_path / ".." / "src" / "test" / "generated.rs"
    
    print(":: update", dst)
    with open(dst, "wt") as fobj_dst:
        p = ft.partial(print, file=fobj_dst)

        p("use super::super::{filter_template, split_templates};")
        p()
        p("macro_rules! build_string_map {")
        p("    ($($key:expr => $value:expr,)*) => {")
        p("        {")
        p("            let mut res = ::std::collections::HashMap::<String, String>::new();")
        p("            $(res.insert(String::from($key), String::from($value));)*")
        p("            res")
        p("        }")
        p("    };")
        p("}")

        for p in self_path.glob("*.toml"):
            with p.open("rb") as fobj:
                spec = tomli.load(fobj)

            
            for test in spec["test"]:
                generate_test(test, fobj=fobj_dst)

    print("done")


def generate_test(test, *, fobj):
    p = ft.partial(print, file=fobj)
    newline = "\n"
    
    test_func_name = test["name"].replace(" ", "_")

    p(f"#[test]")
    p(f"fn {test_func_name}() {{")
    p(f"    let template = concat!(")

    for line in test["source"].splitlines():
        p(f"        {json.dumps(line.rstrip() + newline)},")
    p(f"    );")
    p()
    
    if any("expected" in test for test in test["fragment"]):
        p(f"    let expected = build_string_map! {{")
        for fragment in test["fragment"]:
            p(f"        {json.dumps(fragment['name'])} => concat!(")
            for line in fragment["expected"].splitlines():
                p(f"            {json.dumps(line.rstrip() + newline)},")
            p(f"        ),")
        p(f"    }};")
    
    if not test.get("error", False):
        for fragment in test["fragment"]:
            p(
                f"    assert_eq!(filter_template(template, " 
                f"{json.dumps(fragment['name'])}).as_ref(), "
                f"Ok(&expected[{json.dumps(fragment['name'])}]));"
            )

        p("    assert_eq!(split_templates(template).as_ref(), Ok(&expected));")
    
    else:
        for fragment in test["fragment"]:
            p(f"    assert!(filter_template(template, {json.dumps(fragment['name'])}).is_err());")

        p("    assert!(split_templates(template).is_err());")
    
    p(f"}}")
    p()


if __name__ == "__main__":
    main()
