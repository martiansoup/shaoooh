#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
fn get_config() -> shaoooh::app::Config {
    use std::net::Ipv4Addr;

    shaoooh::app::Config::Bishaan(Ipv4Addr::new(10, 42, 0, 146))
}

#[cfg(not(any(all(target_arch = "aarch64", target_os = "linux"))))]
fn get_config() -> shaoooh::app::Config {
    shaoooh::app::Config::Ditto
}

fn default_arg() -> shaoooh::app::TransitionArg {
    shaoooh::app::TransitionArg::new(
        "WormholeGroudon",
        383,
        shaoooh::app::Game::UltraSunUltraMoon,
        shaoooh::app::Method::SoftResetEncounter,
    )
}

fn main() {
    shaoooh::app::main(&get_config, default_arg());
}
