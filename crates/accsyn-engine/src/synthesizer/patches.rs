use crate::synthesizer::ModuleParameters;
use crate::synthesizer::constants::{MAX_PATCH_FILE_SIZE, MAX_PATCH_NAME_LENGTH};
use anyhow::{Result, anyhow};
use serde_json::json;
use std::fs::{DirEntry, read_to_string};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

const APP_SUPPORT_DIRECTORY: &str = "Library/Application Support";
const DATA_DIRECTORY: &str = "AccidentalSynthesizer";
const USER_PATCH_DIRECTORY: &str = "patches";
const PATCH_FILE_EXTENSION: &str = "json";
const INIT_PARAMETERS: &str = SYSTEM_PATCHES[0].1;
const SYSTEM_PATCHES: &[(&str, &str)] = &[
    ("Init*", include_str!("patches/init.json")),
    (
        "A Slightly Warmer FM*",
        include_str!("patches/a-slightly-warmer-fm.json"),
    ),
    ("Acid Squelch*", include_str!("patches/acid-squelch.json")),
    ("Acid Time*", include_str!("patches/acid-time.json")),
    (
        "Alien Invasion*",
        include_str!("patches/alien-invasion.json"),
    ),
    ("Ambient Drone*", include_str!("patches/ambient-drone.json")),
    (
        "Arpable Dirty FM*",
        include_str!("patches/arpable-dirty-fm.json"),
    ),
    ("Bright Lead*", include_str!("patches/bright-lead.json")),
    ("Buzz Brass*", include_str!("patches/buzz-brass.json")),
    ("Deep Bass*", include_str!("patches/deep-bass.json")),
    (
        "Dirty Bass Echo*",
        include_str!("patches/dirty-bass-echo.json"),
    ),
    ("Drifting Pad*", include_str!("patches/drifting-pad.json")),
    ("FM Bells*", include_str!("patches/fm-bells.json")),
    ("Plucky Bass*", include_str!("patches/plucky-bass.json")),
    ("Plucky Keys*", include_str!("patches/plucky-keys.json")),
    ("Sci-Fi*", include_str!("patches/sci-fi.json")),
    ("Singing Bowls*", include_str!("patches/singing-bowls.json")),
    ("Slide Bass*", include_str!("patches/slide-bass.json")),
    (
        "Supersaw Swirl*",
        include_str!("patches/supersaw-swirl.json"),
    ),
    (
        "Triangles and Claves*",
        include_str!("patches/triangles-and-claves.json"),
    ),
];

/// Errors that can occur during patch file operations.
#[derive(Debug, Clone, Error, PartialEq)]
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

    /// The patch file path is outside of the user patches directory.
    #[error("Patch file path is outside of the user patches directory")]
    PatchFileOutsideOfUserPatchesDirectory,

    /// The patch file could not be created.
    #[error("Failed to create patch file")]
    FailedToCreatePatchFile,

    /// The patch file could not be written to.
    #[error("Failed to write patch file")]
    FailedToWritePatchFile,

    /// Patch Index out of range.
    #[error("Patch name does not exit {0}")]
    PatchNameDoesNotExist(String),
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
    path: Option<PathBuf>,
}

/// Stores a list of patch names and values
#[derive(Debug, Clone)]
pub struct PatchList {
    presets: Vec<Patch>,
    patches: Vec<Patch>,
}

impl PatchList {
    /// Returns the names of the presets and user patches in the list.
    #[must_use]
    pub fn all_names(&self) -> Vec<String> {
        let mut names = self.preset_names();
        names.append(&mut self.patch_names());
        let zero_index_to_one_index_offset: usize = 1;
        names
            .iter()
            .enumerate()
            .map(|(index, name)| format!("{} - {}", index + zero_index_to_one_index_offset, name))
            .collect()
    }

    /// Returns the list of presets and user patches as a vector
    #[must_use]
    pub fn all(&self) -> Vec<Patch> {
        let mut full_list = self.presets.clone();
        full_list.append(&mut self.patches.clone());
        full_list
    }

    fn preset_names(&self) -> Vec<String> {
        self.presets.iter().map(|p| p.name.clone()).collect()
    }
    fn patch_names(&self) -> Vec<String> {
        self.patches.iter().map(|p| p.name.clone()).collect()
    }

    pub(crate) fn delete_user_patch(&mut self, patch_name: String) -> Result<(), PatchesError> {
        let patch = if let Some(index) = self.patches.iter().position(|p| p.name == patch_name) {
            self.patches.remove(index)
        } else {
            log::warn!(target: "synthesizer::patches", "Patch name does not exist: {patch_name}");
            return Err(PatchesError::PatchNameDoesNotExist(patch_name));
        };

        if let Some(patch_file_path) = patch.path.clone() {
            std::fs::remove_file(&patch_file_path).map_err(|err| {
                log::error!(target: "synthesizer::patches", "Failed to delete patch file {}: {err}", patch_file_path.display());
                PatchesError::FailedToWritePatchFile
            })?;
        }

        Ok(())
    }
}

/// Manages reading and writing synthesizer patch and preset files.
pub struct Patches {
    paths: Paths,
    patches: PatchList,
}

impl Patches {
    /// Creates a new Patches instance, initializing application storage directories if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the application data directories cannot be created or if
    /// storage initialization fails.
    pub fn new() -> Result<Self> {
        let mut paths = create_data_paths()?;
        initialize_application_storage(&mut paths)?;
        let presets = load_presets();
        let patches = load_patches(&paths.user_patches);
        Ok(Self {
            paths,
            patches: PatchList { presets, patches },
        })
    }

    /// Serializes the current module parameters to a new, named patch file.
    ///
    /// # Errors
    ///
    /// Returns an error if it cannot create the patch from the supplied name in the patch directory
    pub fn save_patch(
        &mut self,
        name: &str,
        parameters: &ModuleParameters,
    ) -> Result<(), PatchesError> {
        self.create_new_patch(name, parameters)?;
        self.patches = self.patch_list();
        log::info!(target: "synthesizer::patches", "Patch saved: {name}");
        Ok(())
    }

    fn create_new_patch(
        &self,
        name: &str,
        parameters: &ModuleParameters,
    ) -> Result<(), PatchesError> {
        let content = create_patch_from_parameters(parameters);
        let mut patch_file_path = self.paths.user_patches.clone();
        let sanitized_name = sanitize_name(name);
        patch_file_path.push(sanitized_name);
        patch_file_path.set_extension(PATCH_FILE_EXTENSION);

        validate_patch_file_path(&mut patch_file_path, &self.paths.user_patches)?;

        let mut handle = std::fs::File::create(&patch_file_path).map_err(|err| {
            log::error!(target: "synthesizer::patches", "Failed to create patch file {}: {err}", patch_file_path
                .display());
            PatchesError::FailedToCreatePatchFile
        })?;

        handle.write_all(content.as_bytes()).map_err(|err| {
            log::error!(target: "synthesizer::patches", "Failed to write patch file {}: {err}", patch_file_path
                .display());
            PatchesError::FailedToWritePatchFile
        })?;

        log::info!(target: "synthesizer::patches", "Created new patch file: {}", patch_file_path.display());

        Ok(())
    }

    /// Generates and returns a `PatchList` containing presets and user-defined patches.
    #[must_use]
    pub fn patch_list(&self) -> PatchList {
        let presets = load_presets();
        let patches = load_patches(&self.paths.user_patches);
        PatchList { presets, patches }
    }

    /// Returns the names of only the user patches
    #[must_use]
    pub fn user_patch_names(&self) -> Vec<String> {
        self.patches.patch_names()
    }

    /// Delete a patch file from the user patches directory by patch index from all patches list
    ///
    /// # Errors
    ///
    /// Returns an error if the new the provided patch name cannot be deleted or does not exist
    pub fn delete_patch_by_name(&mut self, patch_name: String) -> Result<(), PatchesError> {
        self.patches.patches = load_patches(&self.paths.user_patches);
        self.patches.delete_user_patch(patch_name)?;
        Ok(())
    }
}

/// Loads a preset from the system preset patches by preset index. See `SYSTEM_PATCHES` for the index values
///
/// # Errors
///
/// Returns an error if patch's index is incorrect or if the json is incorrect and cannot be serialized back into
/// module parameters
pub fn get_module_parameters_from_patch_index(
    index: usize,
    patch_list: &PatchList,
) -> Result<ModuleParameters> {
    let patch = &patch_list.all()[index];
    let preset = serde_json::from_str(&patch.content).map_err(|err| {
        log::error!(target: "synthesizer::patches", "Failed to parse preset '{}' (index {index}): {err}", patch.name);
        err
    })?;
    log::info!(target: "synthesizer::patches", "Loaded preset '{}' (index {index})", patch.name);
    Ok(preset)
}

fn validate_patch_file_path(
    patch_file_path: &mut Path,
    expected_patch_directory: &Path,
) -> Result<(), PatchesError> {
    if let Some(patch_path_parent) = patch_file_path.parent() {
        if patch_path_parent != expected_patch_directory {
            log::warn!(target: "synthesizer::patches", "Patch file path is outside of the user patches directory: {}", patch_file_path.display());
            return Err(PatchesError::PatchFileOutsideOfUserPatchesDirectory);
        }
    } else {
        log::warn!(target: "synthesizer::patches", "Patch file path has no parent directory: {}", patch_file_path.display());
        return Err(PatchesError::PatchFileOutsideOfUserPatchesDirectory);
    }

    if patch_file_path.exists() {
        log::warn!(target: "synthesizer::patches", "Patch file already exists: {}", patch_file_path.display());
        return Err(PatchesError::FileAlreadyExists);
    }

    Ok(())
}

fn sanitize_name(name: &str) -> String {
    let sized_name = if name.len() > MAX_PATCH_NAME_LENGTH {
        name.trim()
            .chars()
            .take(MAX_PATCH_NAME_LENGTH)
            .collect::<String>()
    } else {
        name.trim().to_string()
    };

    let stripped_name = sized_name.replace(['.', '*'], "");
    sanitize_filename::sanitize(stripped_name)
}

fn load_presets() -> Vec<Patch> {
    SYSTEM_PATCHES
        .iter()
        .map(|(name, content)| Patch {
            name: name.to_string(),
            content: content.to_string(),
            path: None,
        })
        .collect()
}

/// Create patch collection from user patches directory.
fn load_patches(patch_directory: &Path) -> Vec<Patch> {
    log::debug!(target: "synthesizer::patches", "Loading presets");
    let mut patches = vec![];

    log::debug!(target: "synthesizer::patches", "Loading patches from directory: {}", patch_directory.display());

    if let Ok(entries) = patch_directory.read_dir() {
        // Sort by modification time (newest first)
        let mut files = entries
            .filter_map(std::result::Result::ok)
            .collect::<Vec<DirEntry>>();
        files.sort_by_key(|e| e.metadata().ok()?.modified().ok());

        files.iter().filter(|entry| {
                entry.metadata().map_or_else(|error|{
                    log::warn!(target: "synthesizer::patches", "Failed to read metadata for patch file {}: {error}", entry.path().display());
                    false
                }, |m| m.len() < MAX_PATCH_FILE_SIZE)
            })
            .filter(|entry| !entry.path().is_symlink())
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| {
                let extension = entry.path().extension()?.to_string_lossy().to_string();
                if extension == PATCH_FILE_EXTENSION {
                    Some(entry)
                }  else {
                    None
                }
            })
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_stem()?.to_string_lossy().to_string();
                let sanitized_name = sanitize_name(&name);
                let content = match read_to_string(path.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        log::warn!(target: "synthesizer::patches", "Failed to read patch file {}: {e}", path.display());
                        return None;
                    }
                };
                Some(Patch {
                    name: sanitized_name,
                    content,
                    path: Some(path)
                })
            })
            .for_each(|patch| patches.push(patch));
    }
    patches
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
    let mut base = dirs::home_dir().ok_or(PatchesError::NoHomeDirectory)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "accsyn_patches_test_{}_{label}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write_file(&self, filename: &str, content: &str) -> PathBuf {
            let path = self.0.join(filename);
            let mut f = fs::File::create(&path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    // --- sanitize_name ---

    #[test]
    fn sanitize_name_plain_name_is_unchanged() {
        assert_eq!(sanitize_name("My Patch"), "My Patch");
    }

    #[test]
    fn sanitize_name_trims_surrounding_whitespace() {
        assert_eq!(sanitize_name("  My Patch  "), "My Patch");
    }

    #[test]
    fn sanitize_name_removes_dots() {
        assert_eq!(sanitize_name("My.Patch"), "MyPatch");
    }

    #[test]
    fn sanitize_name_removes_asterisks() {
        // asterisk is the system-patch marker; user names must not contain it
        assert_eq!(sanitize_name("Init*"), "Init");
    }

    #[test]
    fn sanitize_name_removes_dots_and_asterisks_together() {
        assert_eq!(sanitize_name("Patch.*Name"), "PatchName");
    }

    #[test]
    fn sanitize_name_at_exact_max_length_is_not_truncated() {
        let name = "A".repeat(MAX_PATCH_NAME_LENGTH);
        assert_eq!(sanitize_name(&name).len(), MAX_PATCH_NAME_LENGTH);
    }

    #[test]
    fn sanitize_name_over_max_length_is_truncated() {
        let name = "B".repeat(MAX_PATCH_NAME_LENGTH + 10);
        let result = sanitize_name(&name);
        assert!(
            result.len() <= MAX_PATCH_NAME_LENGTH,
            "expected at most {MAX_PATCH_NAME_LENGTH} chars, got {}",
            result.len()
        );
    }

    #[test]
    fn sanitize_name_path_traversal_has_no_separators_or_dots() {
        let result = sanitize_name("../evil");
        assert!(
            !result.contains('/'),
            "sanitized name must not contain path separator"
        );
        assert!(
            !result.contains(".."),
            "sanitized name must not contain relative path sequence"
        );
    }

    #[test]
    fn sanitize_name_all_special_chars_produces_empty_string() {
        assert_eq!(sanitize_name("...**"), "");
    }

    // --- load_patches ---

    #[test]
    fn load_patches_nonexistent_directory_returns_empty() {
        let path = Path::new("/tmp/accsyn_does_not_exist_12345");
        assert!(load_patches(path).is_empty());
    }

    #[test]
    fn load_patches_empty_directory_returns_empty() {
        let dir = TempDir::new("empty");
        assert!(load_patches(dir.path()).is_empty());
    }

    #[test]
    fn load_patches_loads_valid_json_file() {
        let dir = TempDir::new("valid");
        let expected_path = dir.write_file("My Patch.json", r#"{"test":true}"#);
        let result = load_patches(dir.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "My Patch");
        assert_eq!(result[0].content, r#"{"test":true}"#);
        assert_eq!(result[0].path, Some(expected_path));
    }

    #[test]
    fn load_patches_skips_non_json_extension() {
        let dir = TempDir::new("non_json");
        dir.write_file("My Patch.txt", "content");
        assert!(load_patches(dir.path()).is_empty());
    }

    #[test]
    fn load_patches_sanitizes_dots_in_filename() {
        let dir = TempDir::new("dots_in_name");
        dir.write_file("My.Patch.json", r"{}");
        let result = load_patches(dir.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "MyPatch");
    }

    #[test]
    fn load_patches_skips_file_exceeding_size_limit() {
        let dir = TempDir::new("oversized");

        // Constant value MAX_PATCH_FILE_SIZE is manually set and will always be well below the 32 bit usize limit
        #[allow(clippy::cast_possible_truncation)]
        let content = "x".repeat(MAX_PATCH_FILE_SIZE as usize + 1);
        dir.write_file("Big Patch.json", &content);
        assert!(load_patches(dir.path()).is_empty());
    }

    #[test]
    fn load_patches_skips_symlinks() {
        use std::os::unix::fs::symlink;
        let dir = TempDir::new("symlinks");
        let real_path = dir.write_file("real.json", r"{}");
        let link_path = dir.path().join("link.json");
        symlink(&real_path, &link_path).unwrap();
        let result = load_patches(dir.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "real");
    }
}
