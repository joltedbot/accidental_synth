use slint_build::CompilerConfiguration;
use std::process::Command;

fn main() {
    // Version is derived purely from git: `<commit-date>.<commit-count>`, plus a
    // `-dirty` suffix when the working tree has uncommitted changes.
    //
    // Re-run triggers:
    //  - reflog (logs/HEAD) changing catches new commits/checkouts/resets, keeping
    //    the count/date current without forcing a recompile on every build.
    //  - ACCIDENTAL_SYNTH_REBUILD lets distribution/build.sh force a fresh version
    //    capture (e.g. to pick up dirty state) by passing a changing value.
    println!("cargo:rerun-if-env-changed=ACCIDENTAL_SYNTH_REBUILD");
    if let Some(reflog) = git_output(&["rev-parse", "--git-path", "logs/HEAD"]) {
        // Only watch it if it exists; a missing rerun-if-changed path makes Cargo
        // re-run the build script on every build.
        if std::path::Path::new(&reflog).exists() {
            println!("cargo:rerun-if-changed={reflog}");
        }
    }

    println!("cargo:rustc-env=APP_VERSION={}", app_version());

    let config = CompilerConfiguration::new().with_style(String::from("fluent"));
    slint_build::compile_with_config("ui/main.slint", config).expect("Failed to compile UI.");
}

fn app_version() -> String {
    let date = git_output(&[
        "show",
        "-s",
        "--format=%cd",
        "--date=format:%Y.%m.%d",
        "HEAD",
    ])
    .unwrap_or_else(|| "0000.00.00".to_string());
    let count = git_output(&["rev-list", "--count", "HEAD"]).unwrap_or_else(|| "0".to_string());
    // `git status --porcelain` prints nothing on a clean tree; git_output maps the
    // empty output to None, so a clean tree is correctly treated as not dirty.
    let dirty = git_output(&["status", "--porcelain"]).is_some();

    let mut version = format!("{date}.{count}");
    if dirty {
        version.push_str("-dirty");
    }
    version
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}
