use pgrx::prelude::*;

#[pg_schema]
#[allow(non_snake_case)]
mod H160 {
    use ethers::types::{H160, H256};

    use pgrx::prelude::*;

    #[pg_extern(immutable, parallel_safe)]
    fn from_abi(h256: &str) -> String {
        format!("{:#x}", H160::from(h256.parse::<H256>().unwrap()))
    }

    #[pg_extern(immutable, parallel_safe)]
    fn from_h256(h256: &str) -> String {
        format!("{:#x}", H160::from(h256.parse::<H256>().unwrap()))
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use anyhow::Result;
    use ethers::types::{H160, H256};

    #[test]
    fn test_decode() -> Result<()> {
        let address = "0x0000000000000000000000001111111111111111111111111111111111111111";

        assert_eq!(
            format!("{:#x}", H160::from(address.parse::<H256>().unwrap())),
            "0x1111111111111111111111111111111111111111",
        );

        Ok(())
    }
}
