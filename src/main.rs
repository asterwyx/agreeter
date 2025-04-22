use eframe::egui;
use wayland_protocols_wlr::layer_shell
fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("agreeter",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<AGreeter>::default())
        })
    )
}

struct AGreeter {
    name: String,
    greeting: String,
}

impl Default for AGreeter {
    fn default() -> Self {
        Self {
            name: "AGreeter".to_owned(),
            greeting: "Hello, World!".to_owned(),
        }
    }
}

impl eframe::App for AGreeter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Enter your name:");
            ui.image(egui::include_image!("/home/astrea/Pictures/Avatars/Caption.jpg"));
            ui.text_edit_singleline(&mut self.name);
            if ui.button("Greet").clicked() {
                self.greeting = format!("Hello, {}!", self.name);
            }
            ui.label(&self.greeting);
        });
    }
}
