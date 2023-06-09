pub mod model;
pub mod sys;

use crate::graphql::model::{GraphqlMutation, GraphqlQuery};
use crate::infra::error::Result;
use crate::infra::resolver::*;
use async_graphql::{extensions, EmptySubscription, Schema};
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
use std::env;
use std::path::Path;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    listen_addr: String,
    pgsql: String,
    redis: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:3000".to_string(),
            pgsql: "postgres://postgres:postgres@localhost/shop".to_string(),
            redis: "redis://redis/".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Resolver {
    pub listen_addr: Register<String>,
    pub pgsql: Register<&'static Pool<ConnectionManager<PgConnection>>>,
    pub redis: Register<&'static Pool<redis::Client>>,
}

impl BaseResolver for Resolver {
    const TARGET: Target = Target::GRAPHQL;
}

impl NamedResolver for Resolver {
    const SID: &'static str = "sys-graphql";
}

static CONFIG: OnceCell<Config> = OnceCell::new();

impl Resolver {
    pub fn new(conf: impl AsRef<Path>) -> Self {
        CONFIG.get_or_init(|| {
            let settings = config::Config::builder()
                .add_source(File::from(conf.as_ref()))
                .add_source(Environment::with_prefix("APP"))
                .build()
                .unwrap();
            settings.try_deserialize().unwrap()
        });
        println!(
            "Service `{}` is starting...\nDeployment ID: {}\nConfiguration:\n{}",
            Self::SID,
            env::var("APP_DEPLOYMENT_ID").unwrap_or("undefined".to_string()),
            serde_json::to_string_pretty(CONFIG.get().unwrap()).unwrap()
        );
        Self {
            listen_addr: Register::once(|| CONFIG.get().unwrap().listen_addr.to_string()),
            pgsql: Register::once_ref(|| {
                let dsn = CONFIG.get().unwrap().pgsql.as_str();
                Pool::new(ConnectionManager::new(dsn)).unwrap()
            }),
            redis: Register::once_ref(|| {
                let dsn = CONFIG.get().unwrap().redis.as_str();
                Pool::new(redis::Client::open(dsn).unwrap()).unwrap()
            }),
        }
    }

    pub fn pg_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        Ok(self.resolve(&self.pgsql).get()?)
    }

    pub fn redis_conn(&self) -> Result<PooledConnection<redis::Client>> {
        Ok(self.resolve(&self.redis).get()?)
    }

    fn schema(&self) -> Schema<GraphqlQuery, GraphqlMutation, EmptySubscription> {
        Schema::build(GraphqlQuery, GraphqlMutation, EmptySubscription)
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
            .run(self.make_service().with(Cors::new()))
            .await
            .unwrap();
    }
}
