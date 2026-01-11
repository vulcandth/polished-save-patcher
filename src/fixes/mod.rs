use gb_save_core::Patch;

pub struct FixPatchEntry {
    pub dev_type: u8,
    pub patch: &'static dyn Patch,
}

pub fn polished_fix_patches() -> &'static [FixPatchEntry] {
    &[]
}
