use crate::rpc::Resolver;
use volo_gen::*;

pub struct Service(Resolver);

#[volo::async_trait]
impl product::v1::ProductService for Service {
    async fn ping(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
