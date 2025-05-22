pgrx::pg_module_magic!();

mod erc20;
mod erc721;
mod h160;
mod h256;
mod u256;

mod cowswap;
mod sushiswap;
mod uniswap;
mod velodrome;

mod base58;
mod ed25519;
mod spl;

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
