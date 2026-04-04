use crate::synthesizer::ModuleParameters;
use anyhow::{Result, anyhow};
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

const APP_SUPPORT_DIRECTORY: &str = "Library/Application Support";
const DATA_DIRECTORY: &str = "AccidentalSynthesizer";
const USER_PATCH_DIRECTORY: &str = "patches";
const PATCH_FILE_EXTENSION: &str = "json";
const SYSTEM_PATCHES: &[(&str, &str)] = &[
    ("Init", include_str!("patches/init.json")),
    ("Acid Squelch", include_str!("patches/acid-squelch.json")),
    ("Acid Time", include_str!("patches/acid-time.json")),
    (
        "Alien Invasion",
        include_str!("patches/alien-invasion.json"),
    ),
    ("Ambient Drone", include_str!("patches/ambient-drone.json")),
    ("Bright Lead", include_str!("patches/bright-lead.json")),
    ("Deep Bass", include_str!("patches/deep-bass.json")),
    (
        "Dirty Bass Echo",
        include_str!("patches/dirty-bass-echo.json"),
    ),
    ("Drifting Pad", include_str!("patches/drifting-pad.json")),
    ("FM Bells", include_str!("patches/fm-bells.json")),
    ("Plucky Keys", include_str!("patches/plucky-keys.json")),
    ("Sci-Fi", include_str!("patches/sci-fi.json")),
    (
        "Supersaw Swirl",
        include_str!("patches/supersaw-swirl.json"),
    ),
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

/// Stores one patch name and value
#[derive(Debug, Clone)]
pub struct Patch {
    name: String,
    content: String,
}

/// Stores a list of patch names and values
#[derive(Debug, Clone)]
pub struct PatchList {
    list: Vec<Patch>,
}

impl PatchList {
    /// Returns the names of the patches in the list.
    pub fn names(&self) -> Vec<String> {
        self.list.iter().map(|p| p.name.clone()).collect()
    }

    /// Returns the list of patches as a vector
    pub fn list(&self) -> Vec<Patch> {
        self.list.clone()
    }
}

/// Manages reading and writing synthesizer patch and preset files.
pub struct Patches {
    paths: Paths,
    patches: PatchList,
}

impl Patches {
    /// Creates a new Patches instance, initializing application storage directories if needed.
    pub fn new() -> Result<Self> {
        let mut paths = create_data_paths()?;
        initialize_application_storage(&mut paths)?;
        let patches = load_patches(&paths.user_patches);
        Ok(Self { paths, patches })
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

    /// Get the list of user-created patch names from the files in the patch directory.
    pub fn patch_list(&self) -> PatchList {
        self.patches.clone()
    }

    /// Loads a preset from the system preset patches by preset index. See `SYSTEM_PATCHES` for the index values.
    pub fn get_module_parameters_from_patch_index(
        &self,
        index: usize,
        patch_list: &PatchList,
    ) -> Result<ModuleParameters> {
        let patch = &patch_list.list[index];
        let preset = serde_json::from_str(&patch.content).map_err(|err| {
            log::error!(target: "synthesizer::patches", "Failed to parse preset '{}' (index {index}): {err}", patch.name);
            err
        })?;
        log::info!(target: "synthesizer::patches", "Loaded preset '{}' (index {index})", patch.name);
        Ok(preset)
    }
}

fn load_presets() -> Vec<Patch> {
    SYSTEM_PATCHES
        .iter()
        .map(|(name, content)| Patch {
            name: name.to_string(),
            content: content.to_string(),
        })
        .collect()
}

/// Create patch collection from user patches directory.
pub fn load_patches(patch_directory: &Path) -> PatchList {
    let mut patches = load_presets();

    if let Ok(entries) = patch_directory.read_dir() {
        entries.filter_map(|entry| entry.ok()).for_each(|entry| {
            if let Some(name) = entry.path().as_path().file_stem() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    patches.push(Patch {
                        name: name.to_string_lossy().to_string(),
                        content,
                    })
                }
          }
        });
    };

    PatchList { list: patches }
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
