use crate::config::const_config::CHAT_WINDOW_INIT_SIZE;
use crate::config::{Config, WindowMode};
use eframe::egui::{
    CentralPanel, CollapsingHeader, Context, Pos2, Rect, SidePanel, TopBottomPanel, Vec2,
    ViewportCommand,
};
use eframe::Frame;

#[derive(Default)]
pub struct ChatWindow {}

impl super::Window for ChatWindow {
    fn init(&mut self, ctx: &Context, _cfg: &mut Config) {
        let mut pos = ctx.viewport(|v| v.input.raw.viewport().outer_rect.unwrap_or(Rect::ZERO).max);
        pos.x -= CHAT_WINDOW_INIT_SIZE.0;
        pos.y -= ctx.screen_rect().height();
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
        ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::from(
            CHAT_WINDOW_INIT_SIZE,
        )));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        // ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame, cfg: &mut Config) {
        TopBottomPanel::top("ChatWindow.top")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("floating_mode").clicked() {
                        cfg.memory_cfg.chat_window_mode_to_floating();
                    }
                    if ui.button("adsorb_model").clicked() {
                        cfg.memory_cfg.chat_window_mode_to_adsorb();
                    }
                });
            });
        if !cfg.memory_cfg.check_window_mode(WindowMode::CHAT) {
            return;
        }
        SidePanel::left("ChatWindow.left").show(ctx, |ui| {
            CollapsingHeader::new("item 1")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label("sub item 1.1");
                    ui.separator();
                    ui.label("sub item 1.2");
                });
            CollapsingHeader::new("item 2")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label("sub item 2.1");
                    ui.separator();
                    ui.label("sub item 2.2");
                    ui.separator();
                    ui.label("sub item 2.3");
                });
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.label("hello world");
        });
        TopBottomPanel::bottom("ChatWindow.bottom")
            .exact_height(20.0)
            .show(ctx, |ui| {
                ui.label("debug")
                    .on_hover_text("this is a debug info show window");
            });
    }
}
