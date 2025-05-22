use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod Base58 {
    use pgrx::prelude::*;

    #[pg_extern(name = "decode", immutable, parallel_safe)]
    fn decode(string: &str) -> Vec<u8> {
        bs58::decode(string)
            .into_vec()
            .expect("string to be base58")
    }

    #[pg_extern(name = "encode", immutable, parallel_safe)]
    fn encode(bytes: Vec<u8>) -> String {
        bs58::encode(&bytes).into_string()
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::datum::DatumWithOid;
    use pgrx::prelude::*;

    use anyhow::Result;

    #[pg_test]
    fn decode_test() -> Result<()> {
        let data = "3QJmnh";

        let decoded = Spi::get_one_with_args::<Vec<u8>>(
            "SELECT base58.decode($1);",
            &vec![DatumWithOid::from(data)],
        );

        assert_eq!(decoded, Ok(Some(vec![0x5d, 0xf6, 0xe0, 0xe2])));
        Ok(())
    }

    #[pg_test]
    fn encode_test() -> Result<()> {
        let data: Vec<u8> = vec![0x5d, 0xf6, 0xe0, 0xe2];

        let encoded = Spi::get_one_with_args::<String>(
            "SELECT base58.encode($1);",
            &vec![DatumWithOid::from(data)],
        );

        assert_eq!(encoded, Ok(Some("3QJmnh".to_string())));
        Ok(())
    }
}
