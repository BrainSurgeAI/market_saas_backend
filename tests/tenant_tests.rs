use sqlx::{MySqlPool, Row};

#[sqlx::test]
async fn basic_test(pool: MySqlPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    let foo = sqlx::query("SELECT * FROM tenants limit 1")
        .fetch_one(&mut *conn)
        .await?;

    assert_eq!(foo.get::<String, _>("name"), "融链-军采服务中心");

    Ok(())
}
