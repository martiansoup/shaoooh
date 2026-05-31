#[cfg(not(any(all(target_arch = "aarch64", target_os = "linux"))))]
fn get_config() -> shaoooh::app::Config {
    shaoooh::app::Config::Ditto
}

fn default_arg() -> shaoooh::app::TransitionArg {
    shaoooh::app::TransitionArg::new(
        "Sudowoodo",
        185,
        shaoooh::app::Game::HeartGoldSoulSilver,
        shaoooh::app::Method::SoftResetEncounter,
    )
}

fn main() {
    shaoooh::app::main(&get_config, default_arg());
}
