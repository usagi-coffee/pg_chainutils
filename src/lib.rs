use pgrx::prelude::*;
pgrx::pg_module_magic!();

#[allow(non_snake_case)]
mod ERC20;

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
