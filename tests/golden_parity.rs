use std::fs;
use std::path::{Path, PathBuf};

use gb_save_core::SaveBinary;
use gb_save_polished::{
    get_save_version, patch_save_bytes, supported_version_from_u16, symbols_for_version,
    validate_primary_and_backup_checksums,
};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn is_fixture_set_dir(path: &Path) -> bool {
    path.is_dir()
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| !n.eq_ignore_ascii_case("README.md"))
}

fn list_fixture_sets() -> Vec<PathBuf> {
    let root = fixtures_root();
    let mut sets = Vec::new();

    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(_) => return sets,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if is_fixture_set_dir(&path) {
            sets.push(path);
        }
    }

    sets.sort();
    sets
}

fn find_single_sav(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut savs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("sav"))
        })
        .collect();

    savs.sort();
    if savs.len() == 1 {
        savs.pop()
    } else {
        None
    }
}

fn first_diff_offset(a: &[u8], b: &[u8]) -> Option<usize> {
    a.iter().zip(b.iter()).position(|(x, y)| x != y)
}

fn hex_window(bytes: &[u8], center: usize, radius: usize) -> String {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(bytes.len());

    let mut out = String::new();
    for (i, b) in bytes[start..end].iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{b:02X}"));
    }
    out
}

fn closest_v8_sram_symbol(offset: usize) -> Option<(String, usize)> {
    let symbols = symbols_for_version(supported_version_from_u16(8).ok()?).ok()?;

    let mut best: Option<(String, usize)> = None;
    for (name, sym) in symbols.iter() {
        if !gb_save_core::SymbolDatabase::is_sram_address(sym.address) {
            continue;
        }

        let bank_offset = (sym.bank as usize) * 0x2000;
        let address_offset = (sym.address as usize).wrapping_sub(0xA000);
        let abs = bank_offset + address_offset;
        if abs <= offset {
            match &best {
                Some((_, best_abs)) if *best_abs >= abs => {}
                _ => best = Some((name.to_string(), abs)),
            }
        }
    }

    best
}

fn closest_v9_sram_symbol(offset: usize) -> Option<(String, usize)> {
    let symbols = symbols_for_version(supported_version_from_u16(9).ok()?).ok()?;

    let mut best: Option<(String, usize)> = None;
    for (name, sym) in symbols.iter() {
        if !gb_save_core::SymbolDatabase::is_sram_address(sym.address) {
            continue;
        }

        let bank_offset = (sym.bank as usize) * 0x2000;
        let address_offset = (sym.address as usize).wrapping_sub(0xA000);
        let abs = bank_offset + address_offset;
        if abs <= offset {
            match &best {
                Some((_, best_abs)) if *best_abs >= abs => {}
                _ => best = Some((name.to_string(), abs)),
            }
        }
    }

    best
}

fn v8_offset_context(offset: usize) -> String {
    let symbols = match symbols_for_version(supported_version_from_u16(8).unwrap()) {
        Ok(s) => s,
        Err(e) => return format!("<failed to load v8 symbols: {e}>"),
    };

    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("offset=0x{offset:04X}"));

    for name in [
        "sBackupGameData",
        "sBackupGameDataEnd",
        "sGameData",
        "sGameDataEnd",
    ] {
        if let Ok(addr) = symbols.sram_absolute_address(name) {
            parts.push(format!("{name}=0x{a:04X}", a = addr.0));
        }
    }

    if let Ok(backup_start) = symbols.sram_absolute_address("sBackupGameData") {
        if offset >= backup_start.0 as usize {
            parts.push(format!(
                "rel_backup=0x{rel:04X}",
                rel = offset - backup_start.0 as usize
            ));
        }
    }

    if let Ok(game_start) = symbols.sram_absolute_address("sGameData") {
        if offset >= game_start.0 as usize {
            parts.push(format!(
                "rel_game=0x{rel:04X}",
                rel = offset - game_start.0 as usize
            ));
        }
    }

    let player_data_base = symbols
        .wram_relative_to_sram_absolute_address("wPlayerData", "sPlayerData", "wPlayerData")
        .ok();
    if let Some(base) = player_data_base {
        if offset >= base.0 as usize {
            parts.push(format!(
                "rel_wPlayerData=0x{rel:04X}",
                rel = offset - base.0 as usize
            ));
        }
    }

    for wram_symbol in [
        "wObjectStructs",
        "wPlayerStruct",
        "wObject1Struct",
        "wObjectStructsEnd",
        "wRTC",
    ] {
        if let Ok(addr) = symbols.wram_relative_to_sram_absolute_address(
            "wPlayerData",
            "sPlayerData",
            wram_symbol,
        ) {
            parts.push(format!("{wram_symbol}=0x{a:04X}", a = addr.0));
        }
    }

    parts.join(" ")
}

fn v9_offset_context(offset: usize) -> String {
    let symbols = match symbols_for_version(supported_version_from_u16(9).unwrap()) {
        Ok(s) => s,
        Err(e) => return format!("<failed to load v9 symbols: {e}>"),
    };

    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("offset=0x{offset:04X}"));

    for name in [
        "sBackupGameData",
        "sBackupGameDataEnd",
        "sGameData",
        "sGameDataEnd",
    ] {
        if let Ok(addr) = symbols.sram_absolute_address(name) {
            parts.push(format!("{name}=0x{a:04X}", a = addr.0));
        }
    }

    if let Ok(backup_start) = symbols.sram_absolute_address("sBackupGameData") {
        if offset >= backup_start.0 as usize {
            parts.push(format!(
                "rel_backup=0x{rel:04X}",
                rel = offset - backup_start.0 as usize
            ));
        }
    }

    if let Ok(game_start) = symbols.sram_absolute_address("sGameData") {
        if offset >= game_start.0 as usize {
            parts.push(format!(
                "rel_game=0x{rel:04X}",
                rel = offset - game_start.0 as usize
            ));
        }
    }

    let player_data_base = symbols
        .wram_relative_to_sram_absolute_address("wPlayerData", "sPlayerData", "wPlayerData")
        .ok();
    if let Some(base) = player_data_base {
        if offset >= base.0 as usize {
            parts.push(format!(
                "rel_wPlayerData=0x{rel:04X}",
                rel = offset - base.0 as usize
            ));
        }
    }

    for wram_symbol in [
        "wObjectStructs",
        "wPlayerStruct",
        "wObject1Struct",
        "wObjectStructsEnd",
        "wRTC",
    ] {
        if let Ok(addr) = symbols.wram_relative_to_sram_absolute_address(
            "wPlayerData",
            "sPlayerData",
            wram_symbol,
        ) {
            parts.push(format!("{wram_symbol}=0x{a:04X}", a = addr.0));
        }
    }

    parts.join(" ")
}

#[test]
fn v7_to_v8_migration_smoke_updates_version_and_checksums() {
    let sets = list_fixture_sets();
    if sets.is_empty() {
        eprintln!("No fixture sets found under: {}", fixtures_root().display());
        return;
    }

    for set in sets {
        let input_dir = set.join("v7");
        let input = find_single_sav(&input_dir).unwrap_or_else(|| {
            panic!(
                "expected exactly one v7 input .sav under {}",
                input_dir.display()
            )
        });

        let bytes = fs::read(&input).unwrap();
        let patched = patch_save_bytes(bytes, 8, 0)
            .unwrap_or_else(|e| panic!("migration failed for {}: {e}", input.display()));

        let save = SaveBinary::new(patched);
        let version = get_save_version(&save).unwrap();
        assert_eq!(version, 8, "{}", input.display());

        let symbols = symbols_for_version(supported_version_from_u16(8).unwrap()).unwrap();
        validate_primary_and_backup_checksums(&save, &symbols)
            .unwrap_or_else(|e| panic!("checksum validation failed for {}: {e}", input.display()));
    }
}

#[test]
fn golden_parity_v7_inputs_match_expected_v8_output() {
    let sets = list_fixture_sets();
    if sets.is_empty() {
        eprintln!("No fixture sets found under: {}", fixtures_root().display());
        return;
    }

    for set in sets {
        let input_dir = set.join("v7");
        let input = find_single_sav(&input_dir).unwrap_or_else(|| {
            panic!(
                "expected exactly one v7 input .sav under {}",
                input_dir.display()
            )
        });

        let golden_path = set.join("v8").join("patched_save.sav");
        if !golden_path.exists() {
            continue;
        }

        let input_bytes = fs::read(&input).unwrap();
        let expected = fs::read(&golden_path).unwrap();
        let actual = patch_save_bytes(input_bytes, 8, 0).unwrap();

        if expected != actual {
            let offset = first_diff_offset(&expected, &actual).unwrap_or(0);
            let exp_b = expected.get(offset).copied();
            let act_b = actual.get(offset).copied();
            let closest = closest_v8_sram_symbol(offset)
                .map(|(n, a)| format!("{n}@0x{a:04X}"))
                .unwrap_or_else(|| "<none>".to_string());
            let context = v8_offset_context(offset);
            panic!(
                "v8 golden mismatch for set={} offset={} (closest_v8_symbol={}) expected_len={} actual_len={}\n{}\nexpected={}\nactual={}\nexpected_byte={exp_b:?} actual_byte={act_b:?}\nexpected_window={}\nactual_window={}",
                set.display(),
                offset,
                closest,
                expected.len(),
                actual.len(),
                context,
                golden_path.display(),
                input.display(),
                hex_window(&expected, offset, 16),
                hex_window(&actual, offset, 16),
            );
        }
    }
}

#[test]
fn golden_parity_v7_inputs_match_expected_v9_output() {
    let sets = list_fixture_sets();
    if sets.is_empty() {
        eprintln!("No fixture sets found under: {}", fixtures_root().display());
        return;
    }

    for set in sets {
        let input_dir = set.join("v7");
        let input = find_single_sav(&input_dir).unwrap_or_else(|| {
            panic!(
                "expected exactly one v7 input .sav under {}",
                input_dir.display()
            )
        });

        let golden_path = set.join("v9").join("patched_save.sav");
        if !golden_path.exists() {
            continue;
        }

        let input_bytes = fs::read(&input).unwrap();
        let expected = fs::read(&golden_path).unwrap();
        let actual = patch_save_bytes(input_bytes, 9, 0).unwrap();

        if expected != actual {
            let offset = first_diff_offset(&expected, &actual).unwrap_or(0);
            let exp_b = expected.get(offset).copied();
            let act_b = actual.get(offset).copied();
            let closest = closest_v9_sram_symbol(offset)
                .map(|(n, a)| format!("{n}@0x{a:04X}"))
                .unwrap_or_else(|| "<none>".to_string());
            let context = v9_offset_context(offset);
            panic!(
                "v9 golden mismatch for set={} offset={} (closest_v9_symbol={}) expected_len={} actual_len={}\n{}\nexpected={}\nactual={}\nexpected_byte={exp_b:?} actual_byte={act_b:?}\nexpected_window={}\nactual_window={}",
                set.display(),
                offset,
                closest,
                expected.len(),
                actual.len(),
                context,
                golden_path.display(),
                input.display(),
                hex_window(&expected, offset, 16),
                hex_window(&actual, offset, 16),
            );
        }
    }
}

#[test]
fn golden_parity_v7_inputs_match_expected_outputs() {
    let sets = list_fixture_sets();
    if sets.is_empty() {
        eprintln!("No fixture sets found under: {}", fixtures_root().display());
        return;
    }

    for set in sets {
        let input_dir = set.join("v7");
        let input = find_single_sav(&input_dir).unwrap_or_else(|| {
            panic!(
                "expected exactly one v7 input .sav under {}",
                input_dir.display()
            )
        });

        let input_bytes = fs::read(&input).unwrap();

        for target_version in [8u16, 9u16, 10u16] {
            let golden_path = set
                .join(format!("v{target_version}"))
                .join("patched_save.sav");

            if !golden_path.exists() {
                continue;
            }

            let expected = fs::read(&golden_path).unwrap();
            let actual = patch_save_bytes(input_bytes.clone(), target_version, 0).unwrap();

            if expected != actual {
                let offset = first_diff_offset(&expected, &actual);
                panic!(
                    "golden mismatch for set={} target=v{} offset={:?} expected_len={} actual_len={}\nexpected={}\nactual={}",
                    set.display(),
                    target_version,
                    offset,
                    expected.len(),
                    actual.len(),
                    golden_path.display(),
                    input.display(),
                );
            }
        }
    }
}
