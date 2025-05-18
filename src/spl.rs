use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod SPL {
    use pgrx::prelude::*;
    use solana_sdk::{pubkey, pubkey::Pubkey};
    use std::str::FromStr;

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

    const PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
    const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    #[pg_extern(name = "token_account", immutable, parallel_safe)]
    fn spl_token_account(mint: String, address: String) -> Result<String, anyhow::Error> {
        let mint = Pubkey::from_str(mint.as_str())?;
        let address = Pubkey::from_str(address.as_str())?;

        let seeds = [
            &address.to_bytes()[..],
            &TOKEN_PROGRAM_ID.to_bytes()[..],
            &mint.to_bytes()[..],
        ];

        Ok(Pubkey::find_program_address(&seeds, &PROGRAM_ID)
            .0
            .to_string())
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

    #[pg_test]
    fn spl_account_derive_test() -> Result<()> {
        let mint = "CY2E69dSG9vBsMoaXDvYmMDSMEP4SZtRY1rqVQ9tkNDu";
        let address = "D4RU5YKeMuHc25rrgmbggwr95DaogDe8d8hFRD2CNQXb";

        let decoded = Spi::get_one_with_args::<String>(
            "SELECT SPL.token_account($1, $2);",
            &vec![DatumWithOid::from(mint), DatumWithOid::from(address)],
        )
        .unwrap();

        assert_eq!(
            decoded,
            Some(String::from("45TCoQ8FSp4USRsGkuDKVQmZs878wgPAmhKYJBGWnEYd"))
        );

        Ok(())
    }
}
