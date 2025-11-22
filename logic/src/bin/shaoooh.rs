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

fn main() {
    shaoooh::app::main(&get_config);
}
