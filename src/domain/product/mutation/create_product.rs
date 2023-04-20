use crate::infra::mqsrs::Mutation;
use crate::rpc::Resolver;

pub fn execute() {}

impl Resolver {
    pub fn create_product(&self) -> impl Mutation<(), ()> + '_ {
        move |req: ()| async { () }
    }
}
