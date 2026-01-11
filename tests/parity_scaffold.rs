use std::fs;
use std::path::{Path, PathBuf};

use gb_save_core::SaveBinary;
use gb_save_polished::{
    get_save_version, supported_version_from_u16, symbols_for_version,
    validate_primary_and_backup_checksums,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn collect_sav_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sav_files(&path, out);
            continue;
        }

        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("sav"))
        {
            out.push(path);
        }
    }
}

#[test]
fn detects_version_from_all_fixture_saves() {
    let root = fixture_path("");
    let mut saves = Vec::new();
    collect_sav_files(&root, &mut saves);

    if saves.is_empty() {
        eprintln!("No .sav fixtures found under: {}", root.display());
        return;
    }

    for path in saves {
        let bytes = fs::read(&path).expect("read fixture");
        let save = SaveBinary::new(bytes);
        let version = get_save_version(&save).expect("get version");

        let supported = supported_version_from_u16(version)
            .unwrap_or_else(|e| panic!("unsupported save version for {}: {e}", path.display()));
        let symbols = symbols_for_version(supported)
            .unwrap_or_else(|e| panic!("failed to load symbols for v{version}: {e}"));

        validate_primary_and_backup_checksums(&save, &symbols).unwrap_or_else(|e| {
            panic!(
                "checksum validation failed for {} (v{version}): {e}",
                path.display()
            )
        });
    }
}
