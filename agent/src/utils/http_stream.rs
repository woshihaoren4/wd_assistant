use bytes::Bytes;
use reqwest::{Method, RequestBuilder};
use std::future::Future;
use wd_tools::AsBytes;

pub async fn sse<F: Future<Output = bool> + Send,CTX:Send+'static>(
    method: Method,
    url: &str,
    builder: impl FnOnce(RequestBuilder) -> RequestBuilder,
    mut ctx:CTX,
    stream_handle: impl Fn(&mut CTX,anyhow::Result<String>) -> F + Send + 'static,
) -> anyhow::Result<()> {
    let mut req = reqwest::Client::new().request(method, url);
    req = builder(req);
    let mut resp = req.send().await?;
    tokio::spawn(async move {
        while let result = resp.chunk().await {
            let opt = match result {
                Ok(o) => o,
                Err(e) => {
                    stream_handle(&mut ctx,Err(anyhow::Error::from(e))).await;
                    break;
                }
            };

            let resp_str = if let Some(bytes) = opt {
                String::from_utf8_lossy(bytes.as_byte()).to_string()
                // if !stream_handle(Ok(bytes)).await {
                //     break;
                // }
            } else {
                stream_handle(&mut ctx,Ok(String::new())).await;
                break;
            };
            let list = resp_str.split("\n").collect::<Vec<&str>>();
            for i in list {
                if !stream_handle(&mut ctx,Ok(i.to_string())).await {
                    return
                }
            }
        }
    });
    Ok(())
}
