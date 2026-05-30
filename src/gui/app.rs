use eframe::egui;
use std::sync::mpsc::{self, Receiver, Sender};
use crate::core::{device, sideload};

pub struct ResignerApp {
    apple_id: String,
    password: String,
    ipa_path: String,
    status_msg: String,
    days_remaining: u8,
    is_processing: bool,
    tx: Sender<String>,
    rx: Receiver<String>,
}

impl ResignerApp {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            apple_id: String::new(),
            password: String::new(),
            ipa_path: String::new(),
            status_msg: "Awaiting input...".to_string(),
            days_remaining: 0,
            is_processing: false,
            tx,
            rx,
        }
    }
}

impl eframe::App for ResignerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for background task updates
        if let Ok(msg) = self.rx.try_recv() {
            self.status_msg = msg.clone();
            if msg.contains("Success") || msg.contains("Error") {
                self.is_processing = false;
                if msg.contains("Success") {
                    self.days_remaining = 7;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("iOS App Resigner");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Days Remaining:");
                ui.label(egui::RichText::new(self.days_remaining.to_string()).strong());
            });

            ui.add_space(10.0);
            ui.label("Apple ID:");
            ui.text_edit_singleline(&mut self.apple_id);
            
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));

            ui.label("IPA Path:");
            ui.text_edit_singleline(&mut self.ipa_path);

            ui.add_space(20.0);

            if ui.add_enabled(!self.is_processing, egui::Button::new("Trust Device & Resign")).clicked() {
                self.is_processing = true;
                self.status_msg = "Connecting to device...".to_string();

                let id = self.apple_id.clone();
                let pass = self.password.clone();
                let ipa = self.ipa_path.clone();
                let tx = self.tx.clone();

                // Spawn background task so the GUI doesn't freeze
                tokio::spawn(async move {
                    let device = match device::get_and_pair_device() {
                        Ok(d) => d,
                        Err(e) => {
                            let _ = tx.send(format!("Error: {}", e));
                            return;
                        }
                    };

                    let _ = tx.send(format!("Paired: {}. Signing app...", device.name));

                    match sideload::resign_and_install(&id, &pass, &ipa, &device.udid).await {
                        Ok(_) => { let _ = tx.send("Success! App installed on device.".into()); }
                        Err(e) => { let _ = tx.send(format!("Error: {}", e)); }
                    }
                });
            }

            ui.add_space(20.0);
            ui.label(egui::RichText::new(&self.status_msg).color(egui::Color32::YELLOW));
        });
    }
}