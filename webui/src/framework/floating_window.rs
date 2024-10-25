use crate::config::Config;
use crate::pkg::AsyncRT;
use agent::{Agent, MessageType};
use eframe::egui::{
    CentralPanel, Context, Id, PointerButton, ScrollArea, Sense, SidePanel, TopBottomPanel, Ui,
    Vec2, ViewportCommand,
};
use eframe::{egui, Frame};

#[derive(Default)]
pub struct FloatingWindow {
    input: String,
}

impl FloatingWindow {
    fn show_assistant_info(&mut self, ctx: &Context, ui: &mut Ui, cfg: &mut Config) {
        ui.horizontal_top(|ui| {
            let status = cfg.memory_cfg.assistant.get_status();
            let status = format!("status:{status}");
            ui.label(status);
        });
        ui.separator();
        // TopBottomPanel::top("FloatingWindow.show_assistant_info.Top").show(ctx,|ui|{
        //
        // });
    }
    fn show_history(&mut self, ctx: &Context, ui: &mut Ui, cfg: &mut Config) {
        ScrollArea::vertical().show(ui, |ui| {
            //渲染system
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.label(format!("System: {}", cfg.memory_cfg.assistant.prompt));
            });
            //渲染历史消息
            let lock = cfg.memory_cfg.assistant.history.synchronize();
            for (_i, e) in lock.iter().enumerate() {
                match e.role {
                    MessageType::SYSTEM => {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(format!("System: \n{}", e.content)).color(egui::Color32::WHITE).background_color(egui::Color32::RED)).wrap_mode(egui::TextWrapMode::Wrap));
                        });
                    }
                    MessageType::User => {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(format!("User: \n{}", e.content)).color(egui::Color32::WHITE).background_color(egui::Color32::GREEN)).wrap_mode(egui::TextWrapMode::Wrap));
                        });
                    }
                    MessageType::Assistant => {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(format!("Assistant: \n{}", e.content)).color(egui::Color32::WHITE).background_color(egui::Color32::BLUE)).wrap_mode(egui::TextWrapMode::Wrap))
                        });
                    }
                    MessageType::TOOL => {}
                    MessageType::Unknown(_) => {}
                }
            }
            //渲染最新的消息
            if !cfg.memory_cfg.assistant_msg.is_empty() {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.add(egui::Label::new(egui::RichText::new(format!("Assistant: \n{}",cfg.memory_cfg.assistant_msg)).color(egui::Color32::WHITE).background_color(egui::Color32::BLUE)).wrap_mode(egui::TextWrapMode::Wrap))
                });
            }
        });
    }
    fn input(&mut self, ctx: &Context, ui: &mut Ui, cfg: &mut Config) {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min),|ui|{
            ui.horizontal(|ui| {
                let resp = ui.add(egui::TextEdit::multiline(&mut self.input));
                if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if self.input.is_empty() {
                        return;
                    }
                    cfg.memory_cfg.assistant_msg = String::new();
                    let input = std::mem::take(&mut self.input);
                    let fut = cfg.memory_cfg.assistant.chat(input);
                    match AsyncRT::block_on(fut) {
                        Ok(o) => cfg.memory_cfg.chat_stream_resp = Some(o),
                        Err(e) => {
                            cfg.memory_cfg.assistant_msg = e.to_string();
                        }
                    };
                }
            });
        });
        //刷新消息
        if let Some(ref mut resp) = cfg.memory_cfg.chat_stream_resp {
            match resp.next() {
                Ok(o) => {
                    if let Some(msg) = o {
                        if msg.is_empty() {
                            cfg.memory_cfg.chat_stream_resp = None;
                            cfg.memory_cfg.assistant_msg = String::new();
                        } else {
                            cfg.memory_cfg.assistant_msg.push_str(msg.as_str());
                        }
                    }
                }
                Err(e) => {
                    cfg.memory_cfg.assistant_msg = e.to_string();
                }
            };
        }
        //强制刷新
        if !cfg.memory_cfg.assistant.status_is_usable() {
            ctx.request_repaint();
        }
    }
}

impl super::Window for FloatingWindow {
    fn init(&mut self, ctx: &Context, _cfg: &mut Config) {
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
        ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
        // ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(350.0, 700.0)));
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame, cfg: &mut Config) {
        CentralPanel::default().show(ctx, |ui| {
            //设置
            self.show_assistant_info(ctx, ui, cfg);
            //历史
            self.show_history(ctx, ui, cfg);
            //输入
            self.input(ctx, ui, cfg);
            //按住移动
            // let app_rect = ui.max_rect();
            // let title_bar_response = ui.interact(
            //     app_rect,
            //     Id::new("FloatingWindow.update.CentralPanel.interact"),
            //     Sense::click_and_drag(),
            // );
            // if title_bar_response.drag_started_by(PointerButton::Primary) {
            //     ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            // }
            // ui.label("i am your window assistant.");
            //双击回到聊天窗口
            // if title_bar_response.clicked() {
            //     cfg.memory_cfg.chat_window_mode_to_chat();
            //     return;
            // }
        });
    }
}
