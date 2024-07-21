use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::{self};
use axum::routing::get;
use axum::routing::post;
use axum::routing::put;
use axum::routing::Router;

use tower::ServiceBuilder;

use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use tracing::info;
use tracing::instrument;
use tracing::Instrument;
use tracing::Level;

use crate::core::ServerContext;
use crate::handler::fallback_handler;
use crate::handler::health::HealthHandler;
use crate::handler::kafka_handler::KafkaHandler;
use crate::handler::metrics::MetricsHandler;
use crate::handler::parser::Parser;
use crate::handler::task_handler::TaskHandler;
use crate::handler::task_log_handler::TaskLogHandler;

#[instrument(skip(srv_ctx))]
pub async fn start(srv_ctx: &ServerContext) -> anyhow::Result<()> {
	info!("start server app {:?}", srv_ctx.conf);

	let layer = ServiceBuilder::new()
		.layer(middleware::from_fn(inject_trace_id))
		.layer(TraceLayer::new_for_http())
		.layer(CompressionLayer::new().gzip(true).zstd(true))
		.layer(CorsLayer::permissive());

	let app = Router::new()
		.merge(HealthHandler::route())
		.merge(Parser::route())
		.route("/task/:id", get(TaskHandler::fetch_task))
		.route("/task", post(TaskHandler::create_task))
		.route("/task", put(TaskHandler::update_task))
		.route("/task/count", get(TaskHandler::task_count))
		.route("/task/list", get(TaskHandler::list_task))
		.route("/task/start/:id", post(TaskHandler::start_task))
		.route("/task/stop/:id", post(TaskHandler::stop_task))
		.route("/task/del/:id", put(TaskHandler::delete_task))
		.route("/task/health", post(TaskHandler::health))
		.route("/task/running", get(TaskHandler::running_task))
		.route("/task/log/list", get(TaskLogHandler::task_log_list))
		.route("/task/log/count", get(TaskLogHandler::task_log_count))
		.route("/kafka/topic", post(KafkaHandler::topic_list))
		.route("/kafka/next/group", get(KafkaHandler::group_id))
		.route("/kafka/check", post(KafkaHandler::check))
		.route("/metrics", get(MetricsHandler::metrics))
		.fallback(fallback_handler)
		.layer(layer)
		.with_state(srv_ctx.app_state.clone());

	let listener = tokio::net::TcpListener::bind(srv_ctx.conf.http.get_endpoint()).await?;
	info!("listen http://{}", srv_ctx.conf.http.get_endpoint());
	let conn = srv_ctx.app_state.db_conn.clone();

	// re-start post-running while app is shutdown
	tokio::task::spawn(async move {
		info!("start reload running job start server");
		let _ = TaskHandler::reload_running_job(conn).await;
	});
	//
	axum::serve(listener, app.into_make_service()).await?;
	Ok(())
}

async fn inject_trace_id(
	// mut req: Request,
	// request can use as mutable
	req: Request,
	next: axum::middleware::Next,
) -> axum::response::Response {
	// https://docs.rs/crate/nanoid/0.4.0
	// let alphabet: [char; 36] = [
	// 	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
	// 	'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
	// ];

	let alphabet: [char; 16] =
		['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

	let id = nanoid::nanoid!(32, &alphabet); //=> "4f90d13a42"
	let parent = tracing::Span::current();
	let sp = tracing::span!(parent:&parent,Level::DEBUG, "hydrogen", "trace_id" = id);

	let mut resp = next.run(req).instrument(sp).await;

	let resp_header = resp.headers_mut();

	resp_header.insert("Trace-Id", HeaderValue::from_str(&id).unwrap());
	resp_header.insert("Server-Name", HeaderValue::from_str("Hydrogen").unwrap());
	resp_header.insert(
		"Description",
		HeaderValue::from_str("A processing data tool powered by rust").unwrap(),
	);

	resp
}
