use tokio_postgres::{Client};
pub async fn init_tables(client:&Client){
 
client.execute(
            "CREATE TABLE IF NOT EXISTS customers(
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            age INT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            address TEXT NOT NULL
        )",
            &[],
        )
        .await
        .expect("Could not create table");

}
