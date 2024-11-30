use std::{fs::File, io::Write};

use axum::{
    debug_handler,
    extract::{DefaultBodyLimit, Multipart, Query},
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

use crate::{
    db::data as Data,
    logging::db_log,
    utils::{
        middle::Tag,
        queries::{UCPTypeExtraQuery, UCPTypeQuery},
        sql::get_db_conn,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Items {
    pub id: i32,
    pub owner: String,
    pub kep: String,
    pub status: String,
    pub reason: Option<String>,
    pub am: i8,
    pub admin: Option<String>,
    pub date: chrono::DateTime<Utc>,
}

pub fn routes() -> Router {
    Router::new()
        .route("/get", get(ucp_items_get))
        .route("/post", post(ucp_items_post))
        .layer(DefaultBodyLimit::max(64000000))
}

#[debug_handler]
pub async fn ucp_items_get(ext: Extension<Tag>, cucc: Query<UCPTypeQuery>) -> Json<Vec<Items>> {
    let db = get_db_conn().await;
    let getitem = Data::Entity::find()
        .filter(Data::Column::Owner.eq(&ext.name))
        .filter(Data::Column::Type.eq(cucc.tipus.clone()))
        .order_by(Data::Column::Date, Order::Desc)
        .all(&db)
        .await
        .expect("Leintések lekérése sikertelen az adatbázisból");
    let another: Vec<Items> = getitem
        .iter()
        .map(|strucc| -> Items {
            Items {
                am: strucc.am.clone(),
                owner: strucc.owner.clone(),
                kep: strucc.kep.clone(),
                reason: strucc.reason.clone(),
                status: strucc.status.clone(),
                date: strucc.date.clone(),
                id: strucc.id.clone(),
                admin: strucc.admin.clone(),
            }
        })
        .collect();
    Json(another)
}

#[debug_handler]
pub async fn ucp_items_post(
    ext: Extension<Tag>,
    cucc: Query<UCPTypeExtraQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut file_ids: Vec<i32> = Vec::new();
    let mut files_for_leintes: Vec<String> = Vec::new();
    let dates = cucc.dates.clone();
    let ditas: Vec<&str> = dates.split(",").collect();
    let mut i = 0;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();
        if field_name == "files" {
            let file_name = field.file_name().unwrap().to_string();
            let data = field.bytes().await;
            if data.is_ok() {
                let db = get_db_conn().await;
                let mut file =
                    File::create(format!("./public/tmp/{}-{}", ext.name, file_name)).unwrap();
                file.write(&data.unwrap()).unwrap();
                if cucc.tipus.clone() == String::from("leintés") {
                    if files_for_leintes.len().eq(&1) {
                        let iten = Data::ActiveModel {
                            am: Set(if ext.am.clone() { 1 } else { 0 }),
                            date: Set(
                                DateTime::from_timestamp_millis(ditas[i].parse().unwrap()).unwrap()
                            ),
                            owner: Set(ext.name.clone()),
                            r#type: Set(String::from(cucc.tipus.clone())),
                            kep: Set(format!(
                                "['{}','tmp/{}-{}']",
                                files_for_leintes[0], ext.name, file_name
                            )),
                            ..Default::default()
                        };
                        let newitem = Data::Entity::insert(iten)
                            .exec(&db)
                            .await
                            .expect("Adatbázisba mentés sikertelen");
                        db_log(
                            ext.name.clone(),
                            Some(newitem.last_insert_id),
                            Some(cucc.tipus.clone()),
                            "CREATE",
                            None,
                        )
                        .await;
                        file_ids.push(newitem.last_insert_id);
                        files_for_leintes.clear();
                    } else {
                        files_for_leintes.push(format!("tmp/{}-{}", ext.name, file_name))
                    }
                } else {
                    let iten = Data::ActiveModel {
                        am: Set(if ext.am.clone() { 1 } else { 0 }),
                        date: Set(
                            DateTime::from_timestamp_millis(ditas[i].parse().unwrap()).unwrap()
                        ),
                        owner: Set(ext.name.clone()),
                        r#type: Set(String::from(cucc.tipus.clone())),
                        kep: Set(format!("tmp/{}-{}", ext.name, file_name)),
                        ..Default::default()
                    };
                    let newitem = Data::Entity::insert(iten)
                        .exec(&db)
                        .await
                        .expect("Adatbázisba mentés sikertelen");
                    db_log(
                        ext.name.clone(),
                        Some(newitem.last_insert_id),
                        Some(cucc.tipus.clone()),
                        "CREATE",
                        None,
                    )
                    .await;
                    file_ids.push(newitem.last_insert_id)
                }
                i += 1
            } else {
                return Err((StatusCode::NOT_ACCEPTABLE, "toobig".to_string()));
            }
        } else {
            let data = field.text().await.unwrap();
            println!("field: {}   value: {}", field_name, data)
        }
    }
    Ok(Json(file_ids))
}
