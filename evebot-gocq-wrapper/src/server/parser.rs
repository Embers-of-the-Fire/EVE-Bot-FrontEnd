use serde::{Deserialize, Serialize};
use serde_json::Value as jsv;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData(pub serde_json::Value);

impl Deref for MessageData {
    type Target = jsv;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MessageData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MessageData {
    pub fn into_messages(self) -> Option<MessageIter> {
        if let jsv::Array(arr) = self.0 {
            Some(MessageIter {
                message: arr.into_iter(),
                cache_string: None,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageIter {
    message: std::vec::IntoIter<serde_json::Value>,
    cache_string: Option<(Vec<char>, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ParamItem {
    Text(String),
    At(u64),
}

impl std::fmt::Display for ParamItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "Text[{}]", t),
            Self::At(v) => write!(f, "@{}", v),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BracketType {
    Brace,
    Parenthesis,
    Bracket,
}

impl BracketType {
    pub fn match_open(ch: &char) -> Option<Self> {
        match ch {
            '{' | '｛' => Some(Self::Brace),
            '[' | '【' => Some(Self::Bracket),
            '(' | '（' => Some(Self::Parenthesis),
            _ => None,
        }
    }

    pub fn match_close(ch: &char) -> Option<Self> {
        match ch {
            '}' | '｝' => Some(Self::Brace),
            ']' | '】' => Some(Self::Bracket),
            ')' | '）' => Some(Self::Parenthesis),
            _ => None,
        }
    }
}

#[inline(always)]
fn is_whitespace(ch: &char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

#[test]
fn test_msg_iter() {
    use super::ParamItem::*;
    use serde_json::json;
    let json_val = json! {[
        {
            "type": "at",
            "data": {
                "qq": 123456
            }
        },
        {
            "type": "text",
            "data": {
                "text": "  早上好啊 13 (1a  nd}[[c）123"
            }
        },
        {
            "type": "at",
            "data": {
                "qq": 123777
            }
        },
    ]};
    for message in MessageData(json_val.clone()).into_messages().unwrap() {
        println!("{:?}", message);
    }
    assert_eq!(
        MessageData(json_val)
            .into_messages()
            .unwrap()
            .collect::<Vec<_>>(),
        vec![
            At(123456),
            Text("早上好啊".into()),
            Text("13".into()),
            Text("1a  nd}[[c".into()),
            Text("123".into()),
            At(123777)
        ]
    )
}

impl Iterator for MessageIter {
    type Item = ParamItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((ref cache_string, ref mut ptr)) = self.cache_string {
            while *ptr < cache_string.len() && is_whitespace(&cache_string[*ptr]) {
                *ptr += 1;
            }
            if *ptr + 1 >= cache_string.len() {
                self.cache_string = None;
                return self.next();
            }
            if let Some(bracket) = BracketType::match_open(&cache_string[*ptr]) {
                let mut text = String::new();
                while *ptr < cache_string.len() {
                    *ptr += 1;
                    if BracketType::match_close(&cache_string[*ptr]).is_some_and(|v| v == bracket) {
                        *ptr += 1;
                        return Some(ParamItem::Text(text));
                    }
                    text.push(cache_string[*ptr]);
                }
                self.cache_string = None;
                Some(ParamItem::Text(text))
            } else {
                let mut text = String::new();
                while *ptr < cache_string.len()
                    && BracketType::match_open(&cache_string[*ptr]).is_none()
                    && !is_whitespace(&cache_string[*ptr])
                {
                    text.push(cache_string[*ptr]);
                    *ptr += 1;
                }
                if *ptr + 1 == cache_string.len() {
                    self.cache_string = None;
                }
                Some(ParamItem::Text(text))
            }
        } else {
            let nxt = self.message.next()?;
            let json_type = nxt.get("type")?;
            let json_data = nxt.get("data")?;
            match json_type {
                jsv::String(text) if text == "at" => json_data
                    .get("qq")
                    .and_then(|v| v.as_u64())
                    .map(ParamItem::At),
                jsv::String(text) if text == "text" => json_data
                    .get("text")
                    .and_then(|s| {
                        if let jsv::String(string) = s {
                            Some(string.to_owned())
                        } else {
                            None
                        }
                    })
                    .and_then(|s| {
                        self.cache_string = Some((s.chars().collect(), 0));
                        self.next()
                    }),
                _ => None,
            }
        }
    }
}
