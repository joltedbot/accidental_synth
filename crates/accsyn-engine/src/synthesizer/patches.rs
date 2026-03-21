#!allow[dead_code]
use std::fs::DirEntry;
use crate::synthesizer::ModuleParameters;
use anyhow::{Result, anyhow};
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

const APP_SUPPORT_DIRECTORY: &str = "Library/Application Support";
const DATA_DIRECTORY: &str = "AccidentalSynthesizer";
const USER_PATCH_DIRECTORY: &str = "patches";
const PRESETS_DIRECTORY: &str = "presets";

const PATCH_FILE_EXTENSION: &str = "json";
const INIT_PARAMETERS: &str = include_str!("patches/init.json");

#[derive(Debug, Clone, Error)]
pub enum PatchesError {
    #[error("Application support directory does not exist")]
    NoApplicationSupportDirectory,

    #[error("Could not find home directory")]
    NoHomeDirectory,

    #[error("Patch file already exists")]
    FileAlreadyExists,
}

pub struct Paths {
    base: PathBuf,
    application_data: PathBuf,
    user_patches: PathBuf,
    application_presets: PathBuf,
}
pub struct Patches {
    paths: Paths,
}

impl Patches {
    pub fn new() -> Result<Self> {
        let mut paths = create_data_paths()?;
        initialize_application_storage(&mut paths)?;
        Ok(Self { paths })
    }

    pub fn create_new_patch(&self, name: &str, parameters: &ModuleParameters) -> Result<()> {
        let content = create_patch_from_parameters(parameters);
        let file_name = format!("{}.{}", name, PATCH_FILE_EXTENSION);
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

    fn read_patch_file(&self, name: &str) -> Result<ModuleParameters> {
        let content = read_file(name, &self.paths.user_patches)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn read_preset_file(&self, name: &str) -> Result<ModuleParameters> {
        let content = read_file(name, &self.paths.application_presets)?;
        Ok(serde_json::from_str(&content)?)
    }
}

fn read_file(name: &str, path: &Path) -> Result<String> {
    let file_name = format!("{}.{}", name, PATCH_FILE_EXTENSION);
    let file_path = path.join(file_name);
    Ok(std::fs::read_to_string(file_path)?)
}

fn create_data_paths() -> Result<Paths> {
    let mut base = std::env::home_dir().ok_or(PatchesError::NoHomeDirectory)?;
    base.push(APP_SUPPORT_DIRECTORY);
    let application_data = base.join(DATA_DIRECTORY);
    let user_patches = application_data.join(USER_PATCH_DIRECTORY);
    let system_presets = application_data.join(PRESETS_DIRECTORY);

    let paths = Paths {
        base,
        application_data,
        user_patches,
        application_presets: system_presets,
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

    if !paths.application_presets.exists() {
        log::debug!(target: "synthesizer::patches", "Application presets directory does not exist. Creating: {}", paths
            .application_presets.display());
        std::fs::create_dir(&paths.application_presets)?;
    }

    Ok(())
}

fn create_patch_from_parameters(parameters: &ModuleParameters) -> String {
    json!(parameters).to_string()
}
