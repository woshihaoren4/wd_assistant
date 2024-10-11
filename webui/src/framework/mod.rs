mod adsorb_window;
mod chat_window;
mod floating_window;

use crate::config::{Config, WindowMode};
use crate::framework::adsorb_window::AdsorbWindow;
use crate::framework::chat_window::ChatWindow;
use crate::framework::floating_window::FloatingWindow;
use eframe::egui::Context;
use eframe::{egui, App, Frame};

pub trait Window {
    fn init(&mut self, ctx: &egui::Context, cfg: &mut Config) {}
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame, cfg: &mut Config);
}

pub struct WdApp {
    pub cfg: Config,
    pub chat_window: Box<dyn Window>,
    pub floating_window: Box<dyn Window>,
    pub adsorb: Box<dyn Window>,
}

impl Default for WdApp {
    fn default() -> Self {
        WdApp {
            cfg: Default::default(),
            chat_window: Box::new(ChatWindow::default()),
            floating_window: Box::new(FloatingWindow::default()),
            adsorb: Box::new(AdsorbWindow::default()),
        }
    }
}

impl App for WdApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        match self.cfg.memory_cfg.window_mode {
            WindowMode::CHAT => {
                if self.cfg.memory_cfg.check_window_mode_change() {
                    self.chat_window.init(ctx, &mut self.cfg);
                }
                self.chat_window.update(ctx, frame, &mut self.cfg);
            }
            WindowMode::FLOATING => {
                if self.cfg.memory_cfg.check_window_mode_change() {
                    self.floating_window.init(ctx, &mut self.cfg);
                }
                self.floating_window.update(ctx, frame, &mut self.cfg);
            }
            WindowMode::ADSORB => {
                if self.cfg.memory_cfg.check_window_mode_change() {
                    self.adsorb.init(ctx, &mut self.cfg);
                }
                self.adsorb.update(ctx, frame, &mut self.cfg);
            }
        }
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}
