use pgrx::prelude::*;

use num::BigUint;

use anyhow::Result;

pub enum SwapAction {
    BUY,
    SELL,
}

pub struct Swap {
    pub action: SwapAction,
    pub base_amount: BigUint,
    pub quote_amount: BigUint,
}

pub struct Sync {
    pub base_reserve: BigUint,
    pub quote_reserve: BigUint,
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

    #[pg_extern(immutable, parallel_safe)]
    fn swap_type(data: &str) -> i32 {
        decode_swap(&hex::decode(data).unwrap()).unwrap().action as i32
    }

    #[pg_extern(immutable, parallel_safe)]
    fn swap_base_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .base_amount
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(immutable, parallel_safe)]
    fn swap_quote_amount(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .quote_amount
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(immutable, parallel_safe)]
    fn sync_base_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .base_reserve
                .to_string()
                .as_str(),
        )?)
    }

    #[pg_extern(immutable, parallel_safe)]
    fn sync_quote_reserve(data: &str) -> Result<pgrx::AnyNumeric, Box<dyn Error>> {
        Ok(pgrx::AnyNumeric::from_str(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .quote_reserve
                .to_string()
                .as_str(),
        )?)
    }
}

#[allow(dead_code)]
fn decode_swap(data: &[u8]) -> Result<Swap> {
    let amount_0_in = BigUint::from_bytes_be(&data[32..64]);
    let amount_1_in = BigUint::from_bytes_be(&data[64..96]);

    let amount_0_out = BigUint::from_bytes_be(&data[96..128]);
    let amount_1_out = BigUint::from_bytes_be(&data[128..160]);

    let action = match amount_0_in.gt(&BigUint::from(0u32)) {
        true => SwapAction::SELL,
        false => SwapAction::BUY,
    };

    let max_0 = BigUint::max(amount_0_in, amount_0_out);
    let max_1 = BigUint::max(amount_1_in, amount_1_out);

    Ok(Swap {
        action,
        base_amount: max_1,
        quote_amount: max_0,
    })
}

#[allow(dead_code)]
fn decode_sync(data: &[u8]) -> Result<Sync> {
    let reserve_0 = BigUint::from_bytes_be(&data[0..32]);
    let reserve_1 = BigUint::from_bytes_be(&data[32..64]);

    Ok(Sync {
        base_reserve: reserve_0,
        quote_reserve: reserve_1,
    })
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use super::SwapAction;

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn sushi_test_swap() {
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
            Some(pgrx::AnyNumeric::from(36560694159286225374994 as u128))
        );

        assert_eq!(
            quote_amount,
            Some(pgrx::AnyNumeric::from(129999941597395972 as u128))
        );
    }

    #[cfg(not(feature = "no-schema-generation"))]
    #[pg_test]
    fn sushi_test_sync() {
        let data = "00000000000000000000000000000000000000000000000001cdda4213bbfc040000000000000000000000000000000000000000000007bdf58e3f02e2408f12";

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
            Some(pgrx::AnyNumeric::from(129999941597395972 as u128))
        );

        assert_eq!(
            reserve_quote,
            Some(pgrx::AnyNumeric::from(36560694159286225374994 as u128))
        );
    }
}
