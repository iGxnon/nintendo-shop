use shop_backend::graphql::Resolver;

#[tokio::main]
async fn main() {
    let resolver = Resolver::new("config/sys-graphql.toml");
    resolver.serve().await
}
