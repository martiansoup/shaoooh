#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
fn get_config() -> shaoooh::app::Config {
    let paths = shaoooh::app::CaptureControlPaths::new(
        "/dev/video0".to_string(),
        "../control/switch/_control_pipe".to_string(),
    );
    shaoooh::app::Config::Gyaaas(paths)
}

#[cfg(not(any(all(target_arch = "aarch64", target_os = "linux"))))]
fn get_config() -> shaoooh::app::Config {
    shaoooh::app::Config::Ditto
}

fn default_arg() -> shaoooh::app::TransitionArg {
    shaoooh::app::TransitionArg::new(
        "Bulbasaur",
        1,
        shaoooh::app::Game::FireRedLeafGreen,
        shaoooh::app::Method::SoftResetGift,
    )
}

fn main() {
    shaoooh::app::main(&get_config, default_arg());
}
