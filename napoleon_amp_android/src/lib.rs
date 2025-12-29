use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame, NativeOptions};

// #[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: egui_winit::winit::platform::android::activity::AndroidApp) {
    let options = NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    eframe::run_native(
        "Napoleon App",
        options,
        Box::new(|cc| Ok(Box::new(BasicApp))),
    )
    .unwrap();
}

struct BasicApp;

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            for i in 0..10 {
                ui.label(format!("Hello worlda! {}", i));
            }
        });
    }
}
