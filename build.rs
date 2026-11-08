fn main() {
    println!("cargo:rerun-if-changed=assets/pocket-pal.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winres::WindowsResource::new()
            .set_icon("assets/pocket-pal.ico")
            .compile()
            .expect("failed to embed the Pocket Pal Windows icon");
    }
}
