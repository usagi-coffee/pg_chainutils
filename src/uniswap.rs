use pgrx::prelude::*;

use num::{
    bigint::{Sign, ToBigInt},
    rational::Ratio,
    BigInt, BigRational,
};

use bigdecimal::BigDecimal;

use anyhow::Result;

pub enum SwapAction {
    BUY,
    SELL,
}

pub struct Swap {
    pub action: SwapAction,
    pub base_amount: BigInt,
    pub quote_amount: BigInt,
}

pub struct Sync {
    pub base_reserve: BigInt,
    pub quote_reserve: BigInt,
}

#[pg_schema]
#[allow(non_snake_case)]
mod Uniswap {
    use pgrx::prelude::*;

    use std::error::Error;
    use std::str::FromStr;

    use ethers::utils::hex;
    use num::{BigInt, Signed};

    use super::decode_swap;
    use super::decode_sync;
    use super::sync_price;


    #[pg_extern(name = "swap_abi", immutable, parallel_safe)]
    fn uni_swap_abi() -> &'static str {
        "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67"
    }

    #[pg_extern(name = "swap_type", immutable, parallel_safe)]
    fn uni_swap_type(data: &str) -> i32 {
        decode_swap(&hex::decode(data).unwrap()).unwrap().action as i32
    }

    #[pg_extern(name = "swap_base_amount", immutable, parallel_safe)]
    fn uni_swap_base_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        let amount = BigInt::from_signed_bytes_be(&hex::decode(data).unwrap()[0..32]);

        Ok(pgrx::AnyNumeric::from_str(
            amount.abs().to_string().as_str(),
        )?)
    }

    #[pg_extern(name = "swap_quote_amount", immutable, parallel_safe)]
    fn uni_swap_quote_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        let amount = BigInt::from_signed_bytes_be(&hex::decode(data).unwrap()[32..64]);

        Ok(pgrx::AnyNumeric::from_str(
            amount.abs().to_string().as_str(),
        )?)
    }

    #[pg_extern(name = "sync_base_reserve", immutable, parallel_safe)]
    fn uni_sync_base_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .base_reserve
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_quote_reserve", immutable, parallel_safe)]
    fn uni_swap_quote_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .quote_reserve
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_price", immutable, parallel_safe)]
    fn uni_sync_price(data: &str, decimals: i64) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            sync_price(&hex::decode(data).unwrap(), decimals)
                .to_string()
                .as_str(),
        )?)
    }
}

#[allow(dead_code)]
fn decode_swap(data: &[u8]) -> Result<Swap> {
    let amount_base = BigInt::from_signed_bytes_be(&data[0..32]);
    let amount_quote = BigInt::from_signed_bytes_be(&data[32..64]);

    let action = match amount_base.gt(&BigInt::from(0)) {
        true => SwapAction::SELL,
        false => SwapAction::BUY,
    };

    Ok(Swap {
        action,
        base_amount: amount_base,
        quote_amount: amount_quote,
    })
}

#[allow(dead_code)]
fn decode_sync(bytes: &[u8]) -> Result<Sync> {
    let x96: Ratio<BigInt> = BigRational::from(BigInt::from(10).pow(29));

    let liquidity = BigInt::from_bytes_be(Sign::Plus, &bytes[96..128]);
    let fixed_liquidity = BigRational::from(liquidity);

    let sqrt = BigInt::from_bytes_be(Sign::Plus, &bytes[64..96]);
    let sqrt_p = BigRational::from(sqrt.clone()) / &x96;

    let base_reserve = &fixed_liquidity * &sqrt_p;
    let quote_reserve = &fixed_liquidity / &sqrt_p;

    Ok(Sync {
        base_reserve: base_reserve.to_integer().to_bigint().unwrap(),
        quote_reserve: quote_reserve.to_integer().to_bigint().unwrap(),
    })
}

fn sync_price(bytes: &[u8], decimals: i64) -> BigDecimal {
    let sqrt = BigInt::from_bytes_be(Sign::Plus, &bytes[64..96]);

    let p2 = BigDecimal::new(sqrt.pow(2), decimals);
    let exp = BigDecimal::new(BigInt::from(2).pow(192), decimals);

    return (p2 / exp).round(decimals);
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use std::str::FromStr;

    use anyhow::Result;

    use super::SwapAction;

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn uni_test_swap() -> Result<()> {
        let data = "0000000000000000000000000000000000000000000069e3a94cbdc95782d980fffffffffffffffffffffffffffffffffffffffffffffffffca2be462fef64520000000000000000000000000000000000000000002dd9e533e8a406c1663add00000000000000000000000000000000000000000000ac695d7b1db89e7cd0ddfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc865";

        let action = Spi::get_one_with_args::<i32>(
            "SELECT Uniswap.swap_type($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        );

        assert_eq!(action, Ok(Some(SwapAction::SELL as i32)));

        let base_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Uniswap.swap_base_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let quote_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Uniswap.swap_quote_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            base_amount,
            Some(pgrx::AnyNumeric::from_str("500048090940207909755264")?)
        );

        assert_eq!(
            quote_amount,
            Some(pgrx::AnyNumeric::from_str("242422221263379374")?)
        );

        Ok(())
    }

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn uni_test_sync() -> Result<()> {
        let data = "00000000000000000000000000000000000000000000017c7f011ed3d569f235fffffffffffffffffffffffffffffffffffffffffffffffffff464d21969b86f0000000000000000000000000000000000000000002cef82ba9345431228373600000000000000000000000000000000000000000000ac695d7b1db89e7cd0ddfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc6d2";

        let reserve_base = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Uniswap.sync_base_reserve($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let reserve_quote = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Uniswap.sync_quote_reserve($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let price = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Uniswap.sync_price($1, 18);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            reserve_base,
            Some(pgrx::AnyNumeric::from_str("442299260600729518413")?)
        );

        assert_eq!(
            reserve_quote,
            Some(pgrx::AnyNumeric::from_str("1498773615814385624748265981")?)
        );

        assert_eq!(
            price,
            Some(pgrx::AnyNumeric::from_str("0.000000470133292265")?)
        );

        Ok(())
    }
}
