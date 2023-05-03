/// The root of all configuration types
pub struct Config;

mod private {
    pub trait Bound: for<'de> serde::de::Deserialize<'de> {}
}

// A mark trait implemented by all dynamic Deserialize types
pub trait ConfigType: private::Bound {}

impl<T> private::Bound for T where T: for<'de> serde::de::Deserialize<'de> {}
impl<T> ConfigType for T where T: private::Bound {}

pub mod service {
    use crate::infra::config::ConfigType;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    //
    // pub trait ArchitectureConfig {
    //     type Monolith: ConfigType; // monolith architecture
    //     type Micro: ConfigType; // micro-services architecture
    //     type Hybrid: ConfigType; // hybrid architecture
    // }
    //
    // #[derive(Serialize, Deserialize, Debug)]
    // #[serde(tag = "type")]
    // pub enum MonolithConfig {
    //     Rest,
    //     Graphql,
    // }
    //
    // #[derive(Serialize, Deserialize, Debug)]
    // #[serde(tag = "type")]
    // pub enum MicroConfig {
    //     RestGrpc,
    //     RestThrift,
    //     GraphqlGrpc,
    //     GraphqlThrift,
    // }

    pub trait ServiceConfig {
        /// back-end protocol
        type Grpc: ConfigType; // for gRPC service
        type Thrift: ConfigType; // for thrift service
        /// front-end protocol
        type Rest: ConfigType; // for restful service
        type Graphql: ConfigType; // for graphql service
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RateLimitConfig {
        pub num: u64,
        pub per: Duration,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct LimitConfig {
        pub concurrency: usize,    // concurrency limit of this service
        pub rate: RateLimitConfig, // rate limit of this service
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CommonConfig {
        pub name: String, // service name, which is unique under a domain provided by NameResolver
        pub listen_addr: String, // address that this service listens to
        pub timeout: usize, // request timeout of this service
        pub limit: LimitConfig, // limit config
        pub retry: usize, // maximum number of retry when service responses a ServerError
        pub load_shed: bool, // whether to load shed a request when it is not available
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GrpcConfig {
        #[serde(flatten)]
        pub common: CommonConfig,
    }
}
