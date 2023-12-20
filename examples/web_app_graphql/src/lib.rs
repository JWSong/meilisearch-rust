pub mod app_env_vars;
pub mod errors;
mod graphql_schema;
mod models;
mod schema;

use actix_web::{web, HttpResponse, Result};
use app_env_vars::AppEnvVars;
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use errors::ApplicationError;
use graphql_schema::Query;
use meilisearch_sdk::Client as SearchClient;

pub type ApplicationSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub struct GraphQlData {
    pub pool: Pool<AsyncPgConnection>,
    pub client: SearchClient,
}

pub async fn index_graphiql() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/").finish()))
}

pub async fn index(schema: web::Data<ApplicationSchema>, req: GraphQLRequest) -> GraphQLResponse {
    let req_inner = req.into_inner();

    schema.execute(req_inner).await.into()
}

pub fn build_schema(app_env_vars: &AppEnvVars) -> Result<ApplicationSchema, ApplicationError> {
    let client = SearchClient::new(
        &app_env_vars.meilisearch_host,
        Some(&app_env_vars.meilisearch_api_key),
    );

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&app_env_vars.database_url);
    let pool = Pool::builder(config).build()?;

    let schema_data = GraphQlData { pool, client };

    Ok(
        Schema::build(Query::default(), EmptyMutation, EmptySubscription)
            .data(schema_data)
            .finish(),
    )
}
