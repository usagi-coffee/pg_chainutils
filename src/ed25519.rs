use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod ED25519 {
    use pgrx::prelude::*;

    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[pg_extern(name = "on_curve", immutable, parallel_safe)]
    fn on_curve(address: &str) -> bool {
        Pubkey::from_str(address)
            .expect("to be valid key")
            .is_on_curve()
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::datum::DatumWithOid;
    use pgrx::prelude::*;

    use anyhow::Result;

    #[pg_test]
    fn curve_test() -> Result<()> {
        let on_curve = Spi::get_one_with_args::<bool>(
            "SELECT ED25519.on_curve($1);",
            &vec![DatumWithOid::from(
                "7dGrdJRYtsNR8UYxZ3TnifXGjGc9eRYLq9sELwYpuuUu",
            )],
        )?;

        assert_eq!(on_curve, Some(true));

        let not_on_curve = Spi::get_one_with_args::<bool>(
            "SELECT ED25519.on_curve($1);",
            &vec![DatumWithOid::from(
                "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
            )],
        )?;

        assert_eq!(not_on_curve, Some(false));

        Ok(())
    }
}
