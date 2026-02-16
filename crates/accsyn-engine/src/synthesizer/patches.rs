use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

const APP_SUPPORT_DIRECTORY: &str = "Library/Application Support";
const DATA_DIRECTORY: &str = "AccidentalSynthesizer";
const USER_PATCH_DIRECTORY:&str = "patches";
const PRESETS_DIRECTORY:&str = "presets";

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
        Ok(Self {
            paths,
        })
    }

}


fn create_data_paths() -> Result<Paths> {
    let mut base = std::env::home_dir().ok_or(anyhow!("Could not find home directory"))?;
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
        log::warn!(target: "synthesizer::patches", "Application support directory does not exist. Creating: {}", paths
            .base.display
            ());
        return Err(anyhow!("Application support directory does not exist. {}", paths.base.display()));
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