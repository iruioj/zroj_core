use actix_web::{get, post, web::{self, get}, HttpResponse, Responder, Error};
use async_graphql::{Context, EmptySubscription, FieldResult, Object, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

pub struct Query;

#[Object]
impl Query {
    //owners query
    async fn test(&self, input: String) -> FieldResult <String> {
        Ok(format!("Query `{}` Success",input))
    }
}

pub struct Mutation;
#[Object]
impl Mutation {
    async fn test(&self, input: String) -> FieldResult <String> {
        Ok(format!("Modification `{}` Success",input))
    }
}
async fn index(schema: web::Data<ProjectSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
pub type ProjectSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn config(cfg: &mut web::ServiceConfig) {
    let schema_data = Schema::build(Query, Mutation, EmptySubscription)
        //.data(db)
        .finish();
    cfg .app_data(schema_data.clone())
        .service(web::resource("/").to(index));
}



