fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("resources/rustyteams.exe.manifest");
        if std::path::Path::new("resources/icon.ico").exists() {
            res.set_icon("resources/icon.ico");
        }
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=winres compile failed: {e}");
        }
    }
}
