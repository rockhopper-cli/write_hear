// src/main.rs
mod app;
mod ui; 
mod subtitle_pipeline;
mod audio;
mod podman;
mod subtitles;
mod audio_pipeline;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Write Hear",
        options,
        Box::new(|cc| {
            // 1. Scale everything up by 30% (Change 1.5 to whatever looks best to you)
            cc.egui_ctx.set_zoom_factor(1.3);

            // 2. Use Inclusive Font (unable to setup system font) 
            load_custom_font(&cc.egui_ctx);

            Ok(Box::new(app::WriteHear::default()))
        }),
    )
}

fn load_custom_font(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    // 1. Load the font and convert it into an Arc<FontData>
    fonts.font_data.insert(
        "my_system_font".to_owned(),
        std::sync::Arc::new(eframe::egui::FontData::from_static(include_bytes!("../assets/InclusiveSans-VariableFont_wght.ttf"))),
    );

    // 2. Tell egui to use this font as the primary one for Proportional text
    fonts.families.entry(eframe::egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_system_font".to_owned());

    // 3. Apply the changes
    ctx.set_fonts(fonts);
}