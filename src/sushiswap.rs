use pgrx::prelude::*;

use bigdecimal::BigDecimal;
use num::{bigint::Sign, BigInt};

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
mod Sushiswap {
    use pgrx::prelude::*;

    use std::error::Error;
    use std::str::FromStr;

    use ethers::utils::hex;

    use super::decode_swap;
    use super::decode_sync;
    use super::sync_price;

    #[pg_extern(name = "swap_abi", immutable, parallel_safe)]
    fn sushi_swap_abi() -> &'static str {
        "0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822"
    }

    #[pg_extern(name = "sync_abi", immutable, parallel_safe)]
    fn sushi_sync_abi() -> &'static str {
        "0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1"
    }

    #[pg_extern(name = "swap_type", immutable, parallel_safe)]
    fn sushi_swap_type(data: &str) -> i32 {
        decode_swap(&hex::decode(data).unwrap()).unwrap().action as i32
    }

    #[pg_extern(name = "swap_base_amount", immutable, parallel_safe)]
    fn sushi_swap_base_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .base_amount
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "swap_quote_amount", immutable, parallel_safe)]
    fn sushi_swap_quote_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .quote_amount
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_base_reserve", immutable, parallel_safe)]
    fn sushi_sync_base_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .base_reserve
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_quote_reserve", immutable, parallel_safe)]
    fn sushi_sync_quote_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .quote_reserve
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_price", immutable, parallel_safe)]
    fn sushi_sync_price(data: &str, decimals: i64) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            sync_price(&hex::decode(data).unwrap(), decimals)
                .to_string()
                .as_str(),
        )?)
    }
}

#[allow(dead_code)]
fn decode_swap(data: &[u8]) -> Result<Swap> {
    let amount_0_in = BigInt::from_bytes_be(Sign::Plus, &data[32..64]);
    let amount_1_in = BigInt::from_bytes_be(Sign::Plus, &data[64..96]);

    let amount_0_out = BigInt::from_bytes_be(Sign::Plus, &data[96..128]);
    let amount_1_out = BigInt::from_bytes_be(Sign::Plus, &data[128..160]);

    let action = match amount_0_in.gt(&BigInt::from(0u32)) {
        true => SwapAction::SELL,
        false => SwapAction::BUY,
    };

    let max_0 = BigInt::max(amount_0_in, amount_0_out);
    let max_1 = BigInt::max(amount_1_in, amount_1_out);

    Ok(Swap {
        action,
        base_amount: max_1,
        quote_amount: max_0,
    })
}

#[allow(dead_code)]
fn decode_sync(data: &[u8]) -> Result<Sync> {
    let base_reserve = BigInt::from_bytes_be(Sign::Plus, &data[64..96]);
    let quote_reserve = BigInt::from_bytes_be(Sign::Plus, &data[96..128]);

    Ok(Sync {
        base_reserve,
        quote_reserve,
    })
}

fn sync_price(bytes: &[u8], decimals: i64) -> BigDecimal {
    let base_reserve = BigInt::from_bytes_be(Sign::Plus, &bytes[64..96]);
    let quote_reserve = BigInt::from_bytes_be(Sign::Plus, &bytes[96..128]);

    let decimal_base_reserve = BigDecimal::new(base_reserve, decimals);
    let decimal_quote_reserve = BigDecimal::new(quote_reserve, decimals);

    return (decimal_quote_reserve / decimal_base_reserve).round(decimals);
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
    fn sushi_test_swap() -> Result<()> {
        let data = "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001cdda4213bbfc040000000000000000000000000000000000000000000007bdf58e3f02e2408f120000000000000000000000000000000000000000000000000000000000000000";

        let action = Spi::get_one_with_args::<i32>(
            "SELECT Sushiswap.swap_type($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        );

        assert_eq!(action, Ok(Some(SwapAction::SELL as i32)));

        let base_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Sushiswap.swap_base_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let quote_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Sushiswap.swap_quote_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            base_amount,
            Some(pgrx::AnyNumeric::from_str("36560694159286225374994")?)
        );

        assert_eq!(
            quote_amount,
            Some(pgrx::AnyNumeric::from_str("129999941597395972")?)
        );

        Ok(())
    }

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn sushi_test_sync() -> Result<()> {
        let data = "000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000030a017596c201728ecfb31300000000000000000000000000000000000000000000009a2946f7338c7c7108";

        let reserve_base = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Sushiswap.sync_base_reserve($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let reserve_quote = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Sushiswap.sync_quote_reserve($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            reserve_base,
            Some(pgrx::AnyNumeric::from_str("940551179158967834289091347")?)
        );

        assert_eq!(
            reserve_quote,
            Some(pgrx::AnyNumeric::from_str("2843772923755968098568")?)
        );

        let price = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Sushiswap.sync_price($1, 18);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            price,
            Some(pgrx::AnyNumeric::from_str("0.000003023517472275")?)
        );

        Ok(())
    }
}
