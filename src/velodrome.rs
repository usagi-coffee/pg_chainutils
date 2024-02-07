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
mod Velodrome {
    use pgrx::prelude::*;

    use std::error::Error;
    use std::str::FromStr;

    use ethers::utils::hex;

    use super::decode_swap;
    use super::decode_sync;
    use super::sync_price;

    #[pg_extern(name = "swap_abi", immutable, parallel_safe)]
    fn velo_swap_abi() -> &'static str {
        "0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822"
    }

    #[pg_extern(name = "sync_abi", immutable, parallel_safe)]
    fn velo_sync_abi() -> &'static str {
        "0xcf2aa50876cdfbb541206f89af0ee78d44a2abf8d328e37fa4917f982149848a"
    }

    #[pg_extern(name = "swap_type", immutable, parallel_safe)]
    fn velo_swap_type(data: &str) -> i32 {
        decode_swap(&hex::decode(data).unwrap()).unwrap().action as i32
    }

    #[pg_extern(name = "swap_base_amount", immutable, parallel_safe)]
    fn velo_swap_base_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .base_amount
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "swap_quote_amount", immutable, parallel_safe)]
    fn velo_swap_quote_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .quote_amount
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_base_reserve", immutable, parallel_safe)]
    fn velo_sync_base_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .base_reserve
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_quote_reserve", immutable, parallel_safe)]
    fn velo_sync_quote_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .quote_reserve
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(name = "sync_price", immutable, parallel_safe)]
    fn velo_sync_price(
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
    let amount_0_in = BigInt::from_bytes_be(Sign::Plus, &data[64..96]);
    let amount_1_in = BigInt::from_bytes_be(Sign::Plus, &data[96..128]);

    let amount_0_out = BigInt::from_bytes_be(Sign::Plus, &data[128..160]);
    let amount_1_out = BigInt::from_bytes_be(Sign::Plus, &data[160..192]);

    let action = match amount_0_in.gt(&BigInt::from(0u32)) {
        true => SwapAction::SELL,
        false => SwapAction::BUY,
    };

    let max_0 = BigInt::max(amount_0_in, amount_0_out);
    let max_1 = BigInt::max(amount_1_in, amount_1_out);

    Ok(Swap {
        action,
        base_amount: max_0,
        quote_amount: max_1,
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

fn sync_price(bytes: &[u8], base_decimals: i64, quote_decimals: i64) -> BigDecimal {
    let base_reserve = BigInt::from_bytes_be(Sign::Plus, &bytes[64..96]);
    let quote_reserve = BigInt::from_bytes_be(Sign::Plus, &bytes[96..128]);

    let decimal_base_reserve = BigDecimal::new(base_reserve, base_decimals);
    let decimal_quote_reserve = BigDecimal::new(quote_reserve, quote_decimals);

    return (decimal_quote_reserve / decimal_base_reserve).round(quote_decimals);
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
    fn velo_test_swap() -> Result<()> {
        let data = "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000152d02c7e14af680000000000000000000000000000000000000000000000000000001af6004a0664afd0000000000000000000000000000000000000000000000000000000000000000";

        let action = Spi::get_one_with_args::<i32>(
            "SELECT Velodrome.swap_type($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        );

        assert_eq!(action, Ok(Some(SwapAction::BUY as i32)));

        let base_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Velodrome.swap_base_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let quote_amount = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Velodrome.swap_quote_amount($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            base_amount,
            Some(pgrx::AnyNumeric::from_str("121421287949486845")?)
        );

        assert_eq!(
            quote_amount,
            Some(pgrx::AnyNumeric::from_str("100000000000000000000000")?)
        );

        Ok(())
    }

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn velo_test_sync() -> Result<()> {
        let data = "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000307901d1d2a2de57b000000000000000000000000000000000000000000260a3ac38e256b76eb9289";

        let reserve_base = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Velodrome.sync_base_reserve($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        let reserve_quote = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Velodrome.sync_quote_reserve($1);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            reserve_base,
            Some(pgrx::AnyNumeric::from_str("55885199787139392891")?)
        );

        assert_eq!(
            reserve_quote,
            Some(pgrx::AnyNumeric::from_str("45987488812582307820704393")?)
        );

        let price = Spi::get_one_with_args::<pgrx::AnyNumeric>(
            "SELECT Velodrome.sync_price($1, 18, 18);",
            vec![(
                PgOid::BuiltIn(PgBuiltInOids::TEXTOID),
                data.to_string().into_datum(),
            )],
        )
        .unwrap();

        assert_eq!(
            price,
            Some(pgrx::AnyNumeric::from_str("822892.089278442548827008")?)
        );

        Ok(())
    }
}
