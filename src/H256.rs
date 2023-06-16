use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod H256 {
    use ethers::core::utils::keccak256;
    use ethers::types::H256;

    use pgrx::prelude::*;

    #[pg_extern(immutable, parallel_safe)]
    fn from_event(event: &str) -> String {
        format!("{:#x}", H256::from(keccak256(event.as_bytes())))
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use anyhow::Result;

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn h256_sync_decode() -> Result<()> {
        let event = "Sync(uint112,uint112)";

        let decoded = Spi::get_one_with_args::<&str>(
            "SELECT H256.from_event($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                event.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            decoded,
            Some("0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1")
        );

        Ok(())
    }
}
