use axum::extract::Json;
use axum::routing::get;
use axum::routing::post;
use axum::routing::Router;

use tracing::debug;
use tracing::error;
use tracing::instrument;

use crate::errcode;

use crate::types::ParserPlainTextRequest;
use crate::types::ParserPlainTextResponse;
use crate::types::PropertyPlainTextRequest;
use crate::types::PropertyPlainTextResponse;

use crate::core::AppData;
use crate::core::AppErr;
use crate::extractor::RequestContext;

pub struct Parser;

impl Parser {
	pub fn route<S: Clone + Send + Sync + 'static>() -> Router<S> {
		Router::new()
			.route("/parser", get("parser"))
			.route("/debug/property", post(Parser::plain_text_property))
			.route("/debug/parser", post(Parser::parser))
	}
}

impl Parser {
	#[instrument(skip(req_ctx, req))]
	async fn plain_text_property(
		req_ctx: RequestContext,
		Json(req): Json<PropertyPlainTextRequest>,
	) -> Result<AppData<PropertyPlainTextResponse>, AppErr> {
		debug!("debug plain text property uri {:?} req:{:?}", req_ctx.uri, req);

		// detected sep
		let p = req.to_json_parser();
		// parser data node
		let res = p.property(&req.to_str()).await;
		// process data node
		let res = match res {
			Ok(p) => Ok(p),
			Err(err) => {
				error!("parser property error{:?}", err);
				Err(errcode::PARSER_ERROR.clone())
			}
		};
		// response node result
		crate::util::x_data(res)
	}
}

impl Parser {
	#[instrument(skip(req_ctx, req))]
	async fn parser(
		req_ctx: RequestContext,
		Json(req): Json<ParserPlainTextRequest>,
	) -> Result<AppData<ParserPlainTextResponse>, AppErr> {
		debug!("parser plain text {:?} uri: {:?}", req, req_ctx.uri);

		// build parser
		let p = req.to_parser_json_parser();
		// parser
		let res = p.run(&req.debug_str()).await;
		// process result
		let res = match res {
			Ok(res) => {
				// info!("parser success");
				Ok(res)
			}
			Err(err) => {
				error!("parser error text {} {:?}", &req.debug_str(), err);
				Err(errcode::PARSER_ERROR.clone())
			}
		};
		// response parser result
		crate::util::x_data(res)
	}
}
