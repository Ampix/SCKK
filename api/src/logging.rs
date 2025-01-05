use sea_orm::{EntityTrait, Set};

use crate::{db::logs, utils::sql::get_db_conn};

pub async fn db_log(
    owner: String,
    item_id: Option<i32>,
    item_type: Option<i8>,
    action: &str,
    message: Option<String>,
) {
    let db = get_db_conn().await;
    let amodel = logs::ActiveModel {
        owner: Set(owner),
        item_id: Set(item_id),
        item_type: Set(item_type),
        action: Set(String::from(action)),
        message: Set(message),
        ..Default::default()
    };
    logs::Entity::insert(amodel)
        .exec(&db)
        .await
        .expect("[ERROR] Log létrehozása sikertelen");
}
