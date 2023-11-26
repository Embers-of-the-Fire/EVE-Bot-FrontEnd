use crate::server::MessageData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PostType {
    #[serde(rename = "message")]
    Message,
    #[serde(rename = "message_sent")]
    MessageSent,
    #[serde(rename = "request")]
    Request,
    #[serde(rename = "notice")]
    Notice,
    #[serde(rename = "meta_event")]
    MetaEvent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "notice")]
    Notice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: u64,
    pub nickname: String,
    pub level: Option<String>,
    pub role: Option<GroupMemberRole>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GroupMemberRole {
    #[serde(rename = "owner")]
    Owner,
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "member")]
    Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub time: u64,
    pub self_id: u64,
    pub post_type: PostType,
    pub sub_type: MessageType,
    pub message_id: u32,
    pub user_id: u64,
    pub message: MessageData,
    pub raw_message: String,
    pub group_id: Option<u64>,
}
