use crate::model::{Message, MessageType, ModelConfig, Response};
use crate::utils;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use wd_tools::{AsBytes, PFErr, PFSome};

const COZE_ACCESS_TOKEN: &'static str = "COZE_ACCESS_TOKEN";
const COZE_V3_CHAT_PATH: &'static str = " https://api.coze.cn/v3/chat";

#[derive(Debug,Clone)]
pub struct CozeModel {
    pub api_key: String,
}
impl Default for CozeModel {
    fn default() -> Self {
        let api_key = std::env::var(COZE_ACCESS_TOKEN).unwrap_or("".to_string());
        Self::new(api_key)
    }
}

impl CozeModel {
    pub fn new<S: Into<String>>(key: S) -> Self {
        let api_key = key.into();
        Self { api_key }
    }
    pub fn sse_stream_response_process(is_delta_msg:&mut bool,line:anyhow::Result<String>)->anyhow::Result<(bool,Option<Message>)>{
        let content = line?;
        if content.as_str() == "data:\"[DONE]\"" {
            return Ok((false,Message::default().some()))
        }
        if !*is_delta_msg {
            if content.as_str() == "event:conversation.message.delta" {
                *is_delta_msg = true;
            }
            return Ok((true,None))
        }
        *is_delta_msg = false;
        let delta = serde_json::from_slice::<CozeResponseDelta>(&content.as_bytes()[5..])?;
        if delta.code != 0 {
                return anyhow::anyhow!("{content}").err();
        }
        Ok((true,Some(delta.into())))

    }
}

#[async_trait::async_trait]
impl super::Model for CozeModel {
    async fn chat(&self, cfg: &ModelConfig, msg: &[Message]) -> anyhow::Result<Response> {
        if self.api_key.is_empty() {
            return anyhow::anyhow!("coze api is null, please set env[COZE_ACCESS_TOKEN]").err();
        }

        let resp = Response::default();
        let sse = resp.clone();
        let mut is_delta_msg = false;
        let body = CozeRequest::from((cfg, msg));
        let auth_key = format!("Bearer {}", self.api_key);

        utils::sse(
            Method::POST,
            COZE_V3_CHAT_PATH,
            |rb| {
                rb.header("Content-Type", "application/json")
                    .header("Authorization", auth_key)
                    .body(body.to_string())
            },is_delta_msg,
            move |mut is_delta_msg, line| {
                let sse = sse.sender.clone();
                let result = Self::sse_stream_response_process(is_delta_msg,line);
                async move {
                    let (cont,msg) = match result {
                        Ok(o) => o,
                        Err(e) => {
                            if let Err(err) = sse.send(Err(anyhow::Error::from(e))).await {
                                wd_log::log_field("error", err).error(
                                    "CozeModel.stream_handle.send parse delta message error",
                                );
                            }
                            return false
                        }
                    };
                    if let Some(s) = msg {
                        if let Err(err) = sse.send(Ok(s)).await {
                            wd_log::log_field("error", err).error(
                                "CozeModel.stream_handle.send a delta message error",
                            );
                            return false
                        }
                    }
                    return cont
                }
            },
        )
        .await?;

        Ok(resp)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CozeMessage {
    role: String,
    content: String,
    /// - text：文本。
    /// - object_string：多模态内容，即文本和文件的组合、文本和图片的组合。
    /// - card：卡片。此枚举值仅在接口响应中出现，不支持作为入参。
    content_type: String,
}
impl From<&Message> for CozeMessage {
    fn from(value: &Message) -> Self {
        CozeMessage {
            role: value.role.to_string(),
            content: value.content.to_string(),
            content_type: "text".to_string(),
        }
    }
}
#[derive(Debug, Default, Serialize)]
pub struct CozeRequest {
    bot_id: String,
    user_id: String,
    stream: bool,
    auto_save_history: bool,
    additional_messages: Vec<CozeMessage>,
}
impl Display for CozeRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap();
        write!(f, "{s}")
    }
}
impl From<(&ModelConfig, &[Message])> for CozeRequest {
    fn from((cfg, ms): (&ModelConfig, &[Message])) -> Self {
        let additional_messages = ms.iter().map(|x| CozeMessage::from(x)).collect::<Vec<_>>();
        let user_id = cfg
            .extend
            .get("user_id")
            .map(|i| i.to_string())
            .unwrap_or("default".to_string());
        CozeRequest {
            bot_id: cfg.name.clone(),
            user_id,
            stream: true,
            auto_save_history: false,
            additional_messages,
        }
    }
}
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct CozeResponseDelta {
    #[serde(default="Default::default")]
    code: i32,
    msg: String,
    role: String,
    #[serde(rename = "type")]
    ty: String,
    content: String,
    content_type: String,
}

impl Into<Message> for CozeResponseDelta {
    fn into(self) -> Message {
        Message {
            role: MessageType::from(self.role.as_str()),
            content: self.content,
            call_id: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::model::coze::CozeModel;
    use crate::model::{ChatHistory, Model, ModelConfig};
    use std::collections::VecDeque;

    #[tokio::test]
    async fn test_coze_model() {
        let cfg = ModelConfig::default()
            .set_temperature(0.7)
            .set_name("7370540535557898252")
            .append_extend("user_id", "teshin");

        let history: Vec<_> = ChatHistory::default().user("你是谁？").into();

        let mut resp = CozeModel::default()
            .chat(&cfg, history.as_slice())
            .await
            .expect("chat failed");

        while let res = resp.next().await {
            println!("-=-=>{:?}", res);
            if let Ok(o)=res{
                if o.is_over() {
                    break
                }
            }
        }
        println!("---> success <---")
    }
}
