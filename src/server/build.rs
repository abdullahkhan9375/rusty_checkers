use std::process::Command;
use std::path::Path;

fn main() {
    let web_client_path = "../web-client";

    println!("cargo:rerun-if-changed={}/src", web_client_path);
    let status = Command::new("wasm-pack")
        .args(&["build", "--target", "web", "--out-dir", "pkg"])
        .current_dir(web_client_path)
        .status()
        .expect("Failed to run wasm-pack");

    if !status.success() {
        panic!("Failed to build web-client wasm-pack");
    }

    let pkg_path = Path::new(web_client_path).join("pkg");
    println!("cargo:rustc-env=WEB_CLIENT_WASM_DIR={}", pkg_path.display());
}
