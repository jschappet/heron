pub mod db;


use std::sync::Once;

#[allow(dead_code)]
static INIT: Once = Once::new();

#[allow(dead_code)]
pub fn init_test_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .is_test(true) // formats output nicely for tests
            .init();
    });
}
