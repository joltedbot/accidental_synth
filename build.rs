use slint_build::CompilerConfiguration;

fn main() {
    let config = CompilerConfiguration::new().with_style(String::from("fluent"));
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
