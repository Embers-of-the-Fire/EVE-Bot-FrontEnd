use crate::error::{BotError, BotResult};
use crate::metadata::LOCAL_EVE_SERVICE_PORT;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

pub fn random_filename() -> String {
    let uuid = Uuid::new_v4();
    uuid.hyphenated().to_string()
}

pub async fn download_image(resp: Response, fs: &Path) -> BotResult<()> {
    let mut file = fs::File::create(fs).map_err(<std::io::Error as Into<BotError>>::into)?;
    let bytes = resp
        .bytes()
        .await
        .map_err(<reqwest::Error as Into<BotError>>::into)?;
    file.write_all(&bytes)
        .map_err(<std::io::Error as Into<BotError>>::into)?;
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct TypeItem {
    pub type_id: usize,
    pub type_name: String,
    #[serde(default)]
    pub published: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeIDFetch<T: AsRef<str>> {
    Absolute(T),
    Fuzzy(T),
    Manual(T),
}

impl<T: AsRef<str> + std::fmt::Debug> TypeIDFetch<T> {
    pub async fn get(&self) -> BotResult<TypeItem> {
        match self {
            TypeIDFetch::Absolute(t) => {
                reqwest::ClientBuilder::new()
                    .build()?
                    .get(format!(
                        "http://localhost:{}/types/search/absolute/?name={}",
                        LOCAL_EVE_SERVICE_PORT,
                        t.as_ref()
                    ))
                    .send()
                    .await?
                    .json()
                    .await
            }
            TypeIDFetch::Fuzzy(t) => {
                reqwest::ClientBuilder::new()
                    .build()?
                    .get(format!(
                        "http://localhost:{}/types/search/fuzzy/?name={}",
                        LOCAL_EVE_SERVICE_PORT,
                        t.as_ref()
                    ))
                    .send()
                    .await?
                    .json()
                    .await
            }
            TypeIDFetch::Manual(p) => {
                reqwest::ClientBuilder::new()
                    .build()?
                    .get(format!(
                        "http://localhost:{}/types/search/manual/?pattern={}",
                        LOCAL_EVE_SERVICE_PORT,
                        p.as_ref()
                    ))
                    .send()
                    .await?
                    .json()
                    .await
            }
        }
        .map_err(|e| e.into())
    }

    pub fn type_from(val: impl AsRef<str>) -> Box<dyn FnOnce(T) -> BotResult<Self>> {
        match val.as_ref().to_ascii_lowercase().as_str() {
            "absolute" | "a" | "abs" => Box::new(|v| Ok(Self::Absolute(v))),
            "fuzzy" | "f" | "fuz" | "fuzz" => Box::new(|v| Ok(Self::Fuzzy(v))),
            "manual" | "m" | "man" => Box::new(|v| Ok(Self::Manual(v))),
            _ => {
                let val = val.as_ref().to_owned();
                Box::new(move |_| {
                    Err(BotError::Syntax {
                        found: Some(val),
                        expected: Some("choices: [a]bsolute, [f]uzzy, [m]anual.".to_string()),
                        note: Some("use a valid fetch pattern type.".to_string()),
                    })
                })
            }
        }
    }
}

/// # Syntax
///
/// ```
/// fetch_type! {
///     pattern: pattern_value,
///     type_name: type_name_value,
///     error: error_ident,
/// }
/// ```
#[macro_export]
macro_rules! fetch_type {
    {pattern: $pat: expr, type_name: $tn: expr, error: $err: ident $(,)?} => {
        match $crate::utils::fetch::TypeIDFetch::type_from($pat)($tn) {
            Ok(data) => match data.get().await {
                Ok(val) => Some(val),
                Err(err) => {
                    $err.push(err);
                    None
                }
            },
            Err(err) => {
                $err.push(err);
                None
            }
        }
    };
}
