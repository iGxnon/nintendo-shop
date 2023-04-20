#![feature(impl_trait_in_assoc_type)]

use shop_backend::rpc::product::Service;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // let addr: SocketAddr = "[::]:8081".parse().unwrap();
    // let addr = volo::net::Address::from(addr);
    // volo_gen::product::v1::ProductServiceServer::new(Service)
    //     .run(addr)
    //     .await
    //     .unwrap();
}
