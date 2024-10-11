mod memory_config;

pub use memory_config::*;

#[derive(Default)]
pub struct Config {
    pub memory_cfg: MemoryConfig,
}
