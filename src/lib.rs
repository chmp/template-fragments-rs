//! Pre-process Jinja-like templates with fragment tags
//!
//! `template_fragments` offers a way to split a template that is annotated with
//! fragment tags (`{% fragment NAME %}` and `{% endfragment %}`) into the
//! fragments. For example:
//!
//! ```html
//! <body>
//! # Header
//! {% fragment items %}
//! {% for item in items %}
//!     {% fragment item %}
//!         <div>{{ item }}</div>
//!     {% endfragment %}
//! {% endfor %}
//! {% endfragment %}
//! <body>
//! ```
//!
//! This template defines three fragments:
//!
//! - `""`: the whole template without any fragment markers
//! - `"items"`: the template looping over all items
//! - `"item"`: the innermost div
//!
//! `template_fragments` offers two ways to pre-process such a template:
//!
//! - [filter_template]: given a fragment name, only return those parts of the
//!   template that belong to the fragment. This function is designed to be used
//!   when templates are requested dynamically
//! - [split_templates]: split a template into all its fragments. This function
//!   is designed to be used when to extract all templates once at application
//!   startup
//!
//! # Syntax
//!
//! - Fragments start with `{% fragment NAMES... %}` or `{% fragment-block NAMES
//!   %}`
//! - `{% fragment-block NAME %}` and `{% endfragment-block %}` define fragment
//!   blocks: they are rendered as a block, if the fragment is included. This is
//!   equivalent to wrapping a block with a fragment of the same name.
//! - Fragments end with `{% endfragment %}` or `{% endfragment-block %}`
//! - Fragments can occur multiple times in the document
//! - Multiple fragments can be started in a single tag by using multiple
//!   whitespace separated names in the start tag
//! - Fragment tags must be contained in a single line and there must not be any
//!   other non-whitespace content on the same line
//! - Fragment names can contain any alphanumeric character and `'-'`, `'_'`.
//!
//! # Example using `minijinja`
//!
//! One way to use fragment tags with  `minijinja` is to build a template source
//! with the split templates at application start up like this
//!
//! ```
//! # mod minijinja {
//! #   pub struct Source;
//! #   impl Source {
//! #     pub fn new() -> Self { Source }
//! #     pub fn add_template(
//! #       &mut self,
//! #       path: String,
//! #       fragment: &str,
//! #     ) -> Result<(), template_fragments::ErrorWithLine> {
//! #       Ok(())
//! #     }
//! #   }
//! # }
//! # fn main() -> Result<(), template_fragments::ErrorWithLine> {
//! use template_fragments::{split_templates, join_path};
//!
//! let mut source = minijinja::Source::new();
//!
//! for (path, template) in [("index.html", include_str!("../examples/templates/index.html"))] {
//!     for (fragment_name, template_fragment) in split_templates(template)? {
//!         source.add_template(join_path(path, &fragment_name), &template_fragment)?;
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Note the different fragments can be rendered by requesting the relevant
//! template, e.g., `env.get_template("index.html")` or
//! `env.get_template("index.html#fragment")`.
//!
use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod test;

const DEFAULT_TAG_MARKERS: (&str, &str) = ("{%", "%}");

/// Split a template path with optional fragment into the path and fragment
///
/// If no fragment is found, the fragment will be a empty string
///
/// ```rust
/// # use template_fragments::split_path;
/// #
/// assert_eq!(split_path("index.html"), ("index.html", ""));
/// assert_eq!(split_path("index.html#child"), ("index.html", "child"));
///
/// // whitespace is normalized
/// assert_eq!(split_path("  index.html  "), ("index.html", ""));
/// assert_eq!(split_path("  index.html  #  child  "), ("index.html", "child"));
/// ```
pub fn split_path(path: &str) -> (&str, &str) {
    if let Some((path, fragment)) = path.rsplit_once('#') {
        (path.trim(), fragment.trim())
    } else {
        (path.trim(), "")
    }
}

/// Join a path with a fragment (omitting empty fragments)
///
/// ```rust
/// # use template_fragments::join_path;
/// #
/// assert_eq!(join_path("index.html", ""), "index.html");
/// assert_eq!(join_path("index.html", "child"), "index.html#child");
///
/// // whitespace is normalized
/// assert_eq!(join_path("  index.html  ", "  "), "index.html");
/// assert_eq!(join_path("  index.html  ", "  child  "), "index.html#child");
/// ```
pub fn join_path(path: &str, fragment: &str) -> String {
    let path = path.trim();
    let fragment = fragment.trim();

    if fragment.is_empty() {
        path.to_string()
    } else {
        format!("{path}#{fragment}")
    }
}

/// Process the template and return all parts for the given fragment
///
/// To obtain the base template use an empty string for the fragment.
///
/// ```rust
/// # use template_fragments::filter_template;
/// let source = concat!(
///     "<body>\n",
///     "  {% fragment item %}\n",
///     "    <div>{{ item }}</div>\n",
///     "  {% endfragment %}\n",
///     "<body>\n",
/// );
///
/// assert_eq!(
///     filter_template(source, "").unwrap(),
///     concat!(
///         "<body>\n",
///         "    <div>{{ item }}</div>\n",
///         "<body>\n",
///     ),
/// );
///
/// assert_eq!(
///     filter_template(source, "item").unwrap(),
///     "    <div>{{ item }}</div>\n",
/// );
/// ```
///
pub fn filter_template(src: &str, fragment: &str) -> Result<String, ErrorWithLine> {
    let mut stack: FragmentStack<'_> = Default::default();
    let mut res = String::new();
    let mut last_line_idx = 0;

    for (line_idx, line) in iterate_with_endings(src).enumerate() {
        last_line_idx = line_idx;

        match parse_fragment_tag(line, DEFAULT_TAG_MARKERS).map_err(|err| err.at(line_idx))? {
            Some(Tag::Start(tag)) => stack.push(tag.fragments).map_err(|err| err.at(line_idx))?,
            Some(Tag::End(_)) => {
                stack.pop().map_err(|err| err.at(line_idx))?;
            }
            Some(Tag::StartBlock(tag)) => {
                stack
                    .push(HashSet::from([tag.fragment]))
                    .map_err(|err| err.at(line_idx))?;
                let line = format!(
                    "{}{{% block {} %}}{}",
                    tag.prefix,
                    tag.fragment,
                    get_ending(line)
                );
                if stack.is_active(fragment) {
                    res.push_str(&line);
                }
            }
            Some(Tag::EndBlock(tag)) => {
                let active = stack.pop().map_err(|err| err.at(line_idx))?;
                let line = format!("{}{{% endblock %}}{}", tag.prefix, get_ending(line));
                if active.contains(fragment) {
                    res.push_str(&line);
                }
            }
            None => {
                if stack.is_active(fragment) {
                    res.push_str(line);
                }
            }
        }
    }
    stack.done().map_err(|err| err.at(last_line_idx))?;

    Ok(res)
}

/// Split the template into all fragments available
///
/// The base template is included as the fragment `""`.
///
/// ```rust
/// # use template_fragments::split_templates;
/// let source = concat!(
///     "<body>\n",
///     "  {% fragment item %}\n",
///     "    <div>{{ item }}</div>\n",
///     "  {% endfragment %}\n",
///     "<body>\n",
/// );
/// let templates = split_templates(source).unwrap();
///
/// assert_eq!(
///     templates[""],
///     concat!(
///         "<body>\n",
///         "    <div>{{ item }}</div>\n",
///         "<body>\n",
///     ),
/// );
///
/// assert_eq!(
///     templates["item"],
///     "    <div>{{ item }}</div>\n",
/// );
/// ```
pub fn split_templates(src: &str) -> Result<HashMap<String, String>, ErrorWithLine> {
    let mut stack: FragmentStack<'_> = Default::default();
    let mut res: HashMap<String, String> = Default::default();
    let mut last_line_idx = 0;

    for (line_idx, line) in iterate_with_endings(src).enumerate() {
        last_line_idx = line_idx;

        match parse_fragment_tag(line, DEFAULT_TAG_MARKERS).map_err(|err| err.at(line_idx))? {
            Some(Tag::Start(tag)) => stack.push(tag.fragments).map_err(|err| err.at(line_idx))?,
            Some(Tag::End(_)) => {
                stack.pop().map_err(|err| err.at(line_idx))?;
            }
            Some(Tag::StartBlock(tag)) => {
                stack
                    .push(HashSet::from([tag.fragment]))
                    .map_err(|err| err.at(line_idx))?;
                let line = format!(
                    "{}{{% block {} %}}{}",
                    tag.prefix,
                    tag.fragment,
                    get_ending(line)
                );
                for fragment in &stack.active_fragments {
                    push_line(&mut res, fragment, &line);
                }
            }
            Some(Tag::EndBlock(tag)) => {
                let fragments = stack.pop().map_err(|err| err.at(line_idx))?;
                let line = format!("{}{{% endblock %}}{}", tag.prefix, get_ending(line));

                for fragment in fragments {
                    push_line(&mut res, fragment, &line);
                }
            }
            None => {
                for fragment in &stack.active_fragments {
                    push_line(&mut res, fragment, line);
                }
            }
        }
    }
    stack.done().map_err(|err| err.at(last_line_idx))?;

    Ok(res)
}

fn push_line(res: &mut HashMap<String, String>, fragment: &str, line: &str) {
    if let Some(target) = res.get_mut(fragment) {
        target.push_str(line);
    } else {
        res.insert(fragment.to_owned(), line.to_owned());
    }
}

fn get_ending(line: &str) -> &str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

#[derive(Debug)]
struct FragmentStack<'a> {
    stack: Vec<HashSet<&'a str>>,
    active_fragments: HashSet<&'a str>,
}

impl<'a> std::default::Default for FragmentStack<'a> {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            active_fragments: HashSet::from([""]),
        }
    }
}

impl<'a> FragmentStack<'a> {
    /// Add new fragments to the currently active fragments
    fn push(&mut self, fragments: HashSet<&'a str>) -> Result<(), Error> {
        let mut reentrant_fragments = Vec::new();

        for &fragment in &fragments {
            let not_seen = self.active_fragments.insert(fragment);
            if !not_seen {
                reentrant_fragments.push(fragment);
            }
        }
        if !reentrant_fragments.is_empty() {
            return Err(Error::ReentrantFragment(sorted_fragments(
                reentrant_fragments,
            )));
        }

        self.stack.push(fragments);
        Ok(())
    }

    /// Pop the last addeed fragments and return the active fragments before
    /// this op
    fn pop(&mut self) -> Result<HashSet<&'a str>, Error> {
        let fragments = self.active_fragments.clone();
        for fragment in self.stack.pop().ok_or(Error::UnbalancedEndTag)? {
            self.active_fragments.remove(fragment);
        }

        Ok(fragments)
    }

    fn done(&self) -> Result<(), Error> {
        if !self.stack.is_empty() {
            let fragments: HashSet<&str> = self.stack.iter().flatten().copied().collect();
            Err(Error::UnclosedTag(sorted_fragments(fragments)))
        } else {
            Ok(())
        }
    }

    fn is_active(&self, fragment: &str) -> bool {
        self.active_fragments.contains(fragment)
    }
}

fn iterate_with_endings(mut s: &str) -> impl Iterator<Item = &str> {
    std::iter::from_fn(move || {
        let res;
        match s.find('\n') {
            Some(new_line_idx) => {
                let split_idx = new_line_idx + '\n'.len_utf8();
                res = Some(&s[..split_idx]);
                s = &s[split_idx..];
            }
            None if !s.is_empty() => {
                res = Some(s);
                s = "";
            }
            None => {
                res = None;
            }
        }
        res
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tag<'a> {
    Start(StartTag<'a>),
    End(EndTag),
    StartBlock(StartBlockTag<'a>),
    EndBlock(EndBlockTag<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartTag<'a> {
    fragments: HashSet<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartBlockTag<'a> {
    prefix: &'a str,
    fragment: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EndBlockTag<'a> {
    prefix: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EndTag;

fn parse_fragment_tag<'l>(
    line: &'l str,
    tag_markers: (&str, &str),
) -> Result<Option<Tag<'l>>, Error> {
    let parts = match parse_base(line, tag_markers) {
        Some(parts) => parts,
        None => return Ok(None),
    };

    if !parts.head.trim().is_empty() {
        return Err(Error::LeadingContent(parts.head.to_owned()));
    }

    if !parts.tail.trim().is_empty() {
        return Err(Error::TrailingContent(parts.tail.to_owned()));
    }

    match parts.fragment_type {
        FragmentType::Start | FragmentType::BlockStart => {
            let data = parts.data.trim();
            if data.is_empty() {
                return Err(Error::StartTagWithoutData);
            }

            let block = matches!(parts.fragment_type, FragmentType::BlockStart);

            let fragments: HashSet<&str> = data.split_whitespace().collect();

            let mut invalid_fragments = Vec::new();
            for &fragment in &fragments {
                if !is_valid_fragment_name(fragment) {
                    invalid_fragments.push(fragment);
                }
            }
            if !invalid_fragments.is_empty() {
                return Err(Error::InvalidFragmentName(sorted_fragments(
                    invalid_fragments,
                )));
            }

            if !block {
                Ok(Some(Tag::Start(StartTag { fragments })))
            } else {
                if fragments.len() > 1 {
                    return Err(Error::MultipleNamesBlock(sorted_fragments(fragments)));
                } else if fragments.is_empty() {
                    return Err(Error::UnnamedBlock);
                }

                let fragment = fragments.into_iter().next().unwrap();
                Ok(Some(Tag::StartBlock(StartBlockTag {
                    prefix: parts.head,
                    fragment,
                })))
            }
        }
        FragmentType::End => {
            if !parts.data.trim().is_empty() {
                return Err(Error::EndTagWithData(parts.data.to_owned()));
            }
            Ok(Some(Tag::End(EndTag)))
        }
        FragmentType::BlockEnd => {
            if !parts.data.trim().is_empty() {
                return Err(Error::EndTagWithData(parts.data.to_owned()));
            }
            Ok(Some(Tag::EndBlock(EndBlockTag { prefix: parts.head })))
        }
    }
}

fn parse_base<'l>(line: &'l str, tag_markers: (&str, &str)) -> Option<LineParts<'l>> {
    // "(?P<head>[^\{]*)\{%\s+(?P<tag>fragment|endfragment)(?P<data>[^%]+)%\}(?P<tail>.*)
    let (head, line) = line.split_once(tag_markers.0)?;
    let line = line.strip_prefix(char::is_whitespace)?;

    use FragmentType as T;

    // NOTE: the order is important: the -block suffixes must come first
    let (fragment_type, line) = None
        .or_else(|| {
            line.strip_prefix("fragment-block")
                .map(|l| (T::BlockStart, l))
        })
        .or_else(|| {
            line.strip_prefix("endfragment-block")
                .map(|l| (T::BlockEnd, l))
        })
        .or_else(|| line.strip_prefix("fragment").map(|l| (T::Start, l)))
        .or_else(|| line.strip_prefix("endfragment").map(|l| (T::End, l)))?;

    let line = line.strip_prefix(char::is_whitespace)?;
    let (data, line) = line.split_once(tag_markers.1)?;
    let tail = line;

    Some(LineParts {
        head,
        fragment_type,
        data,
        tail,
    })
}

fn is_valid_fragment_name(name: &str) -> bool {
    let is_reserved = matches!(name, "block");
    let only_valid_chars = name
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_'));

    !is_reserved && only_valid_chars
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LineParts<'a> {
    head: &'a str,
    fragment_type: FragmentType,
    data: &'a str,
    tail: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FragmentType {
    Start,
    End,
    BlockStart,
    BlockEnd,
}

fn sorted_fragments<'a, I: IntoIterator<Item = &'a str>>(fragments: I) -> String {
    let mut fragments = fragments.into_iter().collect::<Vec<_>>();
    fragments.sort();

    let mut res = String::new();
    for fragment in fragments {
        push_join(&mut res, fragment);
    }
    res
}

/// Errors that can occurs during processing
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Non-whitespace content before the fragment tag
    LeadingContent(String),
    /// None-whitespace content after the fragment tag
    TrailingContent(String),
    /// Endfragment tag with fragment names
    EndTagWithData(String),
    /// Fragment tag without names
    StartTagWithoutData,
    /// Fragment tag with a fragment that is already active
    ReentrantFragment(String),
    /// Tag without end tag
    UnclosedTag(String),
    /// End tag without corresponding start
    UnbalancedEndTag,
    /// Reserved fragment names (at the moment only `block`) or invalid characters
    InvalidFragmentName(String),
    /// A block fragment without a name
    UnnamedBlock,
    /// A block fragmen with too many names
    MultipleNamesBlock(String),
}

impl Error {
    pub fn at(self, line: usize) -> ErrorWithLine {
        ErrorWithLine(line, self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeadingContent(content) => write!(f, "Error::LeadingContent({content:?})"),
            Self::TrailingContent(content) => write!(f, "Error::TrailingContent({content:?})"),
            Self::EndTagWithData(data) => write!(f, "Error::EndTagWithData({data:?})"),
            Self::StartTagWithoutData => write!(f, "Error::StartTagWithoutData"),
            Self::ReentrantFragment(fragments) => write!(f, "Error::ReentrantFragment({fragments}"),
            Self::UnbalancedEndTag => write!(f, "Error::UnbalancedTags"),
            Self::UnclosedTag(fragments) => write!(f, "Error::UnclosedTag({fragments})"),
            Self::InvalidFragmentName(fragments) => {
                write!(f, "Error::InvalidFragmentName({fragments}")
            }
            Self::UnnamedBlock => write!(f, "Error::UnnamedBlock"),
            Self::MultipleNamesBlock(fragments) => {
                write!(f, "Error::MultipleNamesBlock({fragments}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// An error with the line of the template it occurred on
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorWithLine(pub usize, pub Error);

impl std::fmt::Display for ErrorWithLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}", self.1, self.0 + 1)
    }
}

impl std::error::Error for ErrorWithLine {}

fn push_join(s: &mut String, t: &str) {
    if !s.is_empty() {
        s.push_str(", ");
    }
    s.push_str(t);
}
