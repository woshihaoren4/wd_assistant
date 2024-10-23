use crate::model::coze::CozeModel;
use crate::model::{Message, Model, ModelConfig, Response};
use std::collections::HashMap;
use wd_tools::PFBox;
use crate::model::qwen::QwenModel;

pub const GLOBAL_MODEL_COZE: &'static str = "GLOBAL_MODEL_COZE";
pub const GLOBAL_MODEL_QWEN: &'static str = "GLOBAL_MODEL_QWEN";

#[derive(Default)]
#[wd_macro::global]
pub struct GlobalModel {
    models: HashMap<String, Box<dyn Model+Sync>>,
}

// impl Default for GlobalModel{
//     fn default() -> Self {
//         let mut this = Self{
//             models : HashMap::new()
//         };
//         this.models.insert(GLOBAL_MODEL_COZE.into(), );
//         this.models.insert(GLOBAL_MODEL_QWEN.into(), );
//         this
//     }
// }

impl GlobalModel {
    pub fn default_model(mode_type:&str)->Box<dyn Model+Sync>{
        match mode_type {
            GLOBAL_MODEL_COZE=> CozeModel::default().to_box(),
            GLOBAL_MODEL_QWEN=> QwenModel::default().to_box(),
            _=>{
                panic!("unknown mode type")
            }
        }
    }
}