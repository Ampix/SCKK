use axum::{debug_handler, extract::Request, Json};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::db::{hails, supplements};
use crate::utils::functions::get_fridays;
use crate::utils::types_statuses::get_statuses;
use crate::utils::{api::get_api_envs, middle::Tag, sql::get_db_conn};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DriverRecord {
    pub driver: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Callz {
    pub app: Option<u32>,
    pub leintes: usize,
    pub potlek: Potlek,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Potlek {
    de: usize,
    du: usize,
}

#[debug_handler]
pub async fn ucp_calls(mut request: Request) -> Result<Json<Callz>, (StatusCode, String)> {
    let exts: Option<&Tag> = request.extensions_mut().get();
    let client = reqwest::Client::new();
    let db = get_db_conn().await;
    let statuses = get_statuses();
    let envs = get_api_envs();
    let calls = client
        .get(format!("{}/api/log/status/current", envs.erik))
        .send()
        .await;
    let fridays = get_fridays();
    let dbreturn_supp = supplements::Entity::find()
        .filter(supplements::Column::Owner.eq(&exts.unwrap().name))
        .filter(supplements::Column::Status.eq(statuses.accepted.id))
        .filter(supplements::Column::Date.gt(fridays.last))
        .filter(supplements::Column::Date.lt(fridays.next))
        .all(&db)
        .await
        .expect("Leintések lekérése sikertelen az adatbázisból");
    let dbreturn_hails = hails::Entity::find()
        .filter(hails::Column::Owner.eq(&exts.unwrap().name))
        .filter(hails::Column::Status.eq(statuses.accepted.id))
        .filter(hails::Column::Date.gt(fridays.last))
        .filter(hails::Column::Date.lt(fridays.next))
        .all(&db)
        .await
        .expect("Leintések lekérése sikertelen az adatbázisból");
    let mut leintes = 0;
    let mut de_potlek = 0;
    let mut du_potlek = 0;
    for model in dbreturn_supp.iter() {
        if model.r#type.is_some() {
            if model.r#type.unwrap() == 1 {
                de_potlek += 1
            }
            if model.r#type.unwrap() == 2 {
                du_potlek += 1
            }
        }
    }
    for _ in dbreturn_hails.iter() {
        leintes += 1
    }
    if calls.is_ok() {
        let callsz = calls.unwrap().text().await.expect("Átalakítás sikertelen");
        let driver_records: Result<Vec<DriverRecord>, serde_json::Error> = from_str(&callsz);
        if driver_records.is_ok() {
            let drc = driver_records.unwrap();
            let rec: Option<&DriverRecord> = drc
                .iter()
                .find(|record| record.driver == exts.unwrap().name);
            return Ok(Json(Callz {
                app: if rec.is_some() {
                    Some(rec.unwrap().count)
                } else {
                    Some(0)
                },
                leintes: leintes,
                potlek: Potlek {
                    de: de_potlek,
                    du: du_potlek,
                },
            }));
        }
    };
    Ok(Json(Callz {
        app: None,
        leintes: leintes,
        potlek: Potlek {
            de: de_potlek,
            du: du_potlek,
        },
    }))
}
