//! Defines data structure for storage in Gotham State that provides access to the underlying r2d2
//! pool so a connection can be established if required by Middleware or Handlers.

use diesel::{Connection, r2d2::ConnectionManager};
use r2d2::{Error, Pool, PooledConnection};

use gotham::state::{FromState, State};

/// Convenience function for Middleware and Handlers to obtain a Diesel connection.
pub fn try_connection<T>(s: &State) -> Result<PooledConnection<ConnectionManager<T>>, Error>
where
    T: Connection + 'static,
{
    DieselState::borrow_from(s).conn()
}

/// Provides access to a Diesel connection within an r2d2 pool via Gotham State
#[derive(StateData)]
pub struct DieselState<T>
where
    T: Connection + 'static,
{
    pool: Pool<ConnectionManager<T>>,
}

impl<T> DieselState<T>
where
    T: Connection + 'static,
{
    pub(crate) fn new(pool: Pool<ConnectionManager<T>>) -> Self {
        DieselState { pool }
    }

    /// Provides access to a Diesel connection from our r2d2 backed connection pool.
    pub fn conn(&self) -> Result<PooledConnection<ConnectionManager<T>>, Error> {
        self.pool.get()
    }
}
