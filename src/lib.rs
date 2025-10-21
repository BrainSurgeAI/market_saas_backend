pub mod config;
mod middleware;

pub mod libs;
pub mod models;
pub mod repositories;
pub mod routers;
pub mod services;
pub mod types;
pub mod utils;

pub mod common;
pub mod dto;

#[macro_export]
macro_rules! map_db_err {
    ($msg:expr) => {
        |e| {
            error!(concat!($msg, ": {:#?}"), e);
            AppError::Database(e)
        }
    };
}
