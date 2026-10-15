//! A small Server-Sent Events parser: feed it bytes as they arrive, get
//! back complete events. Handles CRLF, multi-line `data:` and comments.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The `event:` field, if any (Anthropic names its events; OpenAI doesn't).
    pub name: Option<String>,
    pub data: String,
}

#[derive(Default)]
pub struct Parser {
    buf: Vec<u8>,
    name: Option<String>,
    data: Vec<String>,
}

impl Parser {
    /// Adds bytes and returns every event they complete.
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Event> {
        self.buf.extend_from_slice(bytes);
        let mut out = Vec::new();
        while let Some(pos) = self.buf.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&line);
            let line = line.trim_end_matches(['\n', '\r']);
            if line.is_empty() {
                if !self.data.is_empty() {
                    out.push(Event {
                        name: self.name.take(),
                        data: self.data.join("\n"),
                    });
                    self.data.clear();
                } else {
                    self.name = None;
                }
                continue;
            }
            if line.starts_with(':') {
                continue; // comment / keep-alive
            }
            let (field, value) = line.split_once(':').unwrap_or((line, ""));
            let value = value.strip_prefix(' ').unwrap_or(value);
            match field {
                "event" => self.name = Some(value.to_owned()),
                "data" => self.data.push(value.to_owned()),
                _ => {} // id, retry: unused
            }
        }
        out
    }

    /// Flushes a final event that wasn't followed by a blank line.
    pub fn finish(&mut self) -> Option<Event> {
        let rest = std::mem::take(&mut self.buf);
        if !rest.is_empty() {
            let mut evs = self.feed(&[rest.as_slice(), b"\n"].concat());
            if let Some(e) = evs.pop() {
                return Some(e);
            }
        }
        (!self.data.is_empty()).then(|| Event {
            name: self.name.take(),
            data: std::mem::take(&mut self.data).join("\n"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_events_across_chunks() {
        let mut p = Parser::default();
        let mut got = p.feed(b"event: message_start\r\ndata: {\"a\"");
        assert!(got.is_empty());
        got.extend(p.feed(b":1}\r\n\r\n: ping\n\ndata: [DONE]\n\n"));
        assert_eq!(
            got,
            [
                Event {
                    name: Some("message_start".into()),
                    data: "{\"a\":1}".into()
                },
                Event {
                    name: None,
                    data: "[DONE]".into()
                },
            ]
        );
    }

    #[test]
    fn joins_multi_line_data_and_flushes_the_tail() {
        let mut p = Parser::default();
        assert!(p.feed(b"data: one\ndata: two\n").is_empty());
        assert_eq!(p.finish().unwrap().data, "one\ntwo");
    }
}
