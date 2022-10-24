# `template-fragments` for jinja-like engines

Usage with minijinja:

```rust
let mut source = minijinja::Source::new();

for (path, template) in  [
    ("index.html", include_str!("templates/index.html")),
] {
    for (fragment_name, template_fragment) in split_templates(template)? {
        if name != "" {
            source.add_template(&format!("{path}#{fragment_name}"), &template_fragment)?;
        } else {
            source.add_template(path, &template_fragment)?;
        }
    }
}

let mut env = minijinja::Environment::new();
env.set_source(source);
```
