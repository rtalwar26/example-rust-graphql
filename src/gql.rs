use juniper::http::GraphQLRequest;
use std::convert::Infallible;
use std::sync::Arc;
use juniper::RootNode;

use crate::models;
use tokio_postgres::{Client};
pub struct Context {
    pub client: Client,
}
impl juniper::Context for Context {}

pub struct QueryRoot;
pub struct MutationRoot;

type Customer = models::app_models::Customer;
#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    pub async  fn customer(ctx: &Context, id: String) -> juniper::FieldResult<Customer> {
        let uuid = uuid::Uuid::parse_str(&id)?;
        let row = ctx
            .client
            .query_one(
                "SELECT name, age, email, address FROM customers WHERE id = $1",
                &[&uuid],
            )
            .await?;
        let customer = Customer {
            id,
            name: row.try_get(0)?,
            age: row.try_get(1)?,
            email: row.try_get(2)?,
            address: row.try_get(3)?,
        };
        Ok(customer)
    }

    pub async fn customers(ctx: &Context) -> juniper::FieldResult<Vec<Customer>> {
        let rows = ctx
            .client
            .query("SELECT id, name, age, email, address FROM customers", &[])
            .await?;
        let mut customers = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get(0)?;
            let customer = Customer {
                id: id.to_string(),
                name: row.try_get(1)?,
                age: row.try_get(2)?,
                email: row.try_get(3)?,
                address: row.try_get(4)?,
            };
            customers.push(customer);
        }
        Ok(customers)
    }
}

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    pub async fn register_customer(
        ctx: &Context,
        name: String,
        age: i32,
        email: String,
        address: String,
    ) -> juniper::FieldResult<Customer> {
        let id = uuid::Uuid::new_v4();
        let email = email.to_lowercase();
        ctx.client
            .execute(
                "INSERT INTO customers (id, name, age, email, address) VALUES ($1, $2, $3, $4, $5)",
                &[&id, &name, &age, &email, &address],
            )
            .await?;
        Ok(Customer {
            id: id.to_string(),
            name,
            age,
            email,
            address,
        })
    }

    pub async fn update_customer_email(
        ctx: &Context,
        id: String,
        email: String,
    ) -> juniper::FieldResult<String> {
        let uuid = uuid::Uuid::parse_str(&id)?;
        let email = email.to_lowercase();
        let n = ctx
            .client
            .execute(
                "UPDATE customers SET email = $1 WHERE id = $2",
                &[&email, &uuid],
            )
            .await?;
        if n == 0 {
            return Err("User does not exist".into());
        }
        Ok(email)
    }

    async fn delete_customer(ctx: &Context, id: String) -> juniper::FieldResult<bool> {
        let uuid = uuid::Uuid::parse_str(&id)?;
        let n = ctx
            .client
            .execute("DELETE FROM customers WHERE id = $1", &[&uuid])
            .await?;
        if n == 0 {
            return Err("User does not exist".into());
        }
        Ok(true)
    }
}
pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub async fn graphql(
    schema: Arc<Schema>,
    ctx: Arc<Context>,
    req: GraphQLRequest,
) -> Result<impl warp::Reply, Infallible> {
    let res = req.execute_async(&schema, &ctx).await;
    let json = serde_json::to_string(&res).expect("Invalid JSON response");
    Ok(json)
}