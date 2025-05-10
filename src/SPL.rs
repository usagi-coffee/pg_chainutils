use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod SPL {
    use pgrx::prelude::*;

    #[pg_extern(name = "transfer_source", immutable, parallel_safe)]
    fn spl_transfer_source(accounts: Array<&str>) -> String {
        accounts
            .get(0)
            .expect("array to be accessible")
            .expect("0 to be source")
            .into()
    }

    #[pg_extern(name = "transfer_destination", immutable, parallel_safe)]
    fn spl_transfer_destination(accounts: Array<&str>) -> String {
        accounts
            .get(1)
            .expect("array to be accessible")
            .expect("1 to be destination")
            .into()
    }

    #[pg_extern(name = "transfer_value", immutable, parallel_safe)]
    fn spl_transfer_value(data: &str) -> pgrx::AnyNumeric {
        let slice = bs58::decode(data).into_vec().expect("can base58 decode");
        assert_eq!(slice[0], 3, "instruction discriminator should be 3");
        pgrx::AnyNumeric::try_from(
            u64::from_le_bytes(slice[1..9].try_into().expect("slice to be 8 bytes"))
                .to_string()
                .as_str(),
        )
        .expect("can convert u64 to AnyNumeric")
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::datum::DatumWithOid;
    use pgrx::prelude::*;

    use std::str::FromStr;

    use anyhow::Result;

    #[pg_test]
    fn spl_test_transfer_value() -> Result<()> {
        let data = "3T2t139PouH1";

        let decoded = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT SPL.transfer_value($1);",
            &vec![DatumWithOid::from(data)],
        )
        .unwrap();

        assert_eq!(decoded, Some(pgrx::AnyNumeric::from_str("10321")?));

        Ok(())
    }
}
