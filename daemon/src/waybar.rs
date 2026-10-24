//! Waybar JSON rendering with Python `json.dumps` byte parity.
//!
//! Python emits `json.dumps(dict, ensure_ascii=True)`: keys in insertion
//! order, `": "`/`", "` separators, every non-ASCII char as `\uXXXX`.
//! `serde_json::Value` instead sorts keys and keeps UTF-8 raw —
//! semantically equal but byte-different. Tests asserting exact daemon
//! stdout (and any byte-strict Waybar consumer) need the Python bytes,
//! so state types render through [`WaybarJson`] + [`dumps_ascii`].

use serde_json::Value;

/// One Waybar payload preserving Python dict insertion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaybarJson {
    pub fields: Vec<(&'static str, String)>,
}

impl WaybarJson {
    pub fn new(fields: Vec<(&'static str, String)>) -> Self {
        Self { fields }
    }

    pub fn value(&self) -> Value {
        let mut map = serde_json::Map::with_capacity(self.fields.len());
        for (key, val) in &self.fields {
            map.insert(key.to_string(), Value::String(val.clone()));
        }
        Value::Object(map)
    }

    /// `json.dumps` with `ensure_ascii=True`: insertion order, `": "`
    /// and `", "` separators, non-ASCII as `\uXXXX` (surrogate pairs
    /// for astral chars like emoji).
    pub fn dumps_ascii(&self) -> String {
        let mut out = String::from("{");
        for (index, (key, val)) in self.fields.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push('"');
            out.push_str(key);
            out.push_str("\": ");
            push_quoted_ascii(&mut out, val);
        }
        out.push('}');
        out
    }
}

fn push_quoted_ascii(out: &mut String, text: &str) {
    out.push('"');
    for char in text.chars() {
        match char {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            char if (char as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", char as u32)),
            char if (char as u32) < 0x7f => out.push(char),
            char => {
                let code = char as u32;
                if code > 0xFFFF {
                    let code = code - 0x10000;
                    out.push_str(&format!(
                        "\\u{:04x}\\u{:04x}",
                        0xD800 + (code >> 10),
                        0xDC00 + (code & 0x3FF)
                    ));
                } else {
                    out.push_str(&format!("\\u{code:04x}"));
                }
            }
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dumps_matches_python_json_dumps() {
        let payload = WaybarJson::new(vec![
            ("text", "\u{f241}".to_string()),
            ("class", "conservation-on".to_string()),
            ("tooltip", "Conservação de Bateria: LIGADO (~80%)".to_string()),
        ]);
        // Exact bytes of Python `json.dumps` with `ensure_ascii=True`.
        let expected = r#"{"text": "\uf241", "class": "conservation-on", "tooltip": "Conserva\u00e7\u00e3o de Bateria: LIGADO (~80%)"}"#;
        assert_eq!(payload.dumps_ascii(), expected);
    }

    #[test]
    fn escapes_control_quotes_and_astral() {
        let payload = WaybarJson::new(vec![
            ("text", "a\"b\\c\nd\te".to_string()),
            ("tooltip", "🌕 x".to_string()),
        ]);
        // `"` → `\"`, `\` → `\\`, LF/TAB escapes, emoji → surrogate pair.
        assert_eq!(
            payload.dumps_ascii(),
            r#"{"text": "a\"b\\c\nd\te", "tooltip": "\ud83c\udf15 x"}"#
        );
    }

    #[test]
    fn value_preserves_order_as_map() {
        let payload = WaybarJson::new(vec![
            ("text", "x".to_string()),
            ("class", "y".to_string()),
        ]);
        let value = payload.value();
        assert_eq!(value["text"], serde_json::json!("x"));
        assert_eq!(value["class"], serde_json::json!("y"));
    }
}
