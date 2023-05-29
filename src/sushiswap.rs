use pgrx::prelude::*;

use ethers::types::U256;

use anyhow::Result;

pub enum SwapAction {
    BUY,
    SELL,
}

pub struct Swap {
    pub action: SwapAction,
    pub base_amount: U256,
    pub quote_amount: U256,
}

pub struct Sync {
    pub base_reserve: U256,
    pub quote_reserve: U256,
}

#[pg_schema]
#[allow(non_snake_case)]
mod Sushiswap {
    use pgrx::prelude::*;

    use ethers::utils::hex;

    use super::decode_swap;
    use super::decode_sync;

    #[pg_extern(immutable, parallel_safe)]
    fn swap_type(data: &str) -> i32 {
        decode_swap(&data.as_bytes()).unwrap().action as i32
    }

    #[pg_extern(immutable, parallel_safe)]
    fn swap_base_amount(data: &str) -> pgrx::AnyNumeric {
        pgrx::AnyNumeric::from(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .base_amount
                .as_u128(),
        )
    }

    #[pg_extern(immutable, parallel_safe)]
    fn swap_quote_amount(data: &str) -> pgrx::AnyNumeric {
        pgrx::AnyNumeric::from(
            decode_swap(&hex::decode(data).unwrap())
                .unwrap()
                .quote_amount
                .as_u128(),
        )
    }

    #[pg_extern(immutable, parallel_safe)]
    fn sync_base_reserve(data: &str) -> pgrx::AnyNumeric {
        pgrx::AnyNumeric::from(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .base_reserve
                .as_u128(),
        )
    }

    #[pg_extern(immutable, parallel_safe)]
    fn sync_quote_reserve(data: &str) -> pgrx::AnyNumeric {
        pgrx::AnyNumeric::from(
            decode_sync(&hex::decode(data).unwrap())
                .unwrap()
                .quote_reserve
                .as_u128(),
        )
    }
}

#[allow(dead_code)]
fn decode_swap(data: &[u8]) -> Result<Swap> {
    let amount_0_in = U256::from_big_endian(&data[32..64]);
    let amount_1_in = U256::from_big_endian(&data[64..96]);

    let amount_0_out = U256::from_big_endian(&data[96..128]);
    let amount_1_out = U256::from_big_endian(&data[128..160]);

    let max_0 = U256::max(amount_0_in, amount_0_out);
    let max_1 = U256::max(amount_1_in, amount_1_out);

    let action = match amount_0_in.gt(&U256::zero()) {
        true => SwapAction::SELL,
        false => SwapAction::BUY,
    };

    Ok(Swap {
        action,
        base_amount: max_1,
        quote_amount: max_0,
    })
}

#[allow(dead_code)]
fn decode_sync(data: &[u8]) -> Result<Sync> {
    let reserve_0 = U256::from_big_endian(&data[0..32]);
    let reserve_1 = U256::from_big_endian(&data[32..64]);

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
    fn test_swap() {
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
    fn test_sync() {
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
