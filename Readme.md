# `template-fragments` for jinja-like engines

Usage with minijinja (see also [`examples/minijinja.rs`](examples/minijinja.rs)):

```rust
use template_fragments::{join_path, split_templates};

let mut source = minijinja::Source::new();

for (path, template) in  [
    ("index.html", include_str!("templates/index.html")),
    ("users.html", include_str!("templates/users.html")),
] {
    for (fragment_name, template_fragment) in split_templates(template)? {
        source.add_template(join_path(path, &fragment_name), &template_fragment)?;
    }
}
```

