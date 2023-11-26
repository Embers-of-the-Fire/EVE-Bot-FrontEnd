mod blp;
pub mod dis;
mod market;
mod skills;
mod type_item;

use crate::error::BotError;
use serde_json::json;

#[allow(dead_code)]
pub fn generic_response() -> Result<serde_json::Value, BotError> {
    Ok(json! {[{
        "type": "text",
        "data": {
            "text": "Hello, world!"
        }
    }]})
}
