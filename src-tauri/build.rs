fn main() {
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");
    let mut attributes = tauri_build::Attributes::new();
    // Packaged Windows executables request UAC before any startup work. Keep
    // development builds runnable from a normal terminal and test runner.
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        attributes = attributes.windows_attributes(
            tauri_build::WindowsAttributes::new()
                .app_manifest(include_str!("windows-app-manifest.xml")),
        );
    }
    tauri_build::try_build(attributes).expect("failed to run TunnelDesk build script");
}
