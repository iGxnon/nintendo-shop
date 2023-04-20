pub mod model;
pub mod sys;

use crate::graphql::model::{Mutation, Query};
use crate::infra::resolver::*;
use async_graphql::*;
use async_graphql_poem::GraphQL;
use config::{Environment, File};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use poem::listener::TcpListener;
use poem::{get, Route, Server};
use r2d2::PooledConnection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct Config {
    listen_addr: String,
    shutdown_timeout: u64,
    redis: String,
    pgsql: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:3000".to_string(),
            shutdown_timeout: 10,
            redis: "redis://127.0.0.1/".to_string(),
            pgsql: "".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Resolver {
    pub listen_addr: Register<String>,
    pub redis: Register<&'static PooledConnection<redis::Client>>,
    pub pgsql: Register<&'static PooledConnection<ConnectionManager<PgConnection>>>,
}

impl BaseResolver for Resolver {
    const TARGET: Target = Target::GRAPHQL;
}

impl NamedResolver for Resolver {
    const UUID: &'static str = "sys-graphql";
}

impl Resolver {
    pub fn new(conf: impl AsRef<Path>) -> Self {
        let settings = config::Config::builder()
            .add_source(File::from(conf.as_ref()))
            .add_source(
                Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()
            .unwrap();
        let config: Config = settings.try_deserialize().unwrap();
        // Self {
        //     listen_addr: (),
        //     redis: (),
        //     pgsql: (),
        // };
        todo!()
    }

    fn schema(&self) -> Schema<Query, EmptyMutation, EmptySubscription> {
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(self.clone())
            .extension(extensions::Analyzer)
            .extension(extensions::Tracing)
            // .extension(extensions::OpenTelemetry::new(todo!()))
            .finish()
    }

    fn make_service(&self) -> Route {
        Route::new().at(
            "/graphql",
            get(sys::graphiql).post(GraphQL::new(self.schema())),
        )
    }

    pub async fn serve(&self) {
        Server::new(TcpListener::bind(self.resolve(&self.listen_addr)))
            .run_with_graceful_shutdown(
                self.make_service(),
                async {
                    let _ = tokio::signal::ctrl_c().await;
                },
                Some(Duration::from_secs(10)),
            )
            .await
            .unwrap();
    }
}
