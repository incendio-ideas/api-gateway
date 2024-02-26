use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptyMutation, EmptySubscription, Schema,
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
    async fn login(&self, ctx: &Context<'_>) -> String {
        let mut grpc_client: AuthClient<Channel> =
            ctx.data::<AuthClient<Channel>>().unwrap().clone();

        let request = tonic::Request::new(auth::LoginRequest::default());
        let response = grpc_client.login(request).await.unwrap().into_inner();

        response.token
    }

    async fn register(&self, ctx: &Context<'_>) -> String {
        let mut grpc_client: AuthClient<Channel> =
            ctx.data::<AuthClient<Channel>>().unwrap().clone();

        let request = tonic::Request::new(auth::RegisterRequest::default());
        let response = grpc_client.register(request).await.unwrap().into_inner();

        response.token
    }
}

type SchemaType = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[rocket::get("/playground")]
fn graphql_playground() -> rocket::response::content::RawHtml<String> {
    rocket::response::content::RawHtml(playground_source(GraphQLPlaygroundConfig::new(
        "/v0/graphql",
    )))
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn graphql(schema: &rocket::State<SchemaType>, request: GraphQLRequest) -> GraphQLResponse {
    request.execute(schema.inner()).await
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth_grpc_uri = std::env::var("AUTH_GRPC_URI").unwrap_or_else(|_| {
        "http://0.0.0.0:50051".to_string()
    });

    let grpc_client = AuthClient::connect(auth_grpc_uri).await?;
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(grpc_client)
        .finish();

    let _ = rocket::build()
        .manage(schema)
        .mount("/v0", rocket::routes![graphql, graphql_playground])
        .launch()
        .await;

    Ok(())
}
