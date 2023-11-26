use crate::command;
use crate::server::data::GroupMessage;
use actix_web::web;
use actix_web::{HttpResponse, Result};
use serde_json::json;

pub async fn main_handler(payload: web::Payload) -> Result<HttpResponse> {
    let data = payload.to_bytes().await?;
    let json_data: serde_json::Value = serde_json::from_slice(&data)?;
    if json_data.get("post_type") == Some(&json!("message")) {
        group_message_handler(serde_json::from_slice(&data)?).await
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

async fn group_message_handler(data: GroupMessage) -> Result<HttpResponse> {
    let msg = data.message.into_messages();
    if let Some(msg) = msg {
        match command::distributor::dis::distribute(msg).await {
            None => Ok(HttpResponse::NoContent().finish()),
            Some(Err(err)) => Ok(HttpResponse::Ok().json(json! {{
                "at_sender": false,
                "reply": format!("{}", err)
            }})),
            Some(Ok(resp)) => Ok(HttpResponse::Ok().json(json! {{
                "at_sender": false,
                "reply": resp,
            }})),
        }
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}
