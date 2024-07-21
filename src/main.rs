use clap::Parser;

use hydrogen::conf;
use hydrogen::core::ServerContext;
use hydrogen::log::init_log_subscriber;
use hydrogen::transport::http;

#[derive(clap::Parser, Debug)]
#[command(
	author = "冷饮趁热喝 2356450144@qq.com",
	version = "0.1.0",
	about = "data tools",
	long_about = "a data tools powered by rust",
	next_line_help = true
)]
struct Args {
	/// app name
	#[arg(short, long, default_value = "hydrogen")]
	name: String,
	/// config path
	#[arg(long, default_value = "example/etc/config.toml", short)]
	conf: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// set panic metadata info
	human_panic::setup_panic!();
	let args = Args::parse();
	let cfg = conf::AppConf::from_path(&args.conf)?;
	println!("config path {} config {:#?}", args.conf, cfg);
	let _guard = init_log_subscriber(&cfg.log)?;
	let srv_ctx = ServerContext::new(&cfg).await?;
	http::start(&srv_ctx).await?;
	anyhow::Ok(())
}
