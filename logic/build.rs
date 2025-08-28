#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
use std::env;
#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
use std::path::PathBuf;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
fn build() {
    let libdir_path = PathBuf::from("ws2812")
        .canonicalize()
        .expect("Cannot get absolute path");
    let libpio_path = PathBuf::from("rpi-utils/piolib")
        .canonicalize()
        .expect("Cannot get absolute path");

    let headers_path = libdir_path.join("neopixel.h");
    let headers_path_str = headers_path.to_str().expect("Path is not a valid string");

    let obj_path = libdir_path.join("neopixel.o");
    let lib_path = libdir_path.join("libneopixel.a");

    println!("cargo:rustc-link-search={}", libdir_path.to_str().unwrap());
    println!("cargo:rustc-link-search={}", libpio_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=neopixel");
    println!("cargo:rustc-link-lib=pio");

    let cur_dir = std::env::current_dir().expect("Couldn't get current dir");
    std::env::set_current_dir("rpi-utils/piolib").expect("Couldn't set current dir");

    if !std::process::Command::new("cmake")
        .arg(".")
        .output()
        .expect("Couldn't run cmake")
        .status
        .success()
    {
        panic!("Could not run cmake");
    }

    if !std::process::Command::new("make")
        .output()
        .expect("Couldn't run make")
        .status
        .success()
    {
        panic!("Could not run make");
    }

    std::env::set_current_dir(cur_dir).expect("Couldn't set current dir");

    if !std::process::Command::new("clang")
        .arg("-Irpi-utils/piolib/include")
        .arg("-Irpi-utils/piolib/examples")
        .arg("-c")
        .arg("-o")
        .arg(&obj_path)
        .arg(libdir_path.join("neopixel.c"))
        .output()
        .expect("Could not run clang")
        .status
        .success()
    {
        panic!("Could not run clang");
    }

    if !std::process::Command::new("ar")
        .arg("rcs")
        .arg(lib_path)
        .arg(obj_path)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not emit library file");
    }

    let bindings = bindgen::Builder::default()
        .allowlist_function("init_neopixel")
        .allowlist_function("write_pixels")
        .allowlist_type("pio_info_t")
        .header(headers_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

#[cfg(not(all(target_arch = "aarch64", target_os = "linux")))]
fn build() {}

fn opencv_compat() {
    let major = opencv::core::CV_VERSION_MAJOR;
    let minor = opencv::core::CV_VERSION_MINOR;

    println!("cargo::rustc-check-cfg=cfg(opencv5_0_0)");
    println!("cargo::rustc-check-cfg=cfg(opencv4_0_0,opencv4_11_0)");

    // Untested with OpenCV >= 5
    if major == 5 {
        println!("cargo::rustc-cfg=opencv5_0_0");
    }
    if major == 4 {
        println!("cargo::rustc-cfg=opencv4_0_0");
        // Different optional args from 4.10 -> 4.11
        if minor >= 11 {
            println!("cargo::rustc-cfg=opencv4_11_0");
        }
    }
}

fn main() {
    opencv_compat();
    build();
}
