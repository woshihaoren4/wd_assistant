use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use wd_tools::sync::Am;

mod agent;
mod builder;

#[derive(Debug, Clone)]
pub struct ChatRespStream {
    chan: Arc<Am<VecDeque<anyhow::Result<String>>>>,
}
impl ChatRespStream {
    pub fn new() -> Self {
        let chan = Arc::new(Am::new(VecDeque::new()));
        Self { chan }
    }
    pub fn next(&self) -> anyhow::Result<Option<String>> {
        let mut fut = self.chan.synchronize();
        match fut.pop_back() {
            None => Ok(None),
            Some(s) => {
                let r = s?;
                Ok(Some(r))
            }
        }
    }
    pub fn push<S: Into<String>>(&self, msg: S) {
        let mut fut = self.chan.synchronize();
        fut.push_front(Ok(msg.into()));
    }
    pub fn push_err(&self, err: anyhow::Error) {
        let mut fut = self.chan.synchronize();
        fut.push_front(Err(err))
    }
}

#[async_trait::async_trait]
pub trait Agent {
    async fn chat(&self, query: String) -> anyhow::Result<ChatRespStream>;
    async fn clear_chat_history(&self);
    async fn save(&self) -> String;
    async fn delete(&self);
}
