use chrono::{self, DateTime, Datelike, Duration, Local, NaiveTime, Utc};
use eframe::egui;
use egui::RichText;
use log::{log, Level};

// fn main() {
//     let native_options = eframe::NativeOptions::default();
//     let _ = eframe::run_native(
//         "My egui App",
//         native_options,
//         Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
//     );
// }

pub mod upgrades;
use upgrades::{get_upgrades, Upgrade};

pub trait Update {
    fn update(&self);
}

pub struct MyEguiApp {
    time: DateTime<Local>,
    dt: f64,
    cats: [f64; 31],
    cat_multipliers: [f64; 31],
    day_offset: f64,
    day_width: i64,
    //  NOTE: Resets every frame, make sure to update every frame even when implementing static multipliers
    cat_prices: [f64; 31],
    cat_price_multipliers: [f64; 31],
    cat_times: [f64; 31],
    currency: f64,
    colors: [egui::Color32; 1],
    upgrades: Vec<Box<Upgrade>>,
    strawberries: f64,
    cat_strawberries: [i64; 31],
    cat_strawberry_prices: [i64; 31],
    unlocked_tiers: [bool; 1],
    status: String,
    status_time: DateTime<Local>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SaveStruct {
    dt: f64,
    cats: [f64; 31],
    cat_multipliers: [f64; 31],
    day_offset: f64,
    day_width: i64,
    //  NOTE: Resets every frame, make sure to update every frame even when implementing static multipliers
    cat_prices: [f64; 31],
    cat_price_multipliers: [f64; 31],
    cat_times: [f64; 31],
    currency: f64,
    colors: [egui::Color32; 1],
    upgrades: Vec<(String, i64, i64)>,
    strawberries: f64,
    cat_strawberries: [i64; 31],
    cat_strawberry_prices: [i64; 31],
    unlocked_tiers: [bool; 1],
}

impl Default for SaveStruct {
    fn default() -> Self {
        Self {
            dt: 0.0,
            cats: [0.0; 31],
            cat_multipliers: [1.0; 31],
            cat_prices: [1.0; 31],
            cat_price_multipliers: [1.5; 31],
            day_offset: 1.0,
            day_width: 0,
            currency: 1.0,
            cat_times: [0.0; 31],
            colors: [egui::Color32::from_hex("#FF784F").unwrap()],
            upgrades: vec![],
            strawberries: 0.0,
            cat_strawberries: [0; 31],
            cat_strawberry_prices: [1; 31],
            unlocked_tiers: [false; 1],
        }
    }
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            time: Local::now(),
            dt: 0.0,
            cats: [0.0; 31],
            cat_multipliers: [1.0; 31],
            cat_prices: [1.0; 31],
            cat_price_multipliers: [1.5; 31],
            day_offset: 1.0,
            day_width: 0,
            currency: 1.0,
            cat_times: [0.0; 31],
            colors: [egui::Color32::from_hex("#FF784F").unwrap()],
            upgrades: upgrades::get_upgrades(),
            strawberries: 0.0,
            cat_strawberries: [0; 31],
            cat_strawberry_prices: [1; 31],
            unlocked_tiers: [false; 1],
            status: "Opened game".to_owned(),
            status_time: Local::now(),
        }
    }
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        if let Some(storage) = cc.storage {
            let t: SaveStruct = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            let default_upgrades = get_upgrades();

            let mut final_upgrades = vec![];
            for i in default_upgrades {
                let mut upgrade = i;
                for j in &t.upgrades {
                    if &j.0 != &upgrade.text {
                        continue;
                    }
                    if &j.2 != &upgrade.max {
                        continue;
                    }
                    upgrade.count = j.1;
                    upgrade.price = upgrade.price * upgrade.price_mult.powi(j.1 as i32);
                    break;
                }
                final_upgrades.push(upgrade);
            }
            MyEguiApp {
                time: Local::now(),
                dt: t.dt,
                cats: t.cats,
                cat_multipliers: t.cat_multipliers,
                day_offset: t.day_offset,
                day_width: t.day_width,
                cat_prices: t.cat_prices,
                cat_price_multipliers: t.cat_price_multipliers,
                cat_times: t.cat_times,
                currency: t.currency,
                colors: t.colors,
                upgrades: final_upgrades,
                strawberries: t.strawberries,
                cat_strawberries: t.cat_strawberries,
                cat_strawberry_prices: t.cat_strawberry_prices,
                unlocked_tiers: t.unlocked_tiers,
                status: "Opened game".to_owned(),
                status_time: Local::now(),
            }
        } else {
            Default::default()
        }
    }
}

fn update(app: &mut MyEguiApp, date: DateTime<Utc>) -> (f64, f64) {
    app.cat_multipliers = [1.0; 31];
    let day = date.day0() as f64;
    let mut cps = 0.0;
    for i in 0..app.upgrades.len() {
        if app.upgrades[i].count > 0 {
            (app.upgrades[i].effect)(app, app.upgrades[i].count);
        }
    }
    for i in 0..app.cats.len() {
        if within_day_range(day, app.day_width as f64, i as f64) {
            app.cat_multipliers[i] *= 1.5;
            app.cat_times[i] += app.dt;
        } else {
            app.cat_times[i] = -0.05;
        }
        app.cat_multipliers[i] *= 1.5f64.powi(app.cat_strawberries[i] as i32);
    }

    cps += app
        .cats
        .iter()
        .zip(app.cat_multipliers.iter())
        .map(|(x, y)| x * y)
        .sum::<f64>();
    app.currency += cps * app.dt;
    (cps, day)
}

fn within_day_range(day: f64, width: f64, i: f64) -> bool {
    if day + width < 31.0 {
        if i >= day && i <= day + width {
            true
        } else {
            false
        }
    } else {
        if i >= day || i <= (day + width).rem_euclid(31.0) {
            true
        } else {
            false
        }
    }
}

fn cat_handler(app: &mut MyEguiApp, ui: &mut egui::Ui, day: f64) {
    let mut count = 0;
    for i in 0..app.cats.len() {
        ui.horizontal(|ui| {
            if within_day_range(day, app.day_width as f64, i as f64) {
                count += 1;
                ui.label(
                    RichText::new(format!(
                        "You have {} 'Day {}' cats [{:.2}]",
                        app.cats[i],
                        i + 1,
                        app.cat_multipliers[i]
                    ))
                    .color(app.colors[0]),
                );
            } else {
                ui.label(format!(
                    "You have {} 'Day {}' cats [{:.2}]",
                    app.cats[i],
                    i + 1,
                    app.cat_multipliers[i]
                ));
            }

            if ui
                .add_enabled(
                    app.cat_prices[i] <= app.currency,
                    egui::Button::new(format!("Hire another cat {:.2}$", app.cat_prices[i])),
                )
                .on_hover_text(format!(
                    "x{} to self, x5 to all other unbought cats",
                    app.cat_price_multipliers[i],
                ))
                .clicked()
            {
                app.currency -= app.cat_prices[i];
                if app.cats[i] == 0.0 {
                    for j in 0..app.cat_prices.len() {
                        if i != j && app.cats[j] == 0.0 {
                            app.cat_prices[j] *= 5.0;
                        }
                    }
                }
                app.cats[i] += 1.0;
                app.cat_prices[i] *= app.cat_price_multipliers[i];
            }

            if app.unlocked_tiers[0] {
                if ui
                    .add_enabled(
                        app.strawberries >= app.cat_strawberry_prices[i].pow(2) as f64,
                        egui::Button::new(format!(
                            "Feed cat {} strawberry",
                            app.cat_strawberry_prices[i].pow(2)
                        )),
                    )
                    .clicked()
                {
                    app.strawberries -= app.cat_strawberry_prices[i].pow(2) as f64;
                    app.cat_strawberries[i] += 1;
                    app.cat_strawberry_prices[i] += 1;
                }
            }
        });
    }
}

impl eframe::App for MyEguiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let t = SaveStruct {
            dt: self.dt,
            cats: self.cats,
            cat_multipliers: self.cat_multipliers,
            day_offset: 0.0,
            day_width: self.day_width,
            cat_prices: self.cat_prices,
            cat_price_multipliers: self.cat_price_multipliers,
            cat_times: self.cat_times,
            currency: self.currency,
            colors: self.colors,
            upgrades: self
                .upgrades
                .iter()
                .map(|x| (x.text.to_owned(), x.count, x.max))
                .collect(),
            strawberries: self.strawberries,
            cat_strawberries: self.cat_strawberries,
            cat_strawberry_prices: self.cat_strawberry_prices,
            unlocked_tiers: self.unlocked_tiers,
        };
        eframe::set_value(storage, eframe::APP_KEY, &t);
        log!(Level::Info, "Saved!");
        self.status = "Saved!".to_owned();
        self.status_time = Local::now();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if !is_web {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                    if ui.button("Reset").clicked() {
                        *self = MyEguiApp::default();
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{} {}s ago",
                        self.status,
                        (Local::now() - self.status_time).num_seconds(),
                    ));
                });
            });
        });

        let date = Utc::now() + Duration::seconds(self.day_offset as i64);
        let (cps, day) = update(self, date);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("sutki [W.I.P. name]");
            ui.label(format!(
                "You currently have {}$ (+{:.2}$/s)",
                self.currency.round(),
                cps
            ));
            if self.unlocked_tiers[0] {
                ui.label(format!("You have {} strawberries.", self.strawberries));
            }
            let tomorrow_midnight = (date + Duration::days(1))
                .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                .unwrap();

            ui.label(format!(
                "{} seconds until tomorrow.",
                (tomorrow_midnight - date).num_seconds(),
            ));

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_width(330.0);

                cat_handler(self, ui, day);

                for i in 0..self.upgrades.len() {
                    let price = self.upgrades[i].price;
                    if ui
                        .add_enabled(
                            price <= self.currency && self.upgrades[i].count < self.upgrades[i].max,
                            egui::Button::new(format!(
                                "{} {:.2}$ [{}/{}]",
                                self.upgrades[i].text,
                                price,
                                self.upgrades[i].count,
                                self.upgrades[i].max
                            )),
                        )
                        .on_hover_text(&self.upgrades[i].description)
                        .on_disabled_hover_text(format!(
                            "[{}s, x{}] {}",
                            ((price - self.currency) / cps).ceil(),
                            self.upgrades[i].price_mult,
                            self.upgrades[i].description
                        ))
                        .clicked()
                    {
                        self.currency -= self.upgrades[i].price;
                        self.upgrades[i].price *= self.upgrades[i].price_mult;
                        self.upgrades[i].count += 1;
                    }
                }

                if ui
                    .add_enabled(
                        self.cats.iter().sum::<f64>() >= 60.0,
                        egui::Button::new(format!(
                            "Prestige for {:.2} strawberries",
                            self.cats.iter().sum::<f64>() / 30.0 - 1.0
                        )),
                    )
                    .clicked()
                {
                    self.strawberries += self.cats.iter().sum::<f64>() / 30.0 - 1.0;
                    self.cat_prices = [1.0; 31];
                    self.cats = [0.0; 31];
                    self.upgrades = get_upgrades();
                    self.currency = 1.0;
                    self.day_width = 0;
                    self.unlocked_tiers[0] = true;
                }
            });

            // ui.hyperlink("https://github.com/emilk/egui");
            // ui.text_edit_singleline(&mut self.label);
            // if ui.button("Click me").clicked() {}
            // ui.add(egui::Slider::new(&mut self.fps, 0.0..=240.0).prefix("Desired FPS: "));
            // ui.label(format!("Current FPS: {}", (1.0 / self.dt).round()));
            // ui.label(format!("count: {}", self.count));

            // ui.checkbox(&mut false, "Checkbox");

            // ui.horizontal(|ui| {
            //     ui.radio_value(&mut self.num, Enum::First, "First");
            //     ui.radio_value(&mut self.num, Enum::Second, "Second");
            //     ui.radio_value(&mut self.num, Enum::Third, "Third");
            // });

            // ui.separator();

            // ui.collapsing("Click to see what is hidden!", |ui| {
            //     ui.label("Not much, as it turns out");
            // });
        });
        self.dt = (Local::now() - self.time).num_microseconds().unwrap() as f64 * 1e-6;
        self.time = Local::now();
        ctx.request_repaint();
    }
}
