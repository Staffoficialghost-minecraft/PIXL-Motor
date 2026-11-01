use crate::{Output, process_text};
use eframe::egui::{self, ColorImage, TextureHandle};
use egui_extras::install_image_loaders;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageLevel};
use std::fs;
use std::path::PathBuf;

pub fn launch(path: Option<&str>, _scale: usize) -> Result<(), String> {
    let path = path.map(PathBuf::from);
    let initial_text = path
        .as_ref()
        .map(fs::read_to_string)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PIXL MotorRU")
            .with_icon(egui::IconData {
                rgba: include_bytes!("../assets/pixl-logo.rgba").to_vec(),
                width: 64,
                height: 64,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "PIXL MotorRU",
        native_options,
        Box::new(move |creation_context| {
            Ok(Box::new(EditorApp::new(
                creation_context,
                initial_text,
                path,
            )))
        }),
    )
    .map_err(|error| error.to_string())
}

struct EditorApp {
    editor: String,
    file_path: Option<PathBuf>,
    status: String,
    diagnostics: Vec<String>,
    preview: Option<TextureHandle>,
    preview_size: [usize; 2],
}

impl EditorApp {
    fn new(
        _creation_context: &eframe::CreationContext<'_>,
        editor: String,
        file_path: Option<PathBuf>,
    ) -> Self {
        Self {
            editor,
            file_path,
            status: "Listo".into(),
            diagnostics: Vec::new(),
            preview: None,
            preview_size: [0, 0],
        }
    }

    fn open_file(&mut self, context: &egui::Context) {
        let Some(path) = FileDialog::new().add_filter("PIXL", &["pixl"]).pick_file() else {
            return;
        };
        match fs::read_to_string(&path) {
            Ok(text) => {
                self.editor = text;
                self.file_path = Some(path.clone());
                self.status = path.display().to_string();
                self.diagnostics.clear();
                self.preview = None;
                context.request_repaint();
            }
            Err(error) => show_error(&format!("No se pudo leer el archivo: {error}")),
        }
    }

    fn save_file(&mut self) {
        let path = self.file_path.clone().or_else(|| {
            FileDialog::new()
                .add_filter("PIXL", &["pixl"])
                .set_file_name("document.pixl")
                .save_file()
        });
        let Some(path) = path else {
            return;
        };
        match fs::write(&path, &self.editor) {
            Ok(()) => {
                self.file_path = Some(path.clone());
                self.status = format!("Guardado: {}", path.display());
            }
            Err(error) => show_error(&format!("No se pudo guardar el archivo: {error}")),
        }
    }

    fn process(&mut self, context: &egui::Context) {
        let file = self
            .file_path
            .as_deref()
            .map_or("<GUI>", |path| path.to_str().unwrap_or("<GUI>"));
        let (output, diagnostics) = process_text(&self.editor, file);
        self.diagnostics = diagnostics
            .0
            .iter()
            .map(|diagnostic| {
                format!(
                    "[{:?}] {}:{}:{} {}",
                    diagnostic.severity,
                    diagnostic.code,
                    diagnostic.line,
                    diagnostic.character,
                    diagnostic.message
                )
            })
            .collect();
        let Some(output) = output else {
            self.preview = None;
            self.status = "Documento inválido".into();
            return;
        };
        match output {
            Output::Raster(image) => {
                let checker_size = 8_usize;
                let mut preview_pixels = Vec::with_capacity(image.pixels.len() * 4);
                for (index, pixel) in image.pixels.iter().enumerate() {
                    let x = index % image.width;
                    let y = index / image.width;
                    let checker = if (x / checker_size + y / checker_size) % 2 == 0 {
                        224_u16
                    } else {
                        176_u16
                    };
                    let alpha = u16::from(pixel.3);
                    let inverse_alpha = 255_u16 - alpha;
                    preview_pixels.extend([
                        ((u16::from(pixel.0) * alpha + checker * inverse_alpha) / 255) as u8,
                        ((u16::from(pixel.1) * alpha + checker * inverse_alpha) / 255) as u8,
                        ((u16::from(pixel.2) * alpha + checker * inverse_alpha) / 255) as u8,
                        255,
                    ]);
                }
                let color_image = ColorImage::from_rgba_unmultiplied(
                    [image.width, image.height],
                    &preview_pixels,
                );
                self.preview = Some(context.load_texture(
                    "pixl-preview",
                    color_image,
                    egui::TextureOptions::NEAREST,
                ));
                self.preview_size = [image.width, image.height];
                self.status = format!("Render: {}x{}", image.width, image.height);
            }
            Output::Svg(_) => {
                self.preview = None;
                self.status = "TIPE(vec): use la exportación SVG para visualizar".into();
            }
            Output::Animation(animation) => {
                self.preview = None;
                self.status = format!(
                    "PIXL 1.2: {} frames; use la CLI para exportar",
                    animation.frames.len()
                );
            }
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(context);
        egui::TopBottomPanel::top("toolbar").show(context, |ui| {
            ui.add(
                egui::Image::from_bytes(
                    "bytes://pixl-logo.svg",
                    include_bytes!("../assets/pixl-logo.svg"),
                )
                .fit_to_exact_size(egui::vec2(42.0, 42.0)),
            );
            ui.separator();
            if ui.button("Abrir").clicked() {
                self.open_file(context);
            }
            if ui.button("Guardar").clicked() {
                self.save_file();
            }
            if ui.button("Procesar").clicked() {
                self.process(context);
            }
            ui.separator();
            ui.label(&self.status);
        });
        egui::TopBottomPanel::bottom("diagnostics")
            .resizable(true)
            .default_height(120.0)
            .show(context, |ui| {
                ui.heading("Diagnósticos");
                if self.diagnostics.is_empty() {
                    ui.label("Sin diagnósticos");
                } else {
                    for diagnostic in &self.diagnostics {
                        ui.label(diagnostic);
                    }
                }
            });
        egui::SidePanel::left("editor")
            .resizable(true)
            .default_width(480.0)
            .show(context, |ui| {
                ui.heading("Editor PIXL");
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.editor)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(30)
                                .desired_width(900.0),
                        );
                    });
            });
        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("Vista previa");
            if let Some(texture) = &self.preview {
                let available = ui.available_size();
                let scale = (available.x / self.preview_size[0] as f32)
                    .min(available.y / self.preview_size[1] as f32)
                    .max(1.0);
                let size = egui::vec2(
                    self.preview_size[0] as f32 * scale,
                    self.preview_size[1] as f32 * scale,
                );
                ui.add(egui::Image::from_texture(texture).fit_to_exact_size(size));
            } else {
                ui.centered_and_justified(|ui| ui.label("Sin resultado"));
            }
        });
    }
}

fn show_error(message: &str) {
    let _ = MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title("PIXL MotorRU")
        .set_description(message)
        .set_buttons(MessageButtons::Ok)
        .show();
}
