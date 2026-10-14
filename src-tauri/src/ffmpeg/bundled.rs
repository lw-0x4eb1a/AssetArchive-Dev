use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
fn binary_bytes() -> Result<Vec<u8>, String> {
    let compressed = include_bytes!("../../bin/ffmpeg/ffmpeg-aa-8.1.3-aa1-universal-apple-darwin.tar.gz");
    let bytes = zune_inflate::DeflateDecoder::new(compressed)
        .decode_gzip()
        .map_err(|error| error.to_string())?;
    let mut archive = tar::Archive::new(Cursor::new(bytes));
    for entry in archive.entries().map_err(|error| error.to_string())? {
        let mut entry = entry.map_err(|error| error.to_string())?;
        if entry.path().map_err(|error| error.to_string())?.as_ref()
            == Path::new("ffmpeg-aa-8.1.3-aa1-universal-apple-darwin/bin/ffmpeg")
        {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).map_err(|error| error.to_string())?;
            return Ok(bytes);
        }
    }
    Err("Bundled FFmpeg executable is missing from the archive".into())
}

#[cfg(any(target_os = "windows", test))]
fn windows_binary_bytes() -> Result<Vec<u8>, String> {
    let compressed = include_bytes!("../../bin/ffmpeg/ffmpeg-aa-8.1.3-aa1-x86_64-pc-windows-msvc.zip");
    let mut archive = zip::ZipArchive::new(Cursor::new(compressed.as_slice()))
        .map_err(|error| error.to_string())?;
    let mut entry = archive
        .by_name("ffmpeg-aa-8.1.3-aa1-x86_64-pc-windows-msvc/bin/ffmpeg.exe")
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}

#[cfg(target_os = "windows")]
fn binary_bytes() -> Result<Vec<u8>, String> {
    windows_binary_bytes()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn binary_bytes() -> Result<Vec<u8>, String> {
    Err("Bundled FFmpeg is not available on this platform; configure a custom executable path".into())
}

pub fn unpack(bin_dir: &Path) -> Result<PathBuf, String> {
    let bytes = binary_bytes()?;
    if bytes.is_empty() {
        return Err("Bundled FFmpeg executable is empty".into());
    }
    std::fs::create_dir_all(bin_dir).map_err(|error| error.to_string())?;
    let path = bin_dir.join(format!("ffmpeg{}", std::env::consts::EXE_SUFFIX));
    if std::fs::read(&path).map(|content| content != bytes).unwrap_or(true) {
        std::fs::write(&path, &bytes)
            .map_err(|error| format!("Failed to install FFmpeg [{}]: {}", path.display(), error))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .map_err(|error| format!("Failed to set FFmpeg executable permissions: {}", error))?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_windows_executable() {
        let bytes = windows_binary_bytes().unwrap();
        assert!(bytes.starts_with(b"MZ"));
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn installs_updates_and_reuses_bundled_executable() {
        let bin_dir = std::env::temp_dir().join(format!("asset-archive-ffmpeg-{}", uuid::Uuid::new_v4()));
        let path = unpack(&bin_dir).unwrap();
        assert!(super::super::run_version(&path).unwrap());
        let modified = path.metadata().unwrap().modified().unwrap();
        unpack(&bin_dir).unwrap();
        assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
        std::fs::write(&path, b"old downloaded ffmpeg").unwrap();
        unpack(&bin_dir).unwrap();
        assert!(super::super::run_version(&path).unwrap());
        std::fs::remove_dir_all(bin_dir).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn restores_executable_permissions_without_rewriting() {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = std::env::temp_dir().join(format!("asset-archive-ffmpeg-{}", uuid::Uuid::new_v4()));
        let path = unpack(&bin_dir).unwrap();
        let modified = path.metadata().unwrap().modified().unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        unpack(&bin_dir).unwrap();
        assert_eq!(path.metadata().unwrap().permissions().mode() & 0o777, 0o755);
        assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
        assert!(super::super::run_version(&path).unwrap());
        std::fs::remove_dir_all(bin_dir).unwrap();
    }
}
