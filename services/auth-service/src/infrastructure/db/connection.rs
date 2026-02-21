//! Database connection pool management

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

/// Type alias for the connection pool
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Type alias for a pooled connection
pub type PooledDbConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Creates a new database connection pool
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
/// * `max_size` - Maximum number of connections in the pool
///
/// # Errors
/// Returns error if pool creation fails
pub fn create_connection_pool(database_url: &str, max_size: u32) -> Result<DbPool, r2d2::Error> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(max_size)
        .build(manager)
}
