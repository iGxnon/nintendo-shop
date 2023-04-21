use crate::rpc::Resolver;
use anyhow::Result;
use volo_gen::product::v1::{GetProductReq, GetProductRes};
use volo_gen::*;

pub struct Service(Resolver);

#[volo::async_trait]
impl product::v1::ProductService for Service {
    async fn ping(&self) -> Result<()> {
        Ok(())
    }

    async fn get_product(&self, req: GetProductReq) -> Result<GetProductRes> {
        todo!()
    }
}
