use pgrx::prelude::*;

use num::{bigint::Sign, rational::Ratio, BigInt, BigRational, BigUint, Signed};

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
mod Uniswap {
    use pgrx::prelude::*;

    use std::error::Error;
    use std::str::FromStr;

    use ethers::utils::hex;
    use num::{BigInt, Signed};

    use super::decode_swap;
    use super::decode_sync;

    #[pg_extern(name = "swap_type", immutable, parallel_safe)]
    fn uni_swap_type(data: &str) -> i32 {
        decode_swap(&data.as_bytes()).unwrap().action as i32
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
        // SAFETY: abs ensures number is positive
        base_amount: unsafe { amount_base.abs().to_biguint().unwrap_unchecked() },
        // SAFETY: abs ensures number is positive
        quote_amount: unsafe { amount_quote.abs().to_biguint().unwrap_unchecked() },
    })
}

#[allow(dead_code)]
fn decode_sync(bytes: &[u8]) -> Result<Sync> {
    let x96: Ratio<BigInt> = BigRational::from(BigInt::from(10).pow(29));

    let liquidity = BigInt::from_bytes_be(Sign::Plus, &bytes[96..128]);
    let fixed_liquidity = BigRational::from(liquidity);

    let sqrt = BigUint::from_bytes_be(&bytes[64..96]);
    let sqrt_p = BigRational::new(sqrt.into(), 1.into()) / &x96;

    let reserve_base = &fixed_liquidity * &sqrt_p;
    let reserve_quote = &fixed_liquidity / &sqrt_p;

    Ok(Sync {
        base_reserve: reserve_base.to_integer().to_biguint().unwrap(),
        quote_reserve: reserve_quote.to_integer().to_biguint().unwrap(),
    })
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
        let data = "0000000000000000000000000000000000000000000069e3a94cbdc95782d980fffffffffffffffffffffffffffffffffffffffffffffffffca2be462fef64520000000000000000000000000000000000000000002dd9e533e8a406c1663add00000000000000000000000000000000000000000000ac695d7b1db89e7cd0ddfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc865";

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

        assert_eq!(
            reserve_base,
            Some(pgrx::AnyNumeric::from_str("451311132420498713580")?)
        );

        assert_eq!(
            reserve_quote,
            Some(pgrx::AnyNumeric::from_str("1468845801625245179167532601")?)
        );

        Ok(())
    }
}
