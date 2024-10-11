use std::ptr;

#[derive(Default, Eq, PartialEq, Clone)]
pub enum WindowMode {
    #[default]
    CHAT,
    FLOATING,
    ADSORB,
}

#[derive(Default)]
pub struct MemoryConfig {
    pub window_mode: WindowMode,
    pub last_window_mode: WindowMode,
}

impl MemoryConfig {
    //是否切换了窗口
    pub fn check_window_mode_change(&mut self) -> bool {
        let result = self.window_mode != self.last_window_mode;
        if result {
            self.last_window_mode = self.window_mode.clone();
        }
        result
    }

    //检查窗口模式
    pub fn check_window_mode(&mut self, mode: WindowMode) -> bool {
        self.window_mode == mode
    }

    //切换窗口-CHAT
    pub fn chat_window_mode_to_chat(&mut self) {
        self.last_window_mode = self.window_mode.clone();
        self.window_mode = WindowMode::CHAT;
    }
    //切换窗口-FLOATING
    pub fn chat_window_mode_to_floating(&mut self) {
        self.last_window_mode = self.window_mode.clone();
        self.window_mode = WindowMode::FLOATING;
    }
    //切换窗口-ADSORB
    pub fn chat_window_mode_to_adsorb(&mut self) {
        self.last_window_mode = self.window_mode.clone();
        self.window_mode = WindowMode::ADSORB;
    }
}
