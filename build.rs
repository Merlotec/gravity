use std::io;

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("InternalName", "gravity.exe")
            // manually set version 1.0.0.0
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        res.compile()?;
    }
    Ok(())
}