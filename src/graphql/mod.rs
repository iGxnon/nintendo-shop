pub mod modelv1;
pub mod modelv2;
pub mod sys;

use crate::graphql::modelv1::{Mutation, Query};
use crate::infra::resolver::*;
use async_graphql::*;
use async_graphql_poem::GraphQL;
use config::{Environment, File};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use once_cell::sync::OnceCell;
use poem::listener::TcpListener;
use poem::middleware::Cors;
use poem::{get, EndpointExt, Route, Server};
use r2d2::{Pool, PooledConnection};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    listen_addr: String,
    pgsql: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:3000".to_string(),
            pgsql: "postgres://postgres:postgres@localhost/shop".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Resolver {
    pub listen_addr: Register<String>,
    pub pgsql: Register<&'static Pool<ConnectionManager<PgConnection>>>,
}

impl BaseResolver for Resolver {
    const TARGET: Target = Target::GRAPHQL;
}

impl NamedResolver for Resolver {
    const UID: &'static str = "sys-graphql";
}

static CONFIG: OnceCell<Config> = OnceCell::new();

impl Resolver {
    pub fn new(conf: impl AsRef<Path>) -> Self {
        CONFIG.get_or_init(|| {
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
            settings.try_deserialize().unwrap()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(CONFIG.get().unwrap()).unwrap()
        );
        Self {
            listen_addr: Register::once(|| CONFIG.get().unwrap().listen_addr.to_string()),
            pgsql: Register::once_ref(|| {
                let dsn = CONFIG.get().unwrap().pgsql.as_str();
                Pool::new(ConnectionManager::new(dsn)).unwrap()
            }),
        }
    }

    fn schema(&self) -> Schema<Query, Mutation, EmptySubscription> {
        Schema::build(Query, Mutation, EmptySubscription)
            .data(self.clone())
            .extension(extensions::Analyzer)
            .extension(extensions::Tracing)
            // .extension(extensions::OpenTelemetry::new(todo!()))
            .finish()
    }

    fn pg_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        Ok(self.resolve(&self.pgsql).get()?)
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
                self.make_service().with(Cors::new()),
                async {
                    let _ = tokio::signal::ctrl_c().await;
                },
                Some(Duration::from_secs(10)),
            )
            .await
            .unwrap();
    }
}
