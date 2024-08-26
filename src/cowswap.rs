use pgrx::prelude::*;

use num::{bigint::Sign, BigInt};

use ethers::types::Address;

use anyhow::Result;

pub struct Trade {
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: BigInt,
    pub buy_amount: BigInt,
}

#[pg_schema]
#[allow(non_snake_case)]
mod Cowswap {
    use pgrx::prelude::*;

    use std::error::Error;
    use std::str::FromStr;

    use ethers::utils::hex;

    use super::decode_trade;

    #[pg_extern(name = "trade_abi", immutable, parallel_safe)]
    fn cow_trade_abi() -> &'static str {
        "0xa07a543ab8a018198e99ca0184c93fe9050a79400a0a723441f84de1d972cc17"
    }

    #[pg_extern(name = "trade_sell_token", immutable, parallel_safe)]
    fn cow_trade_sell_token(data: &str) -> String {
        format!(
            "{:#x}",
            decode_trade(&hex::decode(data).unwrap())
                .unwrap()
                .sell_token
        )
    }

    #[pg_extern(name = "trade_buy_token", immutable, parallel_safe)]
    fn cow_trade_buy_token(data: &str) -> String {
        format!(
            "{:#x}",
            decode_trade(&hex::decode(data).unwrap()).unwrap().buy_token
        )
    }

    #[pg_extern(name = "trade_sell_amount", immutable, parallel_safe)]
    fn cow_trade_sell_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_trade(&hex::decode(data).unwrap())
                .unwrap()
                .sell_amount
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "trade_buy_amount", immutable, parallel_safe)]
    fn cow_trade_buy_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_trade(&hex::decode(data).unwrap())
                .unwrap()
                .buy_amount
                .to_string()
                .as_str(),
        )?)
    }
}

#[allow(dead_code)]
fn decode_trade(data: &[u8]) -> Result<Trade> {
    let sell_token = Address::from_slice(&data[44..64]);
    let buy_token = Address::from_slice(&data[76..96]);

    let sell_amount = BigInt::from_bytes_be(Sign::Plus, &data[96..128]);
    let buy_amount = BigInt::from_bytes_be(Sign::Plus, &data[128..160]);

    Ok(Trade {
        sell_token,
        buy_token,
        sell_amount,
        buy_amount,
    })
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use std::str::FromStr;

    use anyhow::Result;

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn cow_test_trade() -> Result<()> {
        let data = "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000111111111111111111111111111111111111111100000000000000000000000022222222222222222222222222222222222222220000000000000000000000000000000000000000000012aa0d534b7e47a0fbfe000000000000000000000000000000000000000000000000021ca8b7020921cc000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000003849aaa4ad76b901e03c43fa6703191349f8c33913858c5b4a5e20a4013ba4c86ddfee48c9df6d26c734296c0e6bd02401100a721766c7be270000000000000000";

        let sell_token = Spi::get_one_with_args::<String>(
            "SELECT Cowswap.trade_sell_token($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let buy_token = Spi::get_one_with_args::<String>(
            "SELECT Cowswap.trade_buy_token($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            sell_token,
            Some(String::from("0x1111111111111111111111111111111111111111"))
        );

        assert_eq!(
            buy_token,
            Some(String::from("0x2222222222222222222222222222222222222222"))
        );

        let sell_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Cowswap.trade_sell_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let buy_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Cowswap.trade_buy_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            sell_amount,
            Some(pgrx::AnyNumeric::from_str("88139503378335537363966")?)
        );

        assert_eq!(
            buy_amount,
            Some(pgrx::AnyNumeric::from_str("152181991390388684")?)
        );

        Ok(())
    }
}
