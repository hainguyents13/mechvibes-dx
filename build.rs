use std::io;

fn main() -> io::Result<()> {
    // Only compile resources on Windows
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();

        // Set application icon
        res.set_icon("assets/icon.ico");

        // Set application metadata
        res.set("ProductName", "MechvibesDX");
        res.set("FileDescription", "MechvibesDX - Interactive Sound Simulator");
        res.set("CompanyName", "Hai Nguyen");
        res.set("LegalCopyright", "Copyright (C) 2026 Hai Nguyen");

        // Compile the resource file
        res.compile()?;
    }

    Ok(())
}
