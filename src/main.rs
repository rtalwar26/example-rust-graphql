use juniper::http::graphiql::graphiql_source;
use std::sync::Arc;
use tokio_postgres::{ NoTls};
use warp::Filter;
mod models;
mod db_util;
mod gql;


type Context = gql::Context;






#[tokio::main]
async fn main() {
    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres password=postgres", NoTls)
        .await
        .unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

   db_util::init_tables(&client).await;
    let schema = Arc::new(gql::Schema::new(gql::QueryRoot, gql::MutationRoot));
    // Create a warp filter for the schema
    let schema = warp::any().map(move || Arc::clone(&schema));

    let ctx = Arc::new(Context { client });
    // Create a warp filter for the context
    let ctx = warp::any().map(move || Arc::clone(&ctx));

    let graphql_route = warp::post()
        .and(warp::path!("graphql"))
        .and(schema.clone())
        .and(ctx.clone())
        .and(warp::body::json())
        .and_then(gql::graphql);

    let graphiql_route = warp::get()
        .and(warp::path!("graphiql"))
        .map(|| warp::reply::html(graphiql_source("graphql")));

    let routes = graphql_route.or(graphiql_route);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
