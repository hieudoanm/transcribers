use std::process::Command;

fn main() {
    let build_time = Command::new("date")
        .args(["-u", "+%Y.%m.%d.%H.%M.%S"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=BUILD_TIME={build_time}");
}
