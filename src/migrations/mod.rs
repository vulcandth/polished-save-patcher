mod addrs;
mod common;
mod scaffold;
mod structs_v8;
mod structs_v9;
mod v7_to_v8;
mod v7_to_v8_maps;
mod v8_to_v9;
mod v8_to_v9_maps;
mod v9_to_v10;
mod v9_to_v10_maps;

pub use v7_to_v8::MIGRATION_V7_TO_V8;
pub use v8_to_v9::MIGRATION_V8_TO_V9;
pub use v9_to_v10::MIGRATION_V9_TO_V10;

pub fn polished_migrations() -> [&'static dyn gb_save_core::Patch; 3] {
    [
        &MIGRATION_V7_TO_V8,
        &MIGRATION_V8_TO_V9,
        &MIGRATION_V9_TO_V10,
    ]
}
