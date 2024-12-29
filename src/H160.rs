use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod H160 {
    use alloy::core::hex;
    use alloy::primitives::{Address, FixedBytes};

    use pgrx::prelude::*;

    #[pg_extern(name = "parse", immutable, parallel_safe)]
    fn parse_h160(h160: &str) -> String {
        hex::encode(h160.parse::<Address>().expect("Failed to parse H160"))
    }

    #[pg_extern(immutable, parallel_safe)]
    fn from_h256(h256: &str) -> String {
        let h256: FixedBytes<32> = h256.parse().expect("Failed to parse H256");
        hex::encode(Address::from_word(h256))
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use anyhow::Result;

    #[pg_test]
    fn h160_parse() -> Result<()> {
        let data = "1111111111111111111111111111111111111111";

        let decoded = Spi::get_one_with_args::<&str>(
            "SELECT H160.parse($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(decoded, Some("1111111111111111111111111111111111111111"));

        Ok(())
    }

    #[pg_test]
    fn h160_from_h256() -> Result<()> {
        let address = "0000000000000000000000001111111111111111111111111111111111111111";

        let decoded = Spi::get_one_with_args::<&str>(
            "SELECT H160.from_h256($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                address.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(decoded, Some("1111111111111111111111111111111111111111"));

        Ok(())
    }
}
