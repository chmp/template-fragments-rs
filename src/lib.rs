//! Pre-process jinja-like templates with fragment directives
//!

use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod test;


/// Split a template path with optional fragment into the path and fragment
/// 
/// If no fragment is found, the fragment will be a empty string
/// 
/// ```rust
/// # use template_fragments::split_path;
/// assert_eq!(split_path("index.html"), ("index.html", ""));
/// assert_eq!(split_path("index.html#child"), ("index.html", "child"));
/// ```
pub fn split_path(path: &str) -> (&str, &str) {
    if let Some((path, fragment)) = path.rsplit_once('#') {
        (path, fragment)
    } else {
        (path, "")
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

        match parse_fragment_tag(line).map_err(|err| err.at(line_idx))? {
            Some(Tag::Start(fragments)) => stack.push(fragments).map_err(|err| err.at(line_idx))?,
            Some(Tag::End) => stack.pop().map_err(|err| err.at(line_idx))?,
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
/// The base template is return under the empty string.
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

        match parse_fragment_tag(line).map_err(|err| err.at(line_idx))? {
            Some(Tag::Start(fragments)) => stack.push(fragments).map_err(|err| err.at(line_idx))?,
            Some(Tag::End) => stack.pop().map_err(|err| err.at(line_idx))?,
            None => {
                for fragment in &stack.active_fragments {
                    if let Some(target) = res.get_mut(fragment) {
                        target.push_str(line);
                    } else {
                        res.insert(fragment.to_owned(), line.to_owned());
                    }
                }
            }
        }
    }
    stack.done().map_err(|err| err.at(last_line_idx))?;

    Ok(res)
}

#[derive(Debug)]
struct FragmentStack<'a> {
    stack: Vec<HashSet<&'a str>>,
    active_fragments: HashSet<String>,
}

impl<'a> std::default::Default for FragmentStack<'a> {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            active_fragments: [String::new()].into(),
        }
    }
}

impl<'a> FragmentStack<'a> {
    fn push(&mut self, fragments: HashSet<&'a str>) -> Result<(), Error> {
        for &fragment in &fragments {
            let not_seen = self.active_fragments.insert(fragment.to_owned());
            if !not_seen {
                return Err(Error::ReentrantFragment);
            }
        }
        self.stack.push(fragments);
        Ok(())
    }

    fn pop(&mut self) -> Result<(), Error> {
        for fragment in self.stack.pop().ok_or(Error::UnbalancedTags)? {
            self.active_fragments.remove(fragment);
        }
        Ok(())
    }

    fn done(&self) -> Result<(), Error> {
        if !self.stack.is_empty() {
            Err(Error::UnbalancedTags)
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
    Start(HashSet<&'a str>),
    End,
}

fn parse_fragment_tag(line: &str) -> Result<Option<Tag<'_>>, Error> {
    let parts = match parse_base(line) {
        Some(parts) => parts,
        None => return Ok(None),
    };

    if !parts.head.trim().is_empty() {
        return Err(Error::LeadingContent);
    }

    if !parts.tail.trim().is_empty() {
        return Err(Error::TrailingContent);
    }

    if parts.start && parts.data.trim().is_empty() {
        return Err(Error::StartTagWithoutData);
    }

    if !parts.start && !parts.data.trim().is_empty() {
        return Err(Error::EndTagWithData);
    }

    if parts.start {
        let fragments = parts.data.split_whitespace().collect();
        Ok(Some(Tag::Start(fragments)))
    } else {
        Ok(Some(Tag::End))
    }
}

fn parse_base(line: &str) -> Option<LineParts<'_>> {
    // "(?P<head>[^\{]*)\{%\s+(?P<tag>fragment|endfragment)(?P<data>[^%]+)%\}(?P<tail>.*)
    let (head, line) = line.split_once("{%")?;
    let line = line.strip_prefix(char::is_whitespace)?;
    let line = line.trim_start();

    #[allow(clippy::manual_map)]
    let (start, line) = if let Some(line) = line.strip_prefix("fragment") {
        Some((true, line))
    } else if let Some(line) = line.strip_prefix("endfragment") {
        Some((false, line))
    } else {
        None
    }?;

    let line = line.strip_prefix(char::is_whitespace)?;
    let (data, line) = line.split_once("%}")?;
    let tail = line;

    Some(LineParts {
        head,
        start,
        data,
        tail,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LineParts<'a> {
    head: &'a str,
    start: bool,
    data: &'a str,
    tail: &'a str,
}

/// The errors that can occurs during processing
/// 
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Content before the fragment tag 
    LeadingContent,
    /// Content after the fragment tag
    TrailingContent,
    /// Endfragment tag with fragment names
    EndTagWithData,
    /// Fragment tag without names
    StartTagWithoutData,
    /// Fragment tag with a fragment that is already active
    ReentrantFragment,
    /// Unbalanced fragment and endfragment tags
    UnbalancedTags,
}

impl Error {
    pub fn at(self, line: usize) -> ErrorWithLine {
        ErrorWithLine(line, self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeadingContent => write!(f, "Error::LeadingContent"),
            Self::TrailingContent => write!(f, "Error::TrailingContent"),
            Self::EndTagWithData => write!(f, "Error::EndTagWithData"),
            Self::StartTagWithoutData => write!(f, "Error::StartTagWithoutData"),
            Self::ReentrantFragment => write!(f, "Error::ReentrantFragment"),
            Self::UnbalancedTags => write!(f, "Error::UnbalancedTags"),
        }
    }
}

impl std::error::Error for Error {}

/// An error with the line it occurred on
/// 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorWithLine(pub usize, pub Error);

impl std::fmt::Display for ErrorWithLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}", self.1, self.0 + 1)
    }
}

impl std::error::Error for ErrorWithLine {}
