use crate::config::Config;
use eframe::egui::{CentralPanel, Context, Id, PointerButton, Sense, Vec2, ViewportCommand};
use eframe::Frame;

#[derive(Default)]
pub struct FloatingWindow {}

impl super::Window for FloatingWindow {
    fn init(&mut self, ctx: &Context, _cfg: &mut Config) {
        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::Transparent(true));
        ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(200.0, 120.0)));
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame, cfg: &mut Config) {
        CentralPanel::default().show(ctx, |ui| {
            //按住移动
            let app_rect = ui.max_rect();
            let title_bar_response = ui.interact(
                app_rect,
                Id::new("FloatingWindow.update.CentralPanel.interact"),
                Sense::click_and_drag(),
            );
            if title_bar_response.drag_started_by(PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }
            ui.label("i am your window assistant.");
            //双击回到聊天窗口
            if title_bar_response.clicked() {
                cfg.memory_cfg.chat_window_mode_to_chat();
                return;
            }
        });
    }
}
