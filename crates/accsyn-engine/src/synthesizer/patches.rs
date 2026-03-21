#!allow[dead_code]
use crate::synthesizer::ModuleParameters;
use anyhow::{Result, anyhow};
use serde_json::json;
use std::fs::{DirEntry, ReadDir};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

const APP_SUPPORT_DIRECTORY: &str = "Library/Application Support";
const DATA_DIRECTORY: &str = "AccidentalSynthesizer";
const USER_PATCH_DIRECTORY: &str = "patches";
const SYSTEM_PATCHES: &str = "./presets";

const PATCH_FILE_EXTENSION: &str = "json";
const INIT_PARAMETERS: &str = include_str!("patches/init.json");

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
    system_patches: PathBuf,
}
/// Manages reading and writing synthesizer patch and preset files.
pub struct Patches {
    paths: Paths,
    system: Vec<PathBuf>,
    user: Vec<PathBuf>,
}

impl Patches {
    /// Creates a new Patches instance, initializing application storage directories if needed.
    pub fn new() -> Result<Self> {
        let mut paths = create_data_paths()?;
        initialize_application_storage(&mut paths)?;

        let system = load_patches(&mut paths.system_patches)?;
        let user = load_patches(&mut paths.user_patches)?;

        Ok(Self {
            paths,
            system,
            user,
        })
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

        let mut handle = std::fs::File::create(&patch_file_path)?;
        handle.write_all(content.as_bytes())?;

        log::info!(target: "synthesizer::patches", "Created new patch file: {}", patch_file_path.display());

        Ok(())
    }

    pub(crate) fn init_module_parameters(&self) -> Result<ModuleParameters> {
        Ok(serde_json::from_str(INIT_PARAMETERS)?)
    }

    pub(crate) fn get_system_patch_names(&self) -> Result<Vec<String>> {
        let mut names: Vec<String> = vec![];

        names = self
            .system
            .iter()
            .filter_map(|path| path.file_prefix())
            .filter_map(|name | name.to_str())
            .map(String::from)
            .map(|name| name.replace('_', " "))
            .collect::<Vec<String>>();

        Ok(names)
    }

    fn read_patch_file(&self, name: &str) -> Result<ModuleParameters> {
        let content = read_file(name, &self.paths.user_patches)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn read_preset_file(&self, name: &str) -> Result<ModuleParameters> {
        let content = read_file(name, &self.paths.system_patches)?;
        Ok(serde_json::from_str(&content)?)
    }
}

fn load_patches(path: &mut PathBuf) -> Result<Vec<PathBuf>> {
    let mut patches = Vec::new();
    let directory = path.clone();
    let directory_entries = std::fs::read_dir(directory)?;
    directory_entries.for_each(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            patches.push(path);
        }
    });
    Ok(patches)
}

fn read_file(name: &str, path: &Path) -> Result<String> {
    let file_name = format!("{name}.{PATCH_FILE_EXTENSION}");
    let file_path = path.join(file_name);
    Ok(std::fs::read_to_string(file_path)?)
}

fn create_data_paths() -> Result<Paths> {
    let mut base = std::env::home_dir().ok_or(PatchesError::NoHomeDirectory)?;
    base.push(APP_SUPPORT_DIRECTORY);
    let application_data = base.join(DATA_DIRECTORY);
    let user_patches = application_data.join(USER_PATCH_DIRECTORY);
    let system_presets = PathBuf::from(SYSTEM_PATCHES);

    let paths = Paths {
        base,
        application_data,
        user_patches,
        system_patches: system_presets,
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
            .application_data.display
            ());
        std::fs::create_dir(&paths.application_data)?;
    }

    if !paths.user_patches.exists() {
        log::debug!(target: "synthesizer::patches", "User patches directory does not exist. Creating: {}", paths
            .user_patches.display());
        std::fs::create_dir(&paths.user_patches)?;
    }

    if !paths.system_patches.exists() {
        log::debug!(target: "synthesizer::patches", "Application presets directory does not exist. Creating: {}", paths
            .system_patches.display());
        std::fs::create_dir(&paths.system_patches)?;
    }

    Ok(())
}

fn create_patch_from_parameters(parameters: &ModuleParameters) -> String {
    json!(parameters).to_string()
}
