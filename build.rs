fn main() {
    #[cfg(windows)]
    {
        let version = env!("CARGO_PKG_VERSION");
        let pe_version = format!("{}.0", version);
        let homepage = env!("CARGO_PKG_HOMEPAGE");
        let authors = env!("CARGO_PKG_AUTHORS");

        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("CompanyName", "Tanuki Softworks");
        res.set("ProductName", "Uzume");
        res.set("ProductVersion", &pe_version);
        res.set("FileVersion", &pe_version);
        res.set("FileDescription", "Uzume");
        res.set("OriginalFilename", "uzume.exe");
        res.set("InternalName", "uzume");
        res.set(
            "LegalCopyright",
            &format!("Copyright (C) 2025-2026 {}. Licensed under GPL-3.0-or-later.", authors),
        );
        res.set("Comments", homepage);

        if let Err(e) = res.compile() {
            eprintln!("Error al compilar recursos de Windows: {}", e);
        }
    }
}
