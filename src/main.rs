use chrono::Duration;
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use std::env;
use std::{convert::Infallible, net::SocketAddr};
use teloxide::prelude::*;

use krusty::bot::start_bot;

async fn dummy(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("")))
}

fn connection() -> PgConnection {
    let uri = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    log::debug!("PG uri: {}", uri);
    PgConnection::establish(&uri).expect("Failed to obtain connection")
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations() {
    let mut connection = connection();
    connection.run_pending_migrations(MIGRATIONS).unwrap();
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    let bot = Bot::from_env();

    let media_timeout_sec =
        env::var("MEDIA_TIMEOUT_SEC").map_or_else(|_| 30, |x| x.parse().unwrap());
    let ignore_message_older_than_sec =
        env::var("IGNORE_MESSAGE_OLDER_THAN_SEC").map_or_else(|_| 60, |x| x.parse().unwrap());

    if media_timeout_sec > ignore_message_older_than_sec {
        panic!("Media timeout is greater than 'ignore message older than': {media_timeout_sec} > {ignore_message_older_than_sec}");
    }

    run_migrations();

    // should be removed once the normal non-http workers will be allowed on fly.io
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(dummy)) });
    let dummy_server = Server::bind(&addr).serve(make_service);

    start_bot(
        bot,
        Duration::seconds(media_timeout_sec),
        Duration::seconds(ignore_message_older_than_sec),
    )
    .await;

    std::mem::drop(dummy_server);
}
