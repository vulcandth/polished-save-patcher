#![forbid(unsafe_code)]

use anyhow::Result;

struct PolishedCli;

impl gb_save_cli::GameCli for PolishedCli {
    fn detect_version(bytes: &[u8]) -> Result<u16> {
        let save = gb_save_polished::SaveBinary::new(bytes.to_vec());
        Ok(gb_save_polished::get_save_version(&save)?)
    }

    fn patch(bytes: Vec<u8>, target: u16, dev_type: u8) -> Result<Vec<u8>> {
        Ok(gb_save_polished::patch_save_bytes(bytes, target, dev_type)?)
    }

    fn patch_with_log(bytes: Vec<u8>, target: u16, dev_type: u8) -> gb_save_cli::PatchOutcome {
        let outcome = gb_save_polished::patch_save_bytes_with_log(bytes, target, dev_type);
        gb_save_cli::PatchOutcome {
            ok: outcome.error.is_none(),
            bytes: outcome.bytes,
            error: outcome.error,
            logs: outcome.logs,
        }
    }
}

fn main() -> Result<()> {
    gb_save_cli::run::<PolishedCli>()
}
