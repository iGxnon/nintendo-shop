use async_graphql::http::GraphiQLSource;
use poem::web::Html;
use poem::*;

#[handler]
pub async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}
