use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod H160 {
    use ethers::types::{H160, H256};
    use pgrx::prelude::*;

    #[pg_extern(name = "parse", immutable, parallel_safe)]
    fn parse_h160(h160: &str) -> String {
        format!("{:#x}", h160.parse::<H160>().unwrap())
    }

    #[pg_extern(immutable, parallel_safe)]
    fn from_abi(h256: &str) -> String {
        format!("{:#x}", H160::from(h256.parse::<H256>().unwrap()))
    }

    #[pg_extern(immutable, parallel_safe)]
    fn from_h256(h256: &str) -> String {
        format!("{:#x}", H160::from(h256.parse::<H256>().unwrap()))
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use anyhow::Result;

    #[cfg(not(feature = "no-schema-generation"))]
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

        assert_eq!(decoded, Some("0x1111111111111111111111111111111111111111"));

        Ok(())
    }

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn h160_from_h256() -> Result<()> {
        let address = "0x0000000000000000000000001111111111111111111111111111111111111111";

        let decoded = Spi::get_one_with_args::<&str>(
            "SELECT H160.from_h256($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                address.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(decoded, Some("0x1111111111111111111111111111111111111111"));

        Ok(())
    }
}
