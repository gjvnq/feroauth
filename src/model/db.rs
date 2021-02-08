use crate::model::prelude::*;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::ConnectOptions;

pub static mut DB_POOL: Option<sqlx::Pool<sqlx::MySql>> = None;

#[derive(Debug)]
pub struct DbConn {
    pool: MySqlPool,
}

pub async fn get_pool(db_host: &str, db_user: &str, db_pass: &str, db_name: &str) -> MySqlPool {
    let mut conn_opts = MySqlConnectOptions::new()
        .host(&db_host)
        .username(&db_user)
        .password(&db_pass)
        .database(&db_name);

    conn_opts.log_statements(log::LevelFilter::Debug);
    let pool = MySqlPoolOptions::new()
        .connect_with(conn_opts)
        .await
        .unwrap();

    // Ensure it is actually in UTC
    let time_diff: Result<(chrono::NaiveTime,), _> =
        sqlx::query_as("SELECT TIMEDIFF(NOW(), UTC_TIMESTAMP);")
            .fetch_one(&pool)
            .await;
    match time_diff {
        Err(err) => {
            // err!("Failed to check database timezone: {}", err);
            panic!("Failed to check database timezone: {}", err);
        }
        Ok((dt,)) => {
            let formated = dt.format("%H:%M:%S");
            if format!("{}", formated) != "00:00:00" {
                // err!("Difference between database timezone and UTF is not zero: {} != 00:00:00", formated);
                panic!(
                    "Database is not in UTC as the difference ({}) is not zero",
                    formated
                );
            }
        }
    };

    pool
}

pub fn get_db_pool() -> &'static sqlx::Pool<sqlx::MySql> {
    unsafe { DB_POOL.as_ref().unwrap() }
}

pub async fn get_tx() -> Transaction<'static> {
    let tx = get_db_pool().begin().await;
    match tx {
        Ok(tx) => tx,
        Err(err) => {
            error!("Failed to get DB transaction: {:?}", err);
            panic!("Failed to get DB transaction: {:?}", err);
        }
    }
}
