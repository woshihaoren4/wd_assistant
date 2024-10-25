use crate::agent::ChatRespStream;
use crate::model::{Message, Model, ModelConfig, Response};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::Arc;
use wd_tools::PFErr;
use wd_tools::sync::Am;

pub struct SingleAgent {
    //1:可用 2:回复中 3:终止回复
    status: Arc<AtomicI8>,
    pub prompt: String,
    pub model_config: ModelConfig,
    pub model: Box<dyn Model + Sync>,
    pub history: Arc<Am<VecDeque<Message>>>,
    pub max_history: usize,
}
impl SingleAgent {
    pub fn new<M:Model+Sync+'static>(model:M)->Self{
        Self{
            status:Arc::new(AtomicI8::new(1)),
            prompt:"".into(),
            model_config: Default::default(),
            model: Box::new(model),
            history: Arc::new(Am::new(VecDeque::new())),
            max_history: 30,
        }
    }
    pub fn cove_chat_history(mut self,msg_list:VecDeque<Message>)->Self{
        self.history = Arc::new(Am::new(msg_list));self
    }
    pub fn set_prompt<P:Into<String>>(mut self,prompt:P)->Self{
        self.prompt = prompt.into();self
    }
    pub fn set_max_history(mut self,max:usize)->Self{
        self.max_history = max;self
    }
    pub fn cove_model_config<C:Into<ModelConfig>>(mut self,cfg:C)->Self{
        self.model_config =cfg.into();self
    }
    pub fn set_model_config(mut self,handle:impl FnOnce(&mut ModelConfig))->Self{
        handle(&mut self.model_config);self
    }
    pub fn status_is_usable(&self)->bool{
        self.status.load(Ordering::Relaxed) == 1
    }
    pub fn get_status(&self)->i8{
        self.status.load(Ordering::Relaxed)
    }
}
struct ChatHistoryWatch {
    status: Arc<AtomicI8>,
    history: Arc<Am<VecDeque<Message>>>,
}
impl ChatHistoryWatch {
    pub async fn watch(self, query: String, mut resp: Response, crs: ChatRespStream) {
        self.status.store(2, Ordering::Relaxed);
        let mut lock = self.history.lock().await;
        lock.push_back(Message::new_user(query));
        drop(lock);
        tokio::spawn(async move {
            let mut over = false;
            let mut res = String::new();
            while let result = resp.next().await {
                match result {
                    Ok(o) => {
                        over = o.content.is_empty();
                        res.push_str(o.content.as_str());
                        crs.push(o.content);
                        if over {
                            break
                        }
                    }
                    Err(e) => {
                        crs.push_err(e);
                        let mut lock = self.history.lock().await;
                        let _ = lock.pop_back();
                        return;
                    }
                }
            }
            let mut lock = self.history.lock().await;
            if self.status.load(Ordering::Relaxed) == 3 {
                let _ = lock.pop_back();
            } else {
                lock.push_back(Message::new_assistant(res))
            }
        });
    }
}
impl From<&SingleAgent> for ChatHistoryWatch {
    fn from(value: &SingleAgent) -> Self {
        Self {
            status: value.status.clone(),
            history: value.history.clone(),
        }
    }
}
impl Drop for ChatHistoryWatch {
    fn drop(&mut self) {
        self.status.store(1, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl super::Agent for SingleAgent {
    async fn chat(&self, query: String) -> anyhow::Result<ChatRespStream> {
        //检查状态
        if !self.status_is_usable() {
            return anyhow::anyhow!("SingleAgent.status check failed,please wait status usable").err()
        }
        //组装请求
        let mut chat_history = VecDeque::new();
        if self.max_history > 0 {
            let lock = self.history.synchronize();
            for (index, msg) in lock.iter().rev().enumerate() {
                if index == self.max_history {
                    break;
                }
                chat_history.push_front(msg.clone());
            }
            drop(lock);
        }
        if !self.prompt.is_empty() {
            chat_history.push_front(Message::new_system(self.prompt.as_str()));
        };
        chat_history.push_back(Message::new_user(query.as_str()));

        let chat_history = chat_history.into_iter().collect::<Vec<_>>();

        //请求大脑
        let resp = self
            .model
            .chat(&self.model_config, chat_history.as_slice())
            .await?;
        let crs = ChatRespStream::new();

        //记忆
        ChatHistoryWatch::from(self)
            .watch(query, resp, crs.clone())
            .await;

        Ok(crs)
    }

    async fn clear_chat_history(&self) {
        todo!()
    }

    async fn save(&self) -> String {
        todo!()
    }

    async fn delete(&self) {
        todo!()
    }
}

#[cfg(test)]
mod test{
    use std::io::Write;
    use std::time::Duration;
    use crate::agent::Agent;
    use crate::agent::agent::SingleAgent;
    use crate::model::qwen::QwenModel;

    // cargo test --lib pkg::pkg::test::test_single_agent -- --nocapture
    #[tokio::test]
    async fn test_single_agent(){
        let agent = SingleAgent::new(QwenModel::default())
            .set_model_config(|cfg|cfg.name = "qwen-turbo".into())
            .set_prompt("## role: 你是一个rust编程小助手。")
            .set_max_history(10);
        print!("SYSTEM: {} \nUSER: ",agent.prompt);
        std::io::stdout().flush().unwrap();

        let mut buf = String::new();
        let stdin = std::io::stdin();
        while stdin.read_line(&mut buf).is_ok() {
            if buf.is_empty() {
                continue
            }
            let answer = agent.chat(buf.clone()).await.expect("pkg chat error:");
            buf = String::new();
            print!("ASSISTANT: ");
            std::io::stdout().flush().unwrap();
            while let stream = answer.next().expect("message stream error:") {
                if let Some(msg) = stream {
                    if msg.is_empty() {
                        print!("\nUSER: ");
                        std::io::stdout().flush().unwrap();
                        break
                    }else{
                        print!("{}",msg);
                        std::io::stdout().flush().unwrap();
                    }
                }else{
                    //fixme: 用ui刷新，不要忙等，这里仅做测试
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}