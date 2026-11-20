use std::{cell::RefCell, panic::AssertUnwindSafe, sync::Arc};

use futures_util::FutureExt;

use crate::{
    AppState,
    app_config::AppConfig,
    webserver::database::{Database, DatabasePool},
};

tokio::task_local! {
    static POOLS: RefCell<Vec<Arc<dyn DatabasePool>>>;
}

/// Drives each test through database shutdown before dropping its Actix runtime.
pub(crate) struct System(actix_web::rt::SystemRunner);

impl System {
    pub(crate) fn new() -> Self {
        Self(actix_web::rt::System::new())
    }

    pub(crate) fn block_on<F: Future>(&self, test: F) -> F::Output {
        self.0.block_on(POOLS.scope(RefCell::default(), async {
            let result = AssertUnwindSafe(test).catch_unwind().await;
            for pool in POOLS.with(RefCell::take) {
                pool.close().await;
            }
            result.unwrap_or_else(|panic| std::panic::resume_unwind(panic))
        }))
    }
}

pub(crate) async fn connect(config: &AppConfig) -> anyhow::Result<Database> {
    let db = Database::init(config).await?;
    POOLS.with(|pools| pools.borrow_mut().push(db.connection.clone()));
    Ok(db)
}

pub(crate) async fn app_state(config: &AppConfig) -> anyhow::Result<AppState> {
    AppState::init_with_db(config, connect(config).await?).await
}
