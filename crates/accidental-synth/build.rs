use slint_build::CompilerConfiguration;
use vergen::{Build, Emitter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = Build::builder().build_date(true).use_local(true).build();
    Emitter::default().add_instructions(&build)?.emit()?;

    let config = CompilerConfiguration::new().with_style(String::from("fluent"));
    slint_build::compile_with_config("ui/main.slint", config).expect("Failed to compile UI.");

    Ok(())
}
