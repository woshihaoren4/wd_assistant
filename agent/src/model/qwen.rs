use crate::model::{Message, MessageType, ModelConfig, Response};
use crate::utils;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use wd_tools::AsBytes;

const QWEN_CHAT_PATH: &'static str =
    "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions";
const DASHSCOPE_API_KEY: &'static str = "DASHSCOPE_API_KEY";

#[derive(Debug,Clone)]
pub struct QwenModel {
    api_key: String,
}
impl Default for QwenModel {
    fn default() -> Self {
        let api_key = std::env::var(DASHSCOPE_API_KEY).unwrap_or("".to_string());
        Self::new(api_key)
    }
}

impl QwenModel {
    pub fn new<S: Into<String>>(key: S) -> Self {
        let api_key = key.into();
        Self { api_key }
    }
}

#[async_trait::async_trait]
impl super::Model for QwenModel {
    async fn chat(&self, cfg: &ModelConfig, msg: &[Message]) -> anyhow::Result<Response> {
        let resp = Response::default();
        let sse = resp.clone();
        let auth_key = format!("Bearer {}", self.api_key);
        let req_body = QwenChatRequest::from((cfg, msg)).to_string();
        utils::sse(
            reqwest::Method::POST,
            QWEN_CHAT_PATH,
            |buidler| {
                buidler
                    .header("Content-Type", "application/json")
                    .header("Authorization", auth_key)
                    .body(req_body)
            },(),
            move|_,x| {
                let mut sse = sse.sender.clone();
                async move {
                    let msg = match x {
                        Ok(o) => o,
                        Err(e) => {
                            if let Err(err) = sse.send(Err(e)).await {
                                wd_log::log_field("error", err)
                                    .error("QwenModel.stream_handle.send error failed");
                            }
                            return false;
                        }
                    };
                    if msg.is_empty() {
                        return true
                    }
                    if msg.as_str() == "data: [DONE]"{
                        if let Err(err) = sse.send(Ok(Message::default())).await {
                            wd_log::log_field("error", err).error(
                                "CozeModel.stream_handle.send over send none msg",
                            );
                        }
                        return false
                    }
                    let delta = match serde_json::from_slice::<QwenStreamResponse>(
                        &msg.as_bytes()[6..],
                    ) {
                        Ok(o) => o,
                        Err(_e) => {
                            if let Err(err) = sse.send(Err(anyhow::anyhow!("{msg}"))).await {
                                wd_log::log_field("error", err).error(
                                    "CozeModel.stream_handle.send parse delta message error",
                                );
                            }
                            return false;
                        }
                    };
                    for i in delta.choices {
                        if i.delta.content.is_empty() {
                            continue
                        }
                        if let Err(err) = sse.send(Ok(Message::new_assistant(i.delta.content))).await {
                            wd_log::log_field("error", err)
                                .error("QwenModel.stream_handle.send delta failed");
                            return false;
                        }
                    }
                    true
                }
            },
        ).await?;

        Ok(resp)
    }
}
#[derive(Debug, Default, Serialize)]
struct QwenMsg {
    role: String,
    content: String,
}
impl From<&Message> for QwenMsg {
    fn from(value: &Message) -> Self {
        Self {
            role: value.role.to_string(),
            content: value.content.clone(),
        }
    }
}
#[derive(Debug, Default, Serialize)]
struct QwenChatRequest {
    model: String,
    messages: Vec<QwenMsg>,
    stream: bool,
    temperature: f32,
    top_p: f32,
    max_tokens: usize,
}
impl Display for QwenChatRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap();
        write!(f, "{s}")
    }
}
impl From<(&ModelConfig, &[Message])> for QwenChatRequest {
    fn from((cfg, msg): (&ModelConfig, &[Message])) -> Self {
        let messages = msg.iter().map(|x| QwenMsg::from(x)).collect::<Vec<_>>();
        Self {
            model: cfg.name.clone(),
            messages,
            stream: cfg.stream.clone(),
            temperature: cfg.temperature,
            top_p: cfg.top_p,
            max_tokens: cfg.max_output_token,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct QwenStreamResponse {
    id: String,
    choices: Vec<QwenResponseDelta>,
}
#[derive(Debug, Default, Deserialize)]
struct QwenResponseDelta {
    delta : QwenDeltaMsg
}
#[derive(Debug, Default, Clone, Deserialize)]
pub struct QwenDeltaMsg {
    #[serde(default="Default::default")]
    pub role: String,
    pub content: String,
}

#[cfg(test)]
mod test {
    use crate::model::{ChatHistory, Model, ModelConfig};
    use crate::model::qwen::QwenModel;

    #[tokio::test]
    async fn test_qwen_model() {
        let cfg = ModelConfig::default()
            .set_temperature(0.7)
            .set_name("qwen-turbo");

        let history: Vec<_> = ChatHistory::system("你是一个rust编程小助手")
            .user("你是谁？")
            .into();

        let mut resp = QwenModel::default()
            .chat(&cfg, history.as_slice())
            .await
            .expect("chat failed");

        while let Ok(msg) = resp.next().await {
            println!("--> {:?}", msg);
        }
        println!("---> success <---")
    }
}
