use base64::{engine::general_purpose::STANDARD, Engine as _};
use libloading::Library;
use serde::Serialize;
use std::{
    ffi::CString,
    fs,
    os::raw::{c_char, c_void},
    path::PathBuf,
    ptr, slice,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex, OnceLock,
    },
};

type EnvironmentCallback = unsafe extern "C" fn(u32, *mut c_void) -> bool;
type VideoCallback = unsafe extern "C" fn(*const c_void, u32, u32, usize);
type AudioCallback = unsafe extern "C" fn(i16, i16);
type AudioBatchCallback = unsafe extern "C" fn(*const i16, usize) -> usize;
type InputPollCallback = unsafe extern "C" fn();
type InputStateCallback = unsafe extern "C" fn(u32, u32, u32, u32) -> i16;

type MemoryDataCallback = unsafe extern "C" fn(u32) -> *mut c_void;
type MemorySizeCallback = unsafe extern "C" fn(u32) -> usize;

const RETRO_MEMORY_SAVE_RAM: u32 = 0;
// Attendre 30 frames sans changement avant d'écrire le fichier.
const SAVE_STABLE_FRAMES: u32 = 30;

#[repr(C)]
struct RetroGameInfo {
    path: *const c_char,
    data: *const c_void,
    size: usize,
    meta: *const c_char,
}

#[derive(Default)]
struct FrameBuffer {
    width: u32,
    height: u32,
    frame_id: u64,
    rgba: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoFrame {
    width: u32,
    height: u32,
    frame_id: u64,
    rgba_base64: String,
}

static VIDEO_FRAME: OnceLock<Mutex<FrameBuffer>> = OnceLock::new();
static PIXEL_FORMAT: AtomicU32 = AtomicU32::new(1);
// État des boutons GBA, partagé avec les callbacks de mGBA.
static INPUT_MASK: AtomicU32 = AtomicU32::new(0);

fn frame_buffer() -> &'static Mutex<FrameBuffer> {
    VIDEO_FRAME.get_or_init(|| Mutex::new(FrameBuffer::default()))
}

unsafe extern "C" fn environment(command: u32, data: *mut c_void) -> bool {
    match command {
        // RETRO_ENVIRONMENT_GET_CAN_DUPE
        3 if !data.is_null() => {
            unsafe { *(data as *mut bool) = true };
            true
        }

        // RETRO_ENVIRONMENT_SET_PIXEL_FORMAT
        10 if !data.is_null() => {
            let format = unsafe { *(data as *const u32) };

            if format <= 2 {
                PIXEL_FORMAT.store(format, Ordering::Relaxed);
                true
            } else {
                false
            }
        }

        // RETRO_ENVIRONMENT_GET_INPUT_BITMASKS
        51 if !data.is_null() => {
            unsafe { *(data as *mut bool) = true };
            true
        }

        _ => false,
    }
}

unsafe extern "C" fn video_refresh(
    data: *const c_void,
    width: u32,
    height: u32,
    pitch: usize,
) {
    if data.is_null() || width == 0 || height == 0 {
        return;
    }

    // Protection contre des dimensions inattendues.
    if width > 4096 || height > 4096 {
        return;
    }

    let format = PIXEL_FORMAT.load(Ordering::Relaxed);
    let bytes_per_pixel = if format == 1 { 4 } else { 2 };
    let row_bytes = width as usize * bytes_per_pixel;

    if pitch < row_bytes {
        return;
    }

    let Some(output_len) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
    else {
        return;
    };

    let mut rgba = vec![0_u8; output_len];
    let source = data.cast::<u8>();

    for y in 0..height as usize {
        let row = unsafe { source.add(y * pitch) };

        for x in 0..width as usize {
            let (r, g, b) = unsafe {
                match format {
                    // RETRO_PIXEL_FORMAT_0RGB1555
                    0 => {
                        let value = ptr::read_unaligned(
                            row.add(x * 2).cast::<u16>(),
                        );
                        (
                            expand5((value >> 10) & 0x1f),
                            expand5((value >> 5) & 0x1f),
                            expand5(value & 0x1f),
                        )
                    }

                    // RETRO_PIXEL_FORMAT_XRGB8888
                    1 => {
                        let value = ptr::read_unaligned(
                            row.add(x * 4).cast::<u32>(),
                        );
                        (
                            ((value >> 16) & 0xff) as u8,
                            ((value >> 8) & 0xff) as u8,
                            (value & 0xff) as u8,
                        )
                    }

                    // RETRO_PIXEL_FORMAT_RGB565
                    2 => {
                        let value = ptr::read_unaligned(
                            row.add(x * 2).cast::<u16>(),
                        );
                        (
                            expand5((value >> 11) & 0x1f),
                            expand6((value >> 5) & 0x3f),
                            expand5(value & 0x1f),
                        )
                    }

                    _ => return,
                }
            };

            let index = (y * width as usize + x) * 4;
            rgba[index] = r;
            rgba[index + 1] = g;
            rgba[index + 2] = b;
            rgba[index + 3] = 255;
        }
    }

    let mut frame = frame_buffer()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    frame.width = width;
    frame.height = height;
    frame.frame_id += 1;
    frame.rgba = rgba;
}

fn expand5(value: u16) -> u8 {
    let value = value as u8;
    (value << 3) | (value >> 2)
}

fn expand6(value: u16) -> u8 {
    let value = value as u8;
    (value << 2) | (value >> 4)
}

unsafe extern "C" fn audio_sample(_left: i16, _right: i16) {}

unsafe extern "C" fn audio_sample_batch(_data: *const i16, frames: usize) -> usize {
    frames
}

unsafe extern "C" fn input_poll() {}

unsafe extern "C" fn input_state(
    port: u32,
    device: u32,
    index: u32,
    id: u32,
) -> i16 {
    // Port 0, manette de type joypad, index principal.
    if port != 0 || device != 1 || index != 0 {
        return 0;
    }

    let buttons = INPUT_MASK.load(Ordering::Relaxed);

    // RETRO_DEVICE_ID_JOYPAD_MASK : tous les boutons en un masque.
    if id == 256 {
        return buttons as i16;
    }

    if id >= 32 {
        return 0;
    }

    if buttons & (1_u32 << id) != 0 {
        1
    } else {
        0
    }
}


pub fn set_button(button: &str, pressed: bool) -> Result<(), String> {
    // Identifiants standard des boutons Libretro.
    let id = match button {
        "start" => 3,
        "select" => 2,
        "b" => 0,
        "a" => 8,
        "l" => 10,
        "r" => 11,
        "up" => 4,
        "down" => 5,
        "left" => 6,
        "right" => 7,
        _ => return Err(format!("Bouton GBA inconnu : {button}")),
    };

    let mask = 1_u32 << id;

    if pressed {
        INPUT_MASK.fetch_or(mask, Ordering::Relaxed);
    } else {
        INPUT_MASK.fetch_and(!mask, Ordering::Relaxed);
    }

    Ok(())
}


fn load_save_ram(
    path: &PathBuf,
    get_memory_data: MemoryDataCallback,
    get_memory_size: MemorySizeCallback,
) -> Result<String, String> {
    let size = unsafe { get_memory_size(RETRO_MEMORY_SAVE_RAM) };
    let data = unsafe { get_memory_data(RETRO_MEMORY_SAVE_RAM) };

    if size == 0 || data.is_null() {
        return Err(
            "mGBA n'expose pas de mémoire de sauvegarde.".to_string(),
        );
    }

    if !path.exists() {
        return Ok(format!(
            "Aucune sauvegarde trouvée. Une nouvelle sera créée : {} octets disponibles.",
            size
        ));
    }

    let saved = fs::read(path)
        .map_err(|e| format!("Lecture du fichier .sav impossible : {e}"))?;

    let copy_len = saved.len().min(size);

    unsafe {
        ptr::copy_nonoverlapping(
            saved.as_ptr(),
            data.cast::<u8>(),
            copy_len,
        );
    }

    Ok(format!(
        "Sauvegarde chargée : {copy_len} octets copiés sur {size} disponibles."
    ))
}

fn write_save_ram(
    path: &PathBuf,
    get_memory_data: MemoryDataCallback,
    get_memory_size: MemorySizeCallback,
) -> Result<String, String> {
    let size = unsafe { get_memory_size(RETRO_MEMORY_SAVE_RAM) };
    let data = unsafe { get_memory_data(RETRO_MEMORY_SAVE_RAM) };

    if size == 0 || data.is_null() {
        return Err(
            "mGBA n'expose pas de mémoire de sauvegarde.".to_string(),
        );
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Création du dossier de sauvegarde impossible : {e}"))?;
    }

    let bytes = unsafe {
        slice::from_raw_parts(data.cast::<u8>(), size)
    };

    fs::write(path, bytes)
        .map_err(|e| format!("Écriture du fichier .sav impossible : {e}"))?;

    Ok(format!("Sauvegarde écrite : {} octets.", size))
}

struct LibretroCore {
    // Garde la DLL chargée tant que ses fonctions sont utilisées.
    _library: Library,
    run: unsafe extern "C" fn(),
    unload_game: unsafe extern "C" fn(),
    deinit: unsafe extern "C" fn(),

    get_memory_data: MemoryDataCallback,
    get_memory_size: MemorySizeCallback,
    save_path: PathBuf,

    // Suivi des changements de la sauvegarde en mémoire.
    observed_save_ram: Vec<u8>,
    save_dirty: bool,
    frames_since_save_change: u32,

    _rom: Vec<u8>,
    _rom_path: CString,
}

impl LibretroCore {
    fn check_save_ram(&mut self) -> Result<(), String> {
        let size = unsafe {
            (self.get_memory_size)(RETRO_MEMORY_SAVE_RAM)
        };
        let data = unsafe {
            (self.get_memory_data)(RETRO_MEMORY_SAVE_RAM)
        };

        if size == 0 || data.is_null() {
            return Err(
                "mGBA n'expose pas de mémoire de sauvegarde.".to_string()
            );
        }

        if size != self.observed_save_ram.len() {
            return Err(
                "La taille de la mémoire de sauvegarde a changé.".to_string()
            );
        }

        let current = unsafe {
            slice::from_raw_parts(data.cast::<u8>(), size).to_vec()
        };

        // Un changement est détecté : on attend que la mémoire se stabilise.
        if current != self.observed_save_ram {
            self.observed_save_ram = current;
            self.save_dirty = true;
            self.frames_since_save_change = 0;
            return Ok(());
        }

        if !self.save_dirty {
            return Ok(());
        }

        self.frames_since_save_change =
            self.frames_since_save_change.saturating_add(1);

        if self.frames_since_save_change < SAVE_STABLE_FRAMES {
            return Ok(());
        }

        match write_save_ram(
            &self.save_path,
            self.get_memory_data,
            self.get_memory_size,
        ) {
            Ok(message) => {
                eprintln!("[mGBA] Autosauvegarde détectée : {message}");
                self.save_dirty = false;
                self.frames_since_save_change = 0;
            }
            Err(error) => {
                eprintln!(
                    "[mGBA] Échec de l'écriture automatique : {error}"
                );
                self.frames_since_save_change = 0;
            }
        }

        Ok(())
    }
}

impl Drop for LibretroCore {
    fn drop(&mut self) {
        unsafe {
            (self.unload_game)();
            (self.deinit)();
        }
    }
}

static CORE: OnceLock<Mutex<Option<LibretroCore>>> = OnceLock::new();

fn core_state() -> &'static Mutex<Option<LibretroCore>> {
    CORE.get_or_init(|| Mutex::new(None))
}

pub fn shutdown_core() -> Result<String, String> {
    let mut state = core_state()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    let Some(core) = state.as_ref() else {
        return Ok("Le cœur mGBA est déjà arrêté.".to_string());
    };

    // Écrit la mémoire de sauvegarde sur disque avant de fermer mGBA.
    let save_result = write_save_ram(
        &core.save_path,
        core.get_memory_data,
        core.get_memory_size,
    );

    // Retire le cœur de l'état global, puis le ferme proprement.
    let core = state.take();
    drop(core);

    match save_result {
        Ok(message) => {
            eprintln!("[mGBA] {message}");
            Ok(message)
        }
        Err(error) => {
            eprintln!("[mGBA] Échec de la sauvegarde à la fermeture : {error}");
            Err(error)
        }
    }
}

pub fn inspect_core() -> Result<String, String> {
    let core_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("cores")
        .join("mgba_libretro.dll");

    if !core_path.is_file() {
        return Err(format!("DLL introuvable : {}", core_path.display()));
    }

    let library = unsafe { Library::new(&core_path) }
        .map_err(|error| format!("Chargement de la DLL impossible : {error}"))?;

    let symbols: &[&[u8]] = &[
        b"retro_api_version\0",
        b"retro_set_environment\0",
        b"retro_set_video_refresh\0",
        b"retro_set_audio_sample\0",
        b"retro_set_audio_sample_batch\0",
        b"retro_set_input_poll\0",
        b"retro_set_input_state\0",
        b"retro_init\0",
        b"retro_deinit\0",
        b"retro_load_game\0",
        b"retro_unload_game\0",
        b"retro_run\0",
    ];

    for symbol in symbols {
        unsafe {
            library
                .get::<*const ()>(symbol)
                .map_err(|error| {
                    format!(
                        "Symbole absent {} : {error}",
                        String::from_utf8_lossy(&symbol[..symbol.len() - 1])
                    )
                })?;
        }
    }

    Ok(format!(
        "DLL mGBA chargée : {} symboles vérifiés.",
        symbols.len()
    ))
}

pub fn boot_rom_test() -> Result<String, String> {
    let mut state = core_state()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    if state.is_some() {
        return Ok("Le cœur mGBA est déjà démarré.".to_string());
    }

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let core_path = base
        .join("resources")
        .join("cores")
        .join("mgba_libretro.dll");

    let rom_path = base
        .join("..")
        .join("games")
        .join("roms")
        .join("PokemonEmeraudeFR.gba");

    if !rom_path.is_file() {
        return Err(format!("ROM introuvable : {}", rom_path.display()));
    }

    eprintln!("[mGBA] Lecture de la ROM...");
    let rom = fs::read(&rom_path)
        .map_err(|error| format!("Lecture de la ROM impossible : {error}"))?;

    let rom_path_c = CString::new(rom_path.to_string_lossy().as_bytes())
        .map_err(|error| format!("Chemin ROM invalide : {error}"))?;

    eprintln!("[mGBA] Chargement de la DLL...");
    let library = unsafe { Library::new(&core_path) }
        .map_err(|error| format!("Chargement de la DLL impossible : {error}"))?;

    unsafe {
        let set_environment = *library
            .get::<unsafe extern "C" fn(Option<EnvironmentCallback>)>(
                b"retro_set_environment\0",
            )
            .map_err(|error| error.to_string())?;

        let set_video = *library
            .get::<unsafe extern "C" fn(Option<VideoCallback>)>(
                b"retro_set_video_refresh\0",
            )
            .map_err(|error| error.to_string())?;

        let set_audio = *library
            .get::<unsafe extern "C" fn(Option<AudioCallback>)>(
                b"retro_set_audio_sample\0",
            )
            .map_err(|error| error.to_string())?;

        let set_audio_batch = *library
            .get::<unsafe extern "C" fn(Option<AudioBatchCallback>)>(
                b"retro_set_audio_sample_batch\0",
            )
            .map_err(|error| error.to_string())?;

        let set_input_poll = *library
            .get::<unsafe extern "C" fn(Option<InputPollCallback>)>(
                b"retro_set_input_poll\0",
            )
            .map_err(|error| error.to_string())?;

        let set_input_state = *library
            .get::<unsafe extern "C" fn(Option<InputStateCallback>)>(
                b"retro_set_input_state\0",
            )
            .map_err(|error| error.to_string())?;

        let init = *library
            .get::<unsafe extern "C" fn()>(b"retro_init\0")
            .map_err(|error| error.to_string())?;

        let load_game = *library
            .get::<unsafe extern "C" fn(*const RetroGameInfo) -> bool>(
                b"retro_load_game\0",
            )
            .map_err(|error| error.to_string())?;

        let run = *library
            .get::<unsafe extern "C" fn()>(b"retro_run\0")
            .map_err(|error| error.to_string())?;

        let unload_game = *library
            .get::<unsafe extern "C" fn()>(b"retro_unload_game\0")
            .map_err(|error| error.to_string())?;

        let deinit = *library
            .get::<unsafe extern "C" fn()>(b"retro_deinit\0")
            .map_err(|error| error.to_string())?;

        let get_memory_data = *library
            .get::<MemoryDataCallback>(b"retro_get_memory_data\0")
            .map_err(|error| error.to_string())?;

        let get_memory_size = *library
            .get::<MemorySizeCallback>(b"retro_get_memory_size\0")
            .map_err(|error| error.to_string())?;

        eprintln!("[mGBA] Enregistrement des callbacks...");
        set_environment(Some(environment));
        set_video(Some(video_refresh));
        set_audio(Some(audio_sample));
        set_audio_batch(Some(audio_sample_batch));
        set_input_poll(Some(input_poll));
        set_input_state(Some(input_state));

        eprintln!("[mGBA] Initialisation...");
        init();

        let game = RetroGameInfo {
            path: rom_path_c.as_ptr(),
            data: rom.as_ptr().cast(),
            size: rom.len(),
            meta: ptr::null(),
        };

        eprintln!("[mGBA] Chargement du jeu...");
        if !load_game(&game) {
            deinit();
            return Err("mGBA a refusé de charger la ROM.".to_string());
        }

        let save_path = base
            .join("..")
            .join("games")
            .join("saves")
            .join("PokemonEmeraudeFR.sav");

        match load_save_ram(
            &save_path,
            get_memory_data,
            get_memory_size,
        ) {
            Ok(message) => eprintln!("[mGBA] {message}"),
            Err(error) => {
                eprintln!("[mGBA] Chargement de la sauvegarde impossible : {error}");
                unload_game();
                deinit();
                return Err(error);
            }
        }

        *frame_buffer()
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = FrameBuffer::default();

        
        let save_size = get_memory_size(RETRO_MEMORY_SAVE_RAM);
        let save_data = get_memory_data(RETRO_MEMORY_SAVE_RAM);

        if save_size == 0 || save_data.is_null() {
            unload_game();
            deinit();
            return Err(
                "Impossible d'initialiser le suivi de la sauvegarde.".to_string()
            );
        }

        let observed_save_ram = slice::from_raw_parts(
            save_data.cast::<u8>(),
            save_size,
        ).to_vec();

        let core = LibretroCore {
            _library: library,
            run,
            unload_game,
            deinit,
            get_memory_data,
            get_memory_size,
            save_path,
            observed_save_ram,
            save_dirty: false,
            frames_since_save_change: 0,
            _rom: rom,
            _rom_path: rom_path_c,
        };

        *state = Some(core);

        eprintln!("[mGBA] Exécution de la première frame...");
        if let Some(core) = state.as_ref() {
            (core.run)();
        }

        let frame = frame_buffer()
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        if frame.rgba.is_empty() {
            return Err(
                "Le cœur a démarré, mais aucune image vidéo n'a été reçue.".to_string(),
            );
        }

        Ok(format!(
            "Pokémon Émeraude démarré. Image reçue : {} × {}.",
            frame.width, frame.height
        ))
    }
}

pub fn run_frame() -> Result<String, String> {
    run_frames(1)
}

pub fn run_frames(frames: u32) -> Result<String, String> {
    if frames == 0 || frames > 120 {
        return Err("Le nombre de frames doit être compris entre 1 et 120.".to_string());
    }

    let mut state = core_state()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    let core = state
        .as_mut()
        .ok_or_else(|| "Le cœur mGBA n'est pas démarré.".to_string())?;

    for _ in 0..frames {
        unsafe {
            (core.run)();
        }

        // Conserve la détection des changements de sauvegarde.
        if let Err(error) = core.check_save_ram() {
            eprintln!("[mGBA] Surveillance de la sauvegarde : {error}");
        }
    }

    Ok(format!("{frames} frame(s) exécutée(s)."))
}

pub fn get_video_frame() -> Result<VideoFrame, String> {
    let frame = frame_buffer()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    if frame.rgba.is_empty() {
        return Err("Aucune image vidéo disponible.".to_string());
    }

    Ok(VideoFrame {
        width: frame.width,
        height: frame.height,
        frame_id: frame.frame_id,
        rgba_base64: STANDARD.encode(&frame.rgba),
    })
}
