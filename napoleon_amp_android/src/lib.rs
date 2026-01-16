use eframe::NativeOptions;
use napoleon_amp_client_ui::NapoleonClientApp;

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
        Box::new(|_cc| Ok(Box::new(NapoleonClientApp::new()))),
    )
    .unwrap();
}
