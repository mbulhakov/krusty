use chrono::Duration;
use deadpool::managed::Pool;
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::LevelFilter;
use percentage::Percentage;
use std::env;
use std::{convert::Infallible, net::SocketAddr};
use teloxide::prelude::*;
use tracing_unwrap::ResultExt;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use krusty::bot::start_bot;

async fn dummy(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("")))
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(db_uri: &str) {
    PgConnection::establish(db_uri)
        .expect("Failed to obtain connection")
        .run_pending_migrations(MIGRATIONS)
        .unwrap_or_log();
}

#[tokio::main]
async fn main() {
    let mut config = log4rs::config::load_config_file("log4rs.yml", Default::default())
        .expect("Failed to load logger config");

    let log_level = env::var("LOG_LEVEL").map_or_else(
        |_| LevelFilter::Info,
        |x| match x.to_lowercase().as_str() {
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            "trace" => LevelFilter::Trace,
            "off" => LevelFilter::Off,
            unknown => panic!("Unrecognized log level: '{unknown}'"),
        },
    );
    config.root_mut().set_level(log_level);
    log4rs::init_config(config).expect("Failed to init logger");

    log::info!("Starting bot...");
    let bot = Bot::from_env();

    let media_timeout_sec =
        env::var("MEDIA_TIMEOUT_SEC").map_or_else(|_| 30, |x| x.parse().unwrap());
    let ignore_message_older_than_sec =
        env::var("IGNORE_MESSAGE_OLDER_THAN_SEC").map_or_else(|_| 60, |x| x.parse().unwrap());

    if media_timeout_sec > ignore_message_older_than_sec {
        panic!("Media timeout is greater than 'ignore message older than': {media_timeout_sec} > {ignore_message_older_than_sec}");
    }

    let media_being_sent_chance_in_percent =
        env::var("MEDIA_SEND_CHANCE_IN_PERCENT").map_or_else(|_| 50, |x| x.parse().unwrap());

    let similarity_threshold_in_decimal =
        env::var("MAX_ACCEPTED_SCORE_SIMILARITY").map_or_else(|_| 0.26f64, |x| x.parse().unwrap());

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    run_migrations(&db_url);

    let mng = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = Pool::builder(mng).build().unwrap_or_log();

    // should be removed once the normal non-http workers will be allowed on fly.io
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(dummy)) });
    let dummy_server = Server::bind(&addr).serve(make_service);

    start_bot(
        bot,
        Duration::seconds(media_timeout_sec),
        Duration::seconds(ignore_message_older_than_sec),
        Percentage::from(media_being_sent_chance_in_percent),
        Percentage::from_decimal(similarity_threshold_in_decimal),
        pool,
    )
    .await;

    std::mem::drop(dummy_server);
}
