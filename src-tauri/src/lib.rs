mod libretro;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use tauri::State;

#[derive(Clone)]
struct PartySnapshot(Arc<Mutex<String>>);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Bienvenue dans NuzEmu, {name} !")
}

#[tauri::command]
fn get_party_snapshot(snapshot: State<'_, PartySnapshot>) -> String {
    // Priorité à l'équipe du jeu exécuté dans NuzEmu.
    match libretro::get_party_snapshot() {
        Ok(live_snapshot) => live_snapshot,
        Err(error) => {
            eprintln!(
                "[NuzEmu] Lecture de la RAM indisponible, \
                 utilisation du pont TCP : {error}"
            );

            snapshot
                .0
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
        }
    }
}

#[tauri::command]
fn check_libretro_core() -> Result<String, String> {
    libretro::inspect_core()
}

#[tauri::command]
fn boot_libretro_rom() -> Result<String, String> {
    libretro::boot_rom_test()
}

#[tauri::command]
fn run_libretro_frame() -> Result<String, String> {
    libretro::run_frame()
}

#[tauri::command]
fn get_libretro_frame() -> Result<libretro::VideoFrame, String> {
    libretro::get_video_frame()
}

#[tauri::command]
fn set_libretro_button(button: String, pressed: bool) -> Result<(), String> {
    libretro::set_button(&button, pressed)
}

#[tauri::command]
fn run_libretro_frames(frames: u32) -> Result<String, String> {
    libretro::run_frames(frames)
}

#[tauri::command]
fn reset_libretro_rom() -> Result<String, String> {
    libretro::shutdown_core()?;
    libretro::boot_rom_test()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let party_snapshot = PartySnapshot(Arc::new(Mutex::new(String::new())));
    let listener_state = party_snapshot.clone();

    tauri::Builder::default()
        .manage(party_snapshot)
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Récepteur des instantanés envoyés par le script Lua de mGBA.
            let shared_snapshot = listener_state.0.clone();

            thread::spawn(move || {
                let listener = match TcpListener::bind("127.0.0.1:18765") {
                    Ok(listener) => listener,
                    Err(error) => {
                        eprintln!(
                            "NuzEmu : impossible d'ouvrir le port TCP 18765 : {error}"
                        );
                        return;
                    }
                };

                println!("NuzEmu : récepteur mGBA démarré sur 127.0.0.1:18765");

                for stream in listener.incoming() {
                    let Ok(stream) = stream else {
                        continue;
                    };

                    let reader = BufReader::new(stream);

                    for line in reader.lines() {
                        let Ok(line) = line else {
                            break;
                        };

                        if line.trim().is_empty() {
                            continue;
                        }

                        let mut current = shared_snapshot
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());

                        *current = line;
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_party_snapshot,
            check_libretro_core,
            boot_libretro_rom,
            run_libretro_frame,
            get_libretro_frame,
            set_libretro_button,
            run_libretro_frames,
            reset_libretro_rom
        ])
        .build(tauri::generate_context!())
        .expect("Erreur lors du démarrage de l'application Tauri")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                if let Err(error) = libretro::shutdown_core() {
                    eprintln!(
                        "[NuzEmu] Erreur pendant la fermeture : {error}"
                    );
                }
            }
        });

}
