use gb_save_core::{NoopPatchLogSink, PatchLogSink, SaveBinary, SaveResult, SymbolDatabase};

pub(super) fn apply_noop_log(
    save: &mut SaveBinary,
    symbols: &SymbolDatabase,
    mut f: impl FnMut(&mut SaveBinary, &SymbolDatabase, &mut dyn PatchLogSink) -> SaveResult<()>,
) -> SaveResult<()> {
    let mut log = NoopPatchLogSink;
    f(save, symbols, &mut log)
}
