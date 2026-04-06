fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/Cargo.toml");
    println!("cargo:rerun-if-changed=frontend/Dioxus.toml");

    if std::env::var("PROFILE").as_deref() == Ok("release") {
        build_frontend();
    } else {
        ensure_dev_frontend_dist();
    }
}

fn ensure_dev_frontend_dist() {
    let dist_dir = std::path::Path::new("frontend/dist");
    if dist_dir.exists() {
        return;
    }

    std::fs::create_dir_all(dist_dir).expect("failed to create frontend/dist");
    std::fs::write(
        dist_dir.join("index.html"),
        "<!doctype html><title>Forge Dioxus Dev Placeholder</title>",
    )
    .expect("failed to create placeholder frontend/dist/index.html");
}

fn build_frontend() {
    let frontend_dir = std::path::Path::new("frontend");
    if !frontend_dir.exists() {
        panic!("frontend directory not found");
    }

    // Skip if frontend was pre-built (e.g. in a separate Docker stage)
    let dist_dir = frontend_dir.join("dist");
    if dist_dir.exists() && dist_dir.join("index.html").exists() && std::fs::read_dir(&dist_dir).map(|d| d.count() > 1).unwrap_or(false) {
        return;
    }

    let dx = std::process::Command::new("dx")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some();

    if !dx {
        panic!("dioxus-cli (dx) is required to build the Dioxus frontend");
    }

    let status = std::process::Command::new("dx")
        .args(["build", "--platform", "web", "--release"])
        .current_dir(frontend_dir)
        .status()
        .expect("failed to build Dioxus frontend");

    if !status.success() {
        panic!("Dioxus frontend build failed");
    }
}
