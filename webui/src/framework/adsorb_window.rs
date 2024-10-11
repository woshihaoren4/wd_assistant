use crate::config::Config;
use eframe::egui::{CentralPanel, Context, Id, Image, PointerButton, Sense, Vec2, ViewportCommand, Widget};
use eframe::{egui, Frame};

#[derive(Default)]
pub struct AdsorbWindow {}

impl super::Window for AdsorbWindow {
    fn init(&mut self, ctx: &Context, _cfg: &mut Config) {
        ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(60.0, 60.0)));
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame, cfg: &mut Config) {

        let panel_frame = egui::Frame {
            fill: egui::Color32::TRANSPARENT,
            rounding: 0.0.into(),
            stroke:ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.0.into(),
            ..Default::default()
        };
        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            //按住移动
            let app_rect = ui.max_rect();
            let title_bar_response = ui.interact(
                app_rect,
                Id::new("AdsorbWindow.update.CentralPanel.interact"),
                Sense::click_and_drag(),
            );
            if title_bar_response.drag_started_by(PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }
            //双击回到聊天窗口
            if title_bar_response.clicked() {
                cfg.memory_cfg.chat_window_mode_to_chat();
                return;
            }
            Image::new(egui::include_image!("../.././resource/wd_assistant_v3.jpg")).max_size(Vec2::new(100.0, 100.0)).ui(ui);
            // ui.label("hello");
        });
    }
}
