
#[derive(juniper::GraphQLObject)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub age: i32,
    pub email: String,
    pub address: String,
}
