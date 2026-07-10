use std::path::{Path, PathBuf};

use crate::tools::ffmpeg::ToolStatus;

pub fn libreoffice_status() -> ToolStatus {
    let path = find_libreoffice();
    ToolStatus {
        name: "LibreOffice",
        available: path.is_some(),
        path: path.map(|value| value.to_string_lossy().to_string()),
        guidance: Some("Install LibreOffice locally to enable DOC/DOCX to PDF conversion."),
    }
}

pub fn find_libreoffice() -> Option<PathBuf> {
    common_install_locations()
        .into_iter()
        .find(|candidate| candidate.is_file())
        .or_else(find_libreoffice_on_path)
}

pub fn find_libreoffice_in(base_dir: &Path) -> Option<PathBuf> {
    let candidates = [
        base_dir.join("soffice.exe"),
        base_dir.join("program").join("soffice.exe"),
        base_dir.join("soffice"),
        base_dir.join("program").join("soffice"),
    ];
    candidates.into_iter().find(|candidate| candidate.is_file())
}

fn common_install_locations() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("LibreOffice")
                .join("program")
                .join("soffice.exe"),
        );
    }
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        candidates.push(
            PathBuf::from(program_files_x86)
                .join("LibreOffice")
                .join("program")
                .join("soffice.exe"),
        );
    }
    candidates.push(PathBuf::from("/usr/bin/soffice"));
    candidates.push(PathBuf::from("/usr/local/bin/soffice"));
    candidates
}

fn find_libreoffice_on_path() -> Option<PathBuf> {
    let executable = if cfg!(windows) { "soffice.exe" } else { "soffice" };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|path| path.join(executable))
            .find(|candidate| candidate.is_file())
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::tools::libreoffice::find_libreoffice_in;

    #[test]
    fn detects_libreoffice_program_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let program = temp_dir.path().join("program");
        fs::create_dir(&program).unwrap();
        let soffice = program.join(if cfg!(windows) { "soffice.exe" } else { "soffice" });
        fs::write(&soffice, b"placeholder").unwrap();

        assert_eq!(find_libreoffice_in(temp_dir.path()), Some(soffice));
    }
}
