use sqlpage::{AppState, app_config, webserver};

mod basic;
mod common;
mod configuration_docs;
mod cookies;
mod core;
mod data_formats;
mod errors;
mod exec;
mod oidc;
mod parameter_binding;
mod requests;
mod server_timing;
pub mod sql_test_files;
#[path = "../src/test_database.rs"]
mod test_database;
mod transactions;
mod uploads;

#[test]
fn database_fixture_closes_pools_on_success_and_panic() {
    let mut config = common::test_config();
    config.database_url = "sqlite::memory:".to_owned();

    for panics in [false, true] {
        let mut pool = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            test_database::System::new().block_on(async {
                pool = Some(test_database::connect(&config).await.unwrap().connection);
                assert!(!panics, "simulated test failure");
            });
        }));
        assert_eq!(result.is_err(), panics);
        assert_eq!(pool.unwrap().size(), 0);
    }
}
