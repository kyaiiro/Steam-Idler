mod modules;
use modules::{Game, steam};
use eframe::egui::{self};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Steam Game Idler",
        native_options,
        Box::new(|_cc| Ok(Box::new(SteamIdler::default()))),
    )
}


struct SteamIdler {
    active: Option<Game>,
    search_id: String,
    rx: Option<std::sync::mpsc::Receiver<Result<(Game, Vec<u8>), String>>>,
    cover_texture: Option<egui::TextureHandle>,
    show_game_popup: bool,
}

impl Default for SteamIdler {
    fn default() -> Self {
        Self {
            active: None,
            search_id: String::new(),
            rx: None,
            cover_texture: None,
            show_game_popup: false,
        }
    }
}

impl eframe::App for SteamIdler {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(rx) = self.rx.take() {
            match rx.try_recv() {
                Ok(Ok((game, bytes))) => match image::load_from_memory(&bytes) {
                    Ok(image) => {
                        let rgba = image.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());

                        self.cover_texture = Some(ui.ctx().load_texture(
                            "game-cover",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ));
                        self.active = Some(game);
                    }
                    Err(error) => eprintln!("Could not decode cover image: {error}"),
                },
                Ok(Err(error)) => eprintln!("Could not fetch game: {error}"),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("Game loopup worker stopped");
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                let query = ui.add(egui::TextEdit::singleline(&mut self.search_id).hint_text("Enter Game ID"));

                if query.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    match self.search_id.trim().parse::<u32>() {
                        Ok(app_id) => {
                            let (tx, rx) = std::sync::mpsc::channel();
                            self.rx = Some(rx);
                            let ctx = ui.ctx().clone();

                            std::thread::spawn(move || {
                                let result = (|| {
                                    let game = steam::get_game_info(app_id)?;
                                    let bytes = reqwest::blocking::get(&game.cover)?
                                        .error_for_status()?
                                        .bytes()?
                                        .to_vec();

                                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>((game, bytes))
                                })()
                                .map_err(|error| error.to_string());

                                let _ = tx.send(result);
                                ctx.request_repaint();
                            });

                            self.show_game_popup = true
                        }
                        Err(_) => {
                            eprint!("Enter a valid numeric game ID, such as 4732690");
                        }
                    }
                }
            });
        });

        if self.show_game_popup {
            egui::Modal::new(egui::Id::unique("game_details"))
                .show(ui.ctx(), |ui| {
                    ui.set_min_width(320.0);
                    ui.heading("Game Details");

                    if let Some(game) = &self.active {
                        if let Some(texture) = &self.cover_texture {
                            ui.add(
                                egui::Image::new((texture.id(), texture.size_vec2())).max_size(egui::vec2(460.0, 260.0)),
                            );
                        }
                        ui.heading(&game.title);
                        let duration = ui.add(egui::TextEdit::singleline(&mut self.idle_duration).hint_text("Duration"))
                    }

                    if ui.button("Close").clicked() {
                        self.show_game_popup = false;
                    }
                });
        }
    }
}