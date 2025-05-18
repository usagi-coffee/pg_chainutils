use pgrx::prelude::*;

use alloy::primitives::{Address, U256};

use anyhow::Result;

#[pg_schema]
#[allow(non_snake_case)]
mod ERC721 {
    use alloy::core::hex;
    use alloy::primitives::{Address, FixedBytes, U256};

    use pgrx::prelude::*;

    #[pg_extern(name = "transfer_from", immutable, parallel_safe)]
    fn erc721_transfer_from(topics: Array<&str>) -> String {
        let t1 = topics.get(1).expect("Invalid topics");

        let h256: FixedBytes<32> = t1.unwrap().parse().expect("Failed to parse H256");
        hex::encode(Address::from_word(h256))
    }

    #[pg_extern(name = "transfer_to", immutable, parallel_safe)]
    fn erc721_transfer_to(topics: Array<&str>) -> String {
        let t2 = topics.get(2).expect("Invalid topics");

        let h256: FixedBytes<32> = t2.unwrap().parse().expect("Failed to parse H256");
        hex::encode(Address::from_word(h256))
    }

    #[pg_extern(immutable, parallel_safe)]
    fn transfer_token(topics: Array<&str>) -> pgrx::AnyNumeric {
        let t3 = topics.get(3).expect("Invalid topics");

        pgrx::AnyNumeric::try_from(t3.unwrap().parse::<U256>().unwrap().to_string().as_str())
            .expect("Failed to convert U256 to AnyNumeric")
    }
}

#[allow(dead_code)]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub token_id: U256,
}

#[allow(dead_code)]
fn decode_transfer(topics: Vec<Option<&str>>, data: &[u8]) -> Result<Transfer> {
    Ok(Transfer {
        from: topics[1].unwrap().parse::<Address>()?,
        to: topics[2].unwrap().parse::<Address>()?,
        token_id: U256::from_be_slice(&data[64..96]),
    })
}
