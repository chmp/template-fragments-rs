use minijinja::context;
use template_fragments::{join_path, split_templates};

fn main() -> Result<(), PanicOnErrors> {
    let env = {
        let mut source = minijinja::Source::new();

        for (path, template) in [("index.html", include_str!("templates/index.html"))] {
            for (fragment_name, template_fragment) in split_templates(template)? {
                source.add_template(join_path(path, &fragment_name), &template_fragment)?;
            }
        }

        let mut env = minijinja::Environment::new();
        env.set_source(source);
        env
    };

    let fragment = std::env::args().nth(1).unwrap_or_default();

    let template = env.get_template(&join_path("index.html", &fragment))?;
    let content = template.render(context!(items => vec!["foo", "bar", "baz"], item => "foo"))?;

    println!("{content}");

    Ok(())
}

#[derive(Debug)]
struct PanicOnErrors;

impl<E: std::fmt::Display> From<E> for PanicOnErrors {
    fn from(err: E) -> Self {
        panic!("Error: {err}")
    }
}
