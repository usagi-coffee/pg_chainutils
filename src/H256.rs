use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod H256 {
    use alloy::core::primitives::{keccak256, B256};

    use pgrx::prelude::*;

    #[pg_extern(name = "parse", immutable, parallel_safe)]
    fn parse_h256(string: &str) -> String {
        format!("{:#x}", string.parse::<B256>().unwrap())
    }

    #[pg_extern(immutable, parallel_safe)]
    fn parse_slice(string: &str, start: i64, end: i64) -> String {
        format!(
            "{:#x}",
            string[start as usize..end as usize]
                .parse::<B256>()
                .unwrap()
        )
    }

    #[pg_extern(immutable, parallel_safe)]
    fn from_event(event: &str) -> String {
        format!("{:#x}", keccak256(event.as_bytes()))
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use anyhow::Result;

    #[pg_test]
    fn h256_parse() -> Result<()> {
        let data = "000000000000000000000000a16e02e87b7454126e5e10d957a927a7f5b5d2be";

        let decoded = Spi::get_one_with_args::<&str>(
            "SELECT H256.parse($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            decoded,
            Some("0x000000000000000000000000a16e02e87b7454126e5e10d957a927a7f5b5d2be")
        );

        Ok(())
    }

    #[pg_test]
    fn h256_parse_slice() -> Result<()> {
        let data = "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000a16e02e87b7454126e5e10d957a927a7f5b5d2be";

        let decoded = Spi::get_one_with_args::<&str>(
            "SELECT H256.parse_slice($1, 128, 192);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            decoded,
            Some("0x000000000000000000000000a16e02e87b7454126e5e10d957a927a7f5b5d2be")
        );

        Ok(())
    }

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
