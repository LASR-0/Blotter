//! RFC 5545 content lines: unfolding, parameters and components on the way
//! in; escaping and line folding on the way out.

use crate::error::{CoreError, Result};

/// One content line: `NAME;PARAM=value:VALUE`.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// Upper case.
    pub name: String,
    /// Names upper case; values unquoted.
    pub params: Vec<(String, String)>,
    /// Raw (still escaped for TEXT values).
    pub value: String,
}

impl Property {
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// The value as TEXT, unescaped.
    pub fn text(&self) -> String {
        unescape(&self.value)
    }

    /// A multi-valued TEXT property (CATEGORIES), split and unescaped.
    pub fn texts(&self) -> Vec<String> {
        split_unescaped(&self.value, ',')
            .into_iter()
            .map(|s| unescape(&s))
            .collect()
    }
}

/// A `BEGIN:…`/`END:…` block and what's inside it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Component {
    /// Upper case, e.g. `VEVENT`.
    pub name: String,
    pub props: Vec<Property>,
    pub children: Vec<Component>,
}

impl Component {
    pub fn prop(&self, name: &str) -> Option<&Property> {
        self.props.iter().find(|p| p.name == name)
    }

    pub fn props_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Property> {
        self.props.iter().filter(move |p| p.name == name)
    }

    /// A TEXT property's unescaped value, if present and not blank.
    pub fn text(&self, name: &str) -> Option<String> {
        self.prop(name)
            .map(Property::text)
            .filter(|s| !s.trim().is_empty())
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Component> {
        self.children.iter().filter(move |c| c.name == name)
    }
}

/// Joins folded lines (a line starting with a space or tab continues the
/// previous one) and drops blank lines.
fn unfold(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in text.trim_start_matches('\u{feff}').lines() {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        match raw.strip_prefix([' ', '\t']) {
            Some(rest) if !lines.is_empty() => {
                if let Some(last) = lines.last_mut() {
                    last.push_str(rest);
                }
            }
            _ if raw.trim().is_empty() => {}
            _ => lines.push(raw.to_owned()),
        }
    }
    lines
}

/// Splits on `sep` outside double quotes.
fn split_quoted(s: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut quoted = false;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if c == '"' {
            quoted = !quoted;
        } else if c == sep && !quoted {
            out.push(&s[start..i]);
            start = i + c.len_utf8();
        }
    }
    out.push(&s[start..]);
    out
}

/// Splits on `sep` where it isn't backslash-escaped. Escapes are kept.
fn split_unescaped(s: &str, sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            cur.push(c);
            if let Some(n) = chars.next() {
                cur.push(n);
            }
        } else if c == sep {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

fn parse_line(line: &str) -> Option<Property> {
    // The value starts at the first colon outside a quoted parameter.
    let mut quoted = false;
    let colon = line.char_indices().find_map(|(i, c)| {
        if c == '"' {
            quoted = !quoted;
        }
        (c == ':' && !quoted).then_some(i)
    })?;
    let (head, value) = (&line[..colon], &line[colon + 1..]);
    let mut parts = split_quoted(head, ';').into_iter();
    let name = parts.next()?.trim().to_ascii_uppercase();
    if name.is_empty() {
        return None;
    }
    let params = parts
        .filter_map(|p| {
            let (k, v) = p.split_once('=')?;
            Some((k.trim().to_ascii_uppercase(), v.trim().replace('"', "")))
        })
        .collect();
    Some(Property {
        name,
        params,
        value: value.to_owned(),
    })
}

/// Parses iCalendar text into its top-level components (normally one
/// `VCALENDAR`). Lenient: lines it can't read are skipped, and an
/// unterminated component is closed at the end.
pub fn parse(text: &str) -> Result<Vec<Component>> {
    let mut roots = Vec::new();
    let mut stack: Vec<Component> = Vec::new();
    for line in unfold(text) {
        let Some(prop) = parse_line(&line) else {
            continue;
        };
        match prop.name.as_str() {
            "BEGIN" => stack.push(Component {
                name: prop.value.trim().to_ascii_uppercase(),
                ..Component::default()
            }),
            "END" => {
                let name = prop.value.trim().to_ascii_uppercase();
                // Close up to the matching BEGIN; a stray END is ignored.
                if !stack.iter().any(|c| c.name == name) {
                    continue;
                }
                while let Some(done) = stack.pop() {
                    let matched = done.name == name;
                    match stack.last_mut() {
                        Some(parent) => parent.children.push(done),
                        None => roots.push(done),
                    }
                    if matched {
                        break;
                    }
                }
            }
            _ => {
                if let Some(cur) = stack.last_mut() {
                    cur.props.push(prop);
                }
            }
        }
    }
    while let Some(done) = stack.pop() {
        match stack.last_mut() {
            Some(parent) => parent.children.push(done),
            None => roots.push(done),
        }
    }
    if !roots.iter().any(|c| c.name == "VCALENDAR") {
        return Err(CoreError::invalid(
            "that file isn't an iCalendar (.ics) file",
        ));
    }
    Ok(roots)
}

pub fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

/// Builds iCalendar text: CRLF line ends, lines folded at 75 octets.
#[derive(Debug, Default)]
pub struct Writer {
    out: String,
}

impl Writer {
    pub fn begin(&mut self, name: &str) {
        self.line("BEGIN", name);
    }

    pub fn end(&mut self, name: &str) {
        self.line("END", name);
    }

    /// `head` is the name with any parameters (`DTSTART;VALUE=DATE`);
    /// `value` is written as is.
    pub fn line(&mut self, head: &str, value: &str) {
        let line = format!("{head}:{value}");
        let mut width = 0;
        for c in line.chars() {
            let len = c.len_utf8();
            // A continuation line's leading space counts towards its 75.
            if width + len > 75 {
                self.out.push_str("\r\n ");
                width = 1;
            }
            self.out.push(c);
            width += len;
        }
        self.out.push_str("\r\n");
    }

    /// A TEXT property, escaped.
    pub fn text(&mut self, head: &str, value: &str) {
        self.line(head, &escape(value));
    }

    pub fn finish(self) -> String {
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unfolds_and_reads_params() {
        let text = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Long \r\n title\r\n\
                    DTSTART;TZID=\"Australia/Melbourne\":20261014T090000\r\n\
                    ATTENDEE;CN=\"Smith; J\":mailto:j@x.org\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let cal = &parse(text).unwrap()[0];
        let ev = &cal.children[0];
        assert_eq!(ev.name, "VEVENT");
        assert_eq!(ev.text("SUMMARY").as_deref(), Some("Long title"));
        let start = ev.prop("DTSTART").unwrap();
        assert_eq!(start.param("tzid"), Some("Australia/Melbourne"));
        assert_eq!(start.value, "20261014T090000");
        let att = ev.prop("ATTENDEE").unwrap();
        assert_eq!(att.param("CN"), Some("Smith; J"));
        assert_eq!(att.value, "mailto:j@x.org");
    }

    #[test]
    fn escapes_round_trip_and_categories_split() {
        let s = "Milk, eggs; bread\\done\nnext";
        assert_eq!(unescape(&escape(s)), s);
        let p = parse_line("CATEGORIES:home,a\\,b, work").unwrap();
        assert_eq!(p.texts(), ["home", "a,b", " work"]);
    }

    #[test]
    fn folds_long_lines_on_char_boundaries() {
        let mut w = Writer::default();
        let title = "é".repeat(60);
        w.text("SUMMARY", &title);
        let out = w.finish();
        for line in out.split("\r\n") {
            assert!(line.len() <= 75, "{} octets", line.len());
        }
        let back = parse(&format!("BEGIN:VCALENDAR\r\n{out}END:VCALENDAR\r\n")).unwrap();
        assert_eq!(back[0].text("SUMMARY").as_deref(), Some(title.as_str()));
    }

    #[test]
    fn rejects_other_files_and_tolerates_damage() {
        assert!(parse("hello").is_err());
        assert!(parse("{ \"format\": \"blotter\" }").is_err());
        // Unterminated event and a stray END still parse.
        let cal = parse("BEGIN:VCALENDAR\nEND:VTODO\nBEGIN:VEVENT\nSUMMARY:x").unwrap();
        assert_eq!(cal[0].children[0].text("SUMMARY").as_deref(), Some("x"));
    }
}
