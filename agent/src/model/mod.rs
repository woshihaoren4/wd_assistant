pub mod coze;
pub mod define;
pub mod qwen;

use async_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_output_token: usize,
    pub stream: bool,

    pub extend: HashMap<String, String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            temperature: 1.0,
            top_p: 0.9,
            max_output_token: 512,
            stream: true,
            extend: Default::default(),
        }
    }
}
impl ModelConfig {
    pub fn set_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }
    pub fn set_stream_mode(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
    pub fn append_extend<K: Into<String>, V: Into<String>>(mut self, k: K, v: V) -> Self {
        self.extend.insert(k.into(), v.into());
        self
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    SYSTEM,
    #[default]
    User,
    Assistant,
    TOOL,
    Unknown(String),
}
impl From<&str> for MessageType {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "system" => MessageType::SYSTEM,
            "user" => MessageType::User,
            "assistant" => MessageType::Assistant,
            "tool" => MessageType::TOOL,
            x => MessageType::Unknown(x.to_string()),
        }
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::SYSTEM => write!(f, "system"),
            MessageType::User => write!(f, "user"),
            MessageType::Assistant => write!(f, "assistant"),
            MessageType::TOOL => write!(f, "tool"),
            MessageType::Unknown(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
}

impl Message {
    pub fn new<T: Into<MessageType>, C: Into<String>>(role: T, content: C) -> Message {
        Message {
            role: role.into(),
            content: content.into(),
            call_id: None,
        }
    }
    pub fn new_system<C: Into<String>>(content: C) -> Message {
        Message::new(MessageType::SYSTEM, content)
    }
    pub fn new_user<C: Into<String>>(content: C) -> Message {
        Message::new(MessageType::User, content)
    }
    pub fn new_assistant<C: Into<String>>(content: C) -> Message {
        Message::new(MessageType::Assistant, content)
    }
    pub fn is_over(&self) -> bool {
        self.content.is_empty()
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatHistory {
    list: Vec<Message>,
}

impl ChatHistory {
    pub fn system<S: Into<String>>(input: S) -> Self {
        let list = vec![Message::new("system", input)];
        Self { list }
    }
    pub fn user<S: Into<String>>(mut self, input: S) -> Self {
        self.list.push(Message::new("user", input));
        self
    }
    pub fn assistant<S: Into<String>>(mut self, input: S) -> Self {
        self.list.push(Message::new("assistant", input));
        self
    }
}
impl From<ChatHistory> for Vec<Message> {
    fn from(value: ChatHistory) -> Self {
        value.list
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    sender: Sender<anyhow::Result<Message>>,
    receiver: Receiver<anyhow::Result<Message>>,
}
impl Response {
    pub async fn next(&mut self) -> anyhow::Result<Message> {
        self.receiver.recv().await?
    }
    pub async fn push(&mut self, msg: anyhow::Result<Message>) -> anyhow::Result<()> {
        self.sender.send(msg).await?;
        Ok(())
    }
}
impl Drop for Response {
    fn drop(&mut self) {
        self.receiver.close();
    }
}

impl Default for Response {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { sender, receiver }
    }
}

#[async_trait::async_trait]
pub trait Model: Send {
    async fn chat(&self, cfg: &ModelConfig, msg: &[Message]) -> anyhow::Result<Response>;
}
