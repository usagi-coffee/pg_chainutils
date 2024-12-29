use pgrx::prelude::*;

use num::{
    bigint::{Sign, ToBigInt},
    rational::Ratio,
    BigInt, BigRational,
};

use bigdecimal::BigDecimal;

use anyhow::Result;

pub enum SwapAction {
    SELL = -1,
    BUY = 1,
}

#[allow(dead_code)]
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

    use alloy::core::hex;
    use num::{BigInt, Signed};

    use super::decode_swap;
    use super::decode_sync;
    use super::sync_price;

    #[pg_extern(name = "swap_type", immutable, parallel_safe)]
    fn uni_swap_type(data: &str) -> i32 {
        decode_swap(&hex::decode(data).unwrap()).unwrap().action as i32
    }

    #[pg_extern(name = "swap_base_amount", immutable, parallel_safe)]
    fn uni_swap_base_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        let amount = BigInt::from_signed_bytes_be(&hex::decode(data).unwrap()[64..96]);

        Ok(pgrx::AnyNumeric::from_str(
            amount.abs().to_string().as_str(),
        )?)
    }

    #[pg_extern(name = "swap_quote_amount", immutable, parallel_safe)]
    fn uni_swap_quote_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        let amount = BigInt::from_signed_bytes_be(&hex::decode(data).unwrap()[96..128]);

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
    fn uni_sync_price(
        data: &str,
        base_decimals: i64,
        quote_decimals: i64,
    ) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            sync_price(&hex::decode(data).unwrap(), base_decimals, quote_decimals)
                .to_string()
                .as_str(),
        )?)
    }
}

#[allow(dead_code)]
fn decode_swap(data: &[u8]) -> Result<Swap> {
    let amount_base = BigInt::from_signed_bytes_be(&data[64..96]);
    let amount_quote = BigInt::from_signed_bytes_be(&data[96..128]);

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

fn decode_sync(bytes: &[u8]) -> Result<Sync> {
    let x96: Ratio<BigInt> = BigRational::from(BigInt::from(10).pow(29));

    let sqrt = BigInt::from_bytes_be(Sign::Plus, &bytes[128..160]);
    let sqrt_p = BigRational::from(sqrt.clone()) / &x96;

    let liquidity = BigInt::from_bytes_be(Sign::Plus, &bytes[160..192]);
    let fixed_liquidity = BigRational::from(liquidity);

    let base_reserve = &fixed_liquidity / &sqrt_p;
    let quote_reserve = &fixed_liquidity * &sqrt_p;

    Ok(Sync {
        base_reserve: base_reserve.to_integer().to_bigint().unwrap(),
        quote_reserve: quote_reserve.to_integer().to_bigint().unwrap(),
    })
}

fn sync_price(bytes: &[u8], base_decimals: i64, quote_decimals: i64) -> BigDecimal {
    let sqrt = BigInt::from_bytes_be(Sign::Plus, &bytes[128..160]);

    let p2 = BigDecimal::new(sqrt.pow(2), quote_decimals);
    let exp = BigDecimal::new(BigInt::from(2).pow(192), quote_decimals);

    let price_ratio = p2 / exp;

    let decimals_difference = base_decimals - quote_decimals;

    if decimals_difference > 0 {
        let adjustment = BigDecimal::new(BigInt::from(10).pow(decimals_difference as u32), 0);
        return (price_ratio * adjustment).round(quote_decimals);
    } else if decimals_difference < 0 {
        let adjustment = BigDecimal::new(BigInt::from(10).pow(decimals_difference.abs() as u32), 0);
        return (price_ratio / adjustment).round(quote_decimals);
    }

    price_ratio.round(quote_decimals)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use std::str::FromStr;

    use anyhow::Result;

    use super::SwapAction;

    #[pg_test]
    fn uni_test_swap() -> Result<()> {
        let data = "000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000069e3a94cbdc95782d980fffffffffffffffffffffffffffffffffffffffffffffffffca2be462fef64520000000000000000000000000000000000000000002dd9e533e8a406c1663add00000000000000000000000000000000000000000000ac695d7b1db89e7cd0ddfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc865";

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

    #[pg_test]
    fn uni_test_sync() -> Result<()> {
        let data = "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000058418f10da628473affffffffffffffffffffffffffffffffffffffffffffffffffffd722236f32722e0000000000000000000000000000000000000000002bc4f31f2528f3970405f300000000000000000000000000000000000000000000ac695d7b1db89e7cd0ddfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc4c4";

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
            "SELECT Uniswap.sync_price($1, 18, 18);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            reserve_base,
            Some(pgrx::AnyNumeric::from_str("1538709118419255752482357599")?)
        );

        assert_eq!(
            reserve_quote,
            Some(pgrx::AnyNumeric::from_str("430819869816330615202")?)
        );

        assert_eq!(
            price,
            Some(pgrx::AnyNumeric::from_str("0.000000446046391448")?)
        );

        Ok(())
    }

    #[pg_test]
    fn uni_test_sync_diff_decimals() -> Result<()> {
        let data = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffa99a52af25fb226800000000000000000000000000000000000000000000000000000002830ac9a200000000000000000000000000000000000000000002ba3e80dffbea705b06590000000000000000000000000000000000000000000000008220d5a03bc02470fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffcebea";

        let price = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Uniswap.sync_price($1, 18, 6);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(price, Some(pgrx::AnyNumeric::from_str("1732.107430")?));

        Ok(())
    }
}
