pgrx::pg_module_magic!();

#[allow(non_snake_case)]
mod H256;

#[allow(non_snake_case)]
mod H160;

#[allow(non_snake_case)]
mod U256;

#[allow(non_snake_case)]
mod ERC20;

#[allow(non_snake_case)]
mod ERC721;

#[allow(non_snake_case)]
mod SPL;

mod cowswap;
mod sushiswap;
mod uniswap;
mod velodrome;

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
