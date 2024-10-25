use std::future::Future;
use tokio::runtime::Runtime;

#[wd_macro::global]
pub struct AsyncRT {
    rt: Runtime,
}
impl Default for AsyncRT {
    fn default() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build async runtime failed");
        Self { rt }
    }
}
impl AsyncRT {
    pub fn block_on<F: Future>(fut: F) -> F::Output {
        AsyncRT::unsafe_mut_ptr(|x| x.rt.block_on(async move { fut.await }))
    }
}
