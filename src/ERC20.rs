use pgrx::prelude::*;

use ethers::types::{Address, H256, U256};

use anyhow::Result;

pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub value: U256,
}

#[pg_schema]
#[allow(non_snake_case)]
mod ERC20 {
    use ethers::types::{H160, H256, U256};

    use ethers::utils::hex;
    use pgrx::prelude::*;

    #[pg_extern(name = "transfer_from", immutable, parallel_safe)]
    fn erc20_transfer_from(topics: Array<&str>) -> String {
        let t1 = topics.get(1).unwrap();

        format!("{:#x}", H160::from(t1.unwrap().parse::<H256>().unwrap()))
    }

    #[pg_extern(name = "transfer_to", immutable, parallel_safe)]
    fn erc20_transfer_to(topics: Array<&str>) -> String {
        let t2 = topics.get(2).unwrap();

        format!("{:#x}", H160::from(t2.unwrap().parse::<H256>().unwrap()))
    }

    #[pg_extern(immutable, parallel_safe)]
    fn transfer_value(data: &str) -> pgrx::AnyNumeric {
        pgrx::AnyNumeric::from(U256::from_big_endian(&hex::decode(data).unwrap()[64..96]).as_u128())
    }
}

#[allow(dead_code)]
fn decode_transfer(topics: Vec<Option<&str>>, data: &[u8]) -> Result<Transfer> {
    Ok(Transfer {
        from: Address::from(topics[1].unwrap().parse::<H256>()?),
        to: Address::from(topics[2].unwrap().parse::<H256>()?),
        value: U256::from_big_endian(&data[64..96]),
    })
}
