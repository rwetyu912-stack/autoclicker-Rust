use eframe::egui;
use enigo::{Button, Direction, Enigo, Mouse, Settings};
use rand::Rng;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};
use device_query::{DeviceQuery, DeviceState, Keycode};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_resizable(false),
        ..Default::default()
    };

    let is_running = Arc::new(AtomicBool::new(false));
    let delay_ms = Arc::new(AtomicU64::new(70));

    let is_running_clicker = Arc::clone(&is_running);
    let delay_ms_clicker = Arc::clone(&delay_ms);

    // Thread 1: Eksekusi Klik Mouse
    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("Gagal inisialisasi Enigo");
        let mut rng = rand::thread_rng();

        loop {
            if is_running_clicker.load(Ordering::Relaxed) {
                let base_delay = delay_ms_clicker.load(Ordering::Relaxed);
                let jitter = rng.gen_range(0..=20);
                let actual_delay = if base_delay > 20 { base_delay - 10 + jitter } else { base_delay };

                let _ = enigo.button(Button::Left, Direction::Click);
                thread::sleep(Duration::from_millis(actual_delay));
            } else {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });

    // Thread 2: Global Hotkey Listener (Tombol F6)
    let is_running_hotkey = Arc::clone(&is_running);
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut prev_f6 = false;

        loop {
            let keys = device_state.get_keys();
            let f6_down = keys.contains(&Keycode::F6);

            if f6_down && !prev_f6 {
                let running = is_running_hotkey.load(Ordering::Relaxed);
                is_running_hotkey.store(!running, Ordering::Relaxed);
            }
            prev_f6 = f6_down;
            thread::sleep(Duration::from_millis(20));
        }
    });

    eframe::run_native(
        "Rust AutoClicker",
        options,
        Box::new(move |_cc| {
            Box::new(AutoClickerApp {
                is_running,
                delay_ms,
                current_delay: 70,
            })
        }),
    )
}

struct AutoClickerApp {
    is_running: Arc<AtomicBool>,
    delay_ms: Arc<AtomicU64>,
    current_delay: u64,
}

impl eframe::App for AutoClickerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("⚡ Rust AutoClicker");
                ui.label("Shortcut Global: Press F6");
            });
            ui.separator();
            ui.add_space(8.0);

            ui.label("Kecepatan Jeda (ms):");
            if ui.add(egui::Slider::new(&mut self.current_delay, 10..=300).text("ms")).changed() {
                self.delay_ms.store(self.current_delay, Ordering::Relaxed);
            }

            ui.add_space(12.0);

            let running = self.is_running.load(Ordering::Relaxed);
            let btn_text = if running { "STOP CLICKING (F6)" } else { "START CLICKING (F6)" };
            let btn_color = if running {
                egui::Color32::from_rgb(210, 50, 50)
            } else {
                egui::Color32::from_rgb(40, 160, 80)
            };

            if ui.add_sized([280.0, 45.0], egui::Button::new(btn_text).fill(btn_color)).clicked() {
                self.is_running.store(!running, Ordering::Relaxed);
            }

            // Minta GUI update berkala biar status tombol di GUI langsung berubah pas F6 ditekan
            ctx.request_repaint_after(Duration::from_millis(100));
        });
    }
}
