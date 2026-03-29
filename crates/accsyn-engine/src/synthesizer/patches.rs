use crate::synthesizer::ModuleParameters;
use anyhow::{Result, anyhow};
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

const APP_SUPPORT_DIRECTORY: &str = "Library/Application Support";
const DATA_DIRECTORY: &str = "AccidentalSynthesizer";
const USER_PATCH_DIRECTORY: &str = "patches";
const PATCH_FILE_EXTENSION: &str = "json";
const SYSTEM_PATCHES: &[(&str, &str)] = &[
    ("Init", include_str!("patches/init.json")),
    ("Acid Squelch", include_str!("patches/acid-squelch.json")),
    ("Acid Time", include_str!("patches/acid-time.json")),
    ("Ambient Drone", include_str!("patches/ambient-drone.json")),
    ("Bright Lead", include_str!("patches/bright-lead.json")),
    ("Deep Bass", include_str!("patches/deep-bass.json")),
    (
        "Dirty Bass Echo",
        include_str!("patches/dirty-bass-echo.json"),
    ),
    ("FM Bells", include_str!("patches/fm-bells.json")),
    ("Glass Marimba", include_str!("patches/glass-marimba.json")),
    ("Industrial", include_str!("patches/harsh-industrial.json")),
    ("Plucky Keys", include_str!("patches/plucky-keys.json")),
    ("Sci-Fi", include_str!("patches/sci-fi.json")),
    (
        "Supersaw Swirl",
        include_str!("patches/supersaw-swirl.json"),
    ),
    ("Synth Brass", include_str!("patches/synth-brass.json")),
    ("Vintage Organ", include_str!("patches/vintage-organ.json")),
    ("Warm Pad", include_str!("patches/warm-pad.json")),
];
const INIT_PARAMETERS: &str = SYSTEM_PATCHES[0].1;

/// Errors that can occur during patch file operations.
#[derive(Debug, Clone, Error)]
pub enum PatchesError {
    /// The macOS Application Support directory was not found.
    #[error("Application support directory does not exist")]
    NoApplicationSupportDirectory,

    /// The user's home directory could not be determined.
    #[error("Could not find home directory")]
    NoHomeDirectory,

    /// A patch file with the given name already exists on disk.
    #[error("Patch file already exists")]
    FileAlreadyExists,
}

/// File system paths used for application data, patches, and presets storage.
pub struct Paths {
    base: PathBuf,
    application_data: PathBuf,
    user_patches: PathBuf,
}

/// Manages reading and writing synthesizer patch and preset files.
pub struct Patches {
    paths: Paths,
}

impl Patches {
    /// Creates a new Patches instance, initializing application storage directories if needed.
    pub fn new() -> Result<Self> {
        let mut paths = create_data_paths()?;
        initialize_application_storage(&mut paths)?;

        Ok(Self { paths })
    }

    /// Serializes the current module parameters to a new named patch file.
    pub fn create_new_patch(&self, name: &str, parameters: &ModuleParameters) -> Result<()> {
        let content = create_patch_from_parameters(parameters);
        let file_name = format!("{name}.{PATCH_FILE_EXTENSION}");
        let patch_file_path = self.paths.user_patches.join(file_name);

        if patch_file_path.exists() {
            log::warn!(target: "synthesizer::patches", "Patch file already exists: {}", patch_file_path.display());
            return Err(anyhow!(
                "Patch file already exists: {}",
                patch_file_path.display()
            ));
        }

        let mut handle = std::fs::File::create(&patch_file_path).map_err(|e| {
            log::error!(target: "synthesizer::patches", "Failed to create patch file {}: {e}", patch_file_path.display());
            e
        })?;
        handle.write_all(content.as_bytes()).map_err(|e| {
            log::error!(target: "synthesizer::patches", "Failed to write patch file {}: {e}", patch_file_path.display());
            e
        })?;

        log::info!(target: "synthesizer::patches", "Created new patch file: {}", patch_file_path.display());

        Ok(())
    }
}

/// Generates a list of preset names from the building system preset patches
pub fn preset_list() -> Vec<String> {
    SYSTEM_PATCHES
        .iter()
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Loads a preset from the system preset patches by preset index. See `SYSTEM_PATCHES` for the index values.
pub fn get_preset_from_index(index: usize) -> Result<ModuleParameters> {
    let (name, content) = SYSTEM_PATCHES[index];
    let preset = serde_json::from_str(content).map_err(|e| {
        log::error!(target: "synthesizer::patches", "Failed to parse preset '{name}' (index {index}): {e}");
        e
    })?;
    log::info!(target: "synthesizer::patches", "Loaded preset '{name}' (index {index})");
    Ok(preset)
}

pub(crate) fn init_module_parameters() -> Result<ModuleParameters> {
    let parameters = serde_json::from_str(INIT_PARAMETERS).map_err(|e| {
        log::error!(target: "synthesizer::patches", "Failed to parse init parameters: {e}");
        e
    })?;
    log::debug!(target: "synthesizer::patches", "Loaded init parameters");
    Ok(parameters)
}

fn create_data_paths() -> Result<Paths> {
    let mut base = std::env::home_dir().ok_or(PatchesError::NoHomeDirectory)?;
    base.push(APP_SUPPORT_DIRECTORY);
    let application_data = base.join(DATA_DIRECTORY);
    let user_patches = application_data.join(USER_PATCH_DIRECTORY);

    log::debug!(target: "synthesizer::patches", "Data paths resolved: base={}, data={}, patches={}",
        base.display(), application_data.display(), user_patches.display());

    let paths = Paths {
        base,
        application_data,
        user_patches,
    };

    Ok(paths)
}

fn initialize_application_storage(paths: &mut Paths) -> Result<()> {
    if !paths.base.exists() {
        log::warn!(target: "synthesizer::patches", "Application support directory does not exist. {}", paths.base.display());
        return Err(anyhow!(
            "{}, {}",
            PatchesError::NoApplicationSupportDirectory,
            paths.base.display()
        ));
    }

    if !paths.application_data.exists() {
        log::debug!(target: "synthesizer::patches", "Application data directory does not exist. Creating: {}", paths
            .application_data.display());
        std::fs::create_dir(&paths.application_data).map_err(|e| {
            log::error!(target: "synthesizer::patches", "Failed to create application data directory {}: {e}", paths.application_data.display());
            e
        })?;
        log::info!(target: "synthesizer::patches", "Created application data directory: {}", paths.application_data.display());
    }

    if !paths.user_patches.exists() {
        log::debug!(target: "synthesizer::patches", "User patches directory does not exist. Creating: {}", paths
            .user_patches.display());
        std::fs::create_dir(&paths.user_patches).map_err(|e| {
            log::error!(target: "synthesizer::patches", "Failed to create user patches directory {}: {e}", paths.user_patches.display());
            e
        })?;
        log::info!(target: "synthesizer::patches", "Created user patches directory: {}", paths.user_patches.display());
    }

    Ok(())
}

fn create_patch_from_parameters(parameters: &ModuleParameters) -> String {
    serde_json::to_string_pretty(&parameters).unwrap_or(json!(parameters).to_string())
}
