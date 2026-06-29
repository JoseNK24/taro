use std::sync::{Arc, Mutex};

use crate::catalog::Catalog;
use crate::community_install::CommunityInstallManager;
use crate::db::Database;
use crate::plugin_catalog::PluginCatalog;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub catalog: Catalog,
    pub plugin_catalog: PluginCatalog,
    pub community_install: CommunityInstallManager,
}
