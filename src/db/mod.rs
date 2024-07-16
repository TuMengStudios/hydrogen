use std::time::Duration;

use anyhow::Context;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

use tracing::debug;
use tracing::instrument;

use crate::conf;

// init database pool
#[instrument(skip(cfg))]
pub async fn init_db_conn(cfg: &conf::DBConf) -> anyhow::Result<MySqlPool> {
	debug!("init mysql connection {cfg:?}");

	let conn = MySqlPoolOptions::new()
		// max connections
		.max_connections(cfg.max_conn)
		//
		.max_lifetime(Duration::from_secs(59))
		// client connect max idle
		.idle_timeout(Duration::from_secs(60))
		// connect db
		.connect(&cfg.dsn)
		.await
		.with_context(|| format!("connect db {}", cfg.dsn))?;

	Ok(conn)
}
