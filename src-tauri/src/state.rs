use std::sync::Mutex;

use crate::catalog::Catalog;
use crate::db::Database;

pub struct AppState {
    pub db: Mutex<Database>,
    pub catalog: Catalog,
}
