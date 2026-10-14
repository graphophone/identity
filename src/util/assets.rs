use std::sync::Arc;

use crate::asset_storage::AssetStorage;

pub fn remove_asset(
    asset_storage: Arc<AssetStorage>,
    old_image_id: Option<String>,
) {
    if let Some(id) = old_image_id {
        tokio::spawn(async move {
            let id = id;
            let _ = asset_storage
                .remove_asset(&id)
                .await;
        });
    }
}