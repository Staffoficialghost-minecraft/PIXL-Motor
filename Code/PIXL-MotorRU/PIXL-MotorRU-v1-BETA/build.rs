fn main() {
    println!("cargo:rerun-if-changed=assets/pixl-logo.ico");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some()
        && std::env::var_os("CARGO_FEATURE_GUI").is_some()
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/pixl-logo.ico");
        resource
            .compile()
            .expect("failed to embed PIXL application icon");
    }
}
