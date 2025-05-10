use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod U256 {
    use alloy::core::primitives::U256;

    use pgrx::prelude::*;

    #[pg_extern(name = "parse", immutable, parallel_safe)]
    fn parse_u256(string: &str) -> pgrx::AnyNumeric {
        pgrx::AnyNumeric::try_from(
            string
                .parse::<U256>()
                .expect("string is U256")
                .to_string()
                .as_str(),
        )
        .expect("can be converted to numeric")
    }
}
