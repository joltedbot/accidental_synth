use slint_build::CompilerConfiguration;
use vergen::{BuildBuilder, Emitter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = BuildBuilder::all_build()?;
    Emitter::default().add_instructions(&build)?.emit()?;

    let config = CompilerConfiguration::new().with_style(String::from("fluent"));
    slint_build::compile_with_config("ui/main.slint", config).expect("Failed to compile UI.");

    Ok(())
}
