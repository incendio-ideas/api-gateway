use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Schema,
};
use async_graphql_rocket::{GraphQLRequest, GraphQLResponse};
use auth::auth_client::AuthClient;

use async_graphql::Object;
use tonic::transport::Channel;

pub mod auth {
    tonic::include_proto!("auth");
}

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello GraphQL".to_owned()
    }
}

type SchemaType = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[rocket::get("/graphiql")]
fn graphql_playground() -> rocket::response::content::RawHtml<String> {
    rocket::response::content::RawHtml(playground_source(GraphQLPlaygroundConfig::new(
        "/v0/graphql",
    )))
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn graphql(
    schema: &rocket::State<SchemaType>,
    _grpc_client: &rocket::State<AuthClient<Channel>>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request.execute(schema.inner()).await
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_client = AuthClient::connect("http://auth:50051").await?;
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let _ = rocket::build()
        .manage(schema)
        .manage(grpc_client)
        .mount("/", rocket::routes![graphql_playground])
        .mount("/v0", rocket::routes![graphql])
        .launch()
        .await;

    Ok(())
}
