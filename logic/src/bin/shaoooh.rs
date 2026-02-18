#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
fn get_config() -> shaoooh::app::Config {
    let paths = shaoooh::app::CaptureControlPaths::new(
        "/dev/video0".to_string(),
        "/dev/ttyAMA0".to_string(),
    );
    shaoooh::app::Config::Shaoooh(paths)
}

#[cfg(not(any(all(target_arch = "aarch64", target_os = "linux"))))]
fn get_config() -> shaoooh::app::Config {
    shaoooh::app::Config::Ditto
}

fn default_arg() -> shaoooh::app::TransitionArg {
    shaoooh::app::TransitionArg::new(
        "Wailmer",
        320,
        shaoooh::app::Game::RubySapphire,
        shaoooh::app::Method::RandomEncounter,
    )
    //shaoooh::app::TransitionArg::new("AllTheSpindaPt2", 327, shaoooh::app::Game::RubySapphire, shaoooh::app::Method::RandomEncounter)
}

fn main() {
    shaoooh::app::main(&get_config, default_arg());
}
