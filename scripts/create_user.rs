use bcrypt::{hash, DEFAULT_COST};
use dotenv::dotenv;
use sqlx::MySqlPool;



/// TODO: 部署后，添加系统超级管理员

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    let pool = MySqlPool::connect(&database_url).await?;

    // let username = "SuperAdmin";
    // let password = "3m4c3n9q8J!";
    // let name = "超级管理员";
    // let hashed_password = hash(password.as_bytes(), DEFAULT_COST)?;
    
    // sqlx::query(
    //     "INSERT INTO users (username, password_hash, name, is_super_admin) 
    //      VALUES (?, ?, ?, ?)"
    // )
    // .bind(username)
    // .bind(hashed_password)
    // .bind(name)
    // .bind(1)
    // .execute(&pool)
    // .await?;
    
    let username = "Malonglong";
    let password = "Blh@123456";
    let name = "马龙龙";
    let hashed_password = hash(password.as_bytes(), DEFAULT_COST)?;
    
    sqlx::query(
        "INSERT INTO users (username, password_hash, name, tenant_id) 
         VALUES (?, ?, ?, ?)"
    )
    .bind(username)
    .bind(hashed_password)
    .bind(name)
    .bind(1)
    .execute(&pool)
    .await?;
    
    println!("Created user:");
    println!("Username: {}", username);
    println!("Password: {}", password);
    
    Ok(())
}