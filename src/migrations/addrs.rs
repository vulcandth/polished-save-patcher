use gb_save_core::{Address, SaveResult, SymbolDatabase};

pub(super) struct PolishedAddrs<'a> {
    symbols: &'a SymbolDatabase,
}

impl<'a> PolishedAddrs<'a> {
    pub(super) fn new(symbols: &'a SymbolDatabase) -> Self {
        Self { symbols }
    }

    pub(super) fn options(&self, wram_symbol: &str) -> SaveResult<Address> {
        self.symbols
            .wram_relative_to_sram_absolute_address("wOptions", "sOptions", wram_symbol)
    }

    pub(super) fn player(&self, wram_symbol: &str) -> SaveResult<Address> {
        self.symbols.wram_relative_to_sram_absolute_address(
            "wPlayerData",
            "sPlayerData",
            wram_symbol,
        )
    }

    pub(super) fn map(&self, wram_symbol: &str) -> SaveResult<Address> {
        self.symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", wram_symbol)
    }

    pub(super) fn pokemon(&self, wram_symbol: &str) -> SaveResult<Address> {
        self.symbols.wram_relative_to_sram_absolute_address(
            "wPokemonData",
            "sPokemonData",
            wram_symbol,
        )
    }
}
