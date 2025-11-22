
#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
fn get_config() -> shaoooh::app::Config {
    use std::net::Ipv4Addr;

    shaoooh::app::Config::Bishaan(Ipv4Addr::new(10, 42, 0, 146))
}

#[cfg(not(any(all(target_arch = "aarch64", target_os = "linux"))))]
fn get_config() -> shaoooh::app::Config {
    shaoooh::app::Config::Ditto
}

fn main() {
    shaoooh::app::main(&get_config);
}
