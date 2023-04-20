//! MQSRS
//! Mutation,Query,Subscription Responsibility Separation

use async_trait::async_trait;
use futures::Stream;
use std::future::Future;

#[async_trait]
pub trait Mutation<I, O> {
    async fn execute(self, input: I) -> O;
}

#[async_trait]
pub trait Query<I, O> {
    async fn execute(&self, input: I) -> O;
}

#[async_trait]
pub trait Subscription<I, O> {
    type OutStream: Stream<Item = O>;
    async fn execute(&self, input: I) -> Self::OutStream;
}

#[async_trait]
impl<I, O, TMutation, TFuture> Mutation<I, O> for TMutation
where
    TMutation: (FnOnce(I) -> TFuture) + Send,
    TFuture: Future<Output = O> + Send,
    I: Send + 'static,
{
    async fn execute(self, input: I) -> O {
        self(input).await
    }
}

#[async_trait]
impl<I, O, TQuery, TFuture> Query<I, O> for TQuery
where
    TQuery: (Fn(I) -> TFuture) + Sync,
    TFuture: Future<Output = O> + Send,
    I: Send + 'static,
{
    async fn execute(&self, input: I) -> O {
        self(input).await
    }
}

#[async_trait]
impl<I, O, TSubscription, TFuture, TStream> Subscription<I, O> for TSubscription
where
    TSubscription: (Fn(I) -> TFuture) + Sync,
    TFuture: Future<Output = TStream> + Send,
    TStream: Stream<Item = O> + Send,
    I: Send + 'static,
{
    type OutStream = TStream;

    async fn execute(&self, input: I) -> Self::OutStream {
        self(input).await
    }
}
