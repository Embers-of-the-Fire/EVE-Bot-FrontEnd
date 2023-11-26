use crate::error::BotError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Server {
    Tranquility,
    #[default]
    Serenity,
}

impl Server {
    pub fn parse_from(s: impl AsRef<str>) -> Result<Server, BotError> {
        match s.as_ref().to_ascii_lowercase().as_str() {
            "tq" | "trans" | "tranquility" => Ok(Server::Tranquility),
            "se" | "seren" | "serenity" => Ok(Server::Serenity),
            _ => Err(BotError::Syntax {
                found: Some(s.as_ref().into()),
                expected: Some("tq/trans/tranquility / se/seren/serenity".into()),
                note: Some("不合法的服务器类型".into()),
            }),
        }
    }

    pub fn as_api_like(&self) -> &'static str {
        match self {
            Server::Tranquility => "tq",
            Server::Serenity => "se",
        }
    }

    pub fn as_readable(&self) -> &'static str {
        match self {
            Server::Tranquility => "宁静",
            Server::Serenity => "晨曦",
        }
    }
}
