use axum::{debug_handler, extract::Request, Json};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::utils::functions::get_fridays;
use crate::utils::{api::get_api_envs, middle::Tag, sql::get_db_conn};

use crate::db::data as Data;

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
    let envs = get_api_envs();
    let calls = client
        .get(format!("{}/api/log/status/current", envs.erik))
        .send()
        .await;
    let fridays = get_fridays();
    let dbreturn = Data::Entity::find()
        .filter(Data::Column::Owner.eq(&exts.unwrap().name))
        .filter(Data::Column::Type.ne("számla"))
        .filter(Data::Column::Status.eq("elfogadva"))
        .filter(Data::Column::Date.gt(fridays.last))
        .filter(Data::Column::Date.lt(fridays.next))
        .all(&db)
        .await
        .expect("Leintések lekérése sikertelen az adatbázisból");
    let mut leintes = vec![];
    let mut de_potlek = vec![];
    let mut du_potlek = vec![];
    for model in dbreturn.iter() {
        if model.r#type == "pótlék" {
            if model.extra == "délelőtti".to_string().into() {
                de_potlek.push(model)
            }
            if model.extra == "éjszakai".to_string().into() {
                du_potlek.push(model)
            }
        }
        if model.r#type == "leintés" {
            leintes.push(model)
        }
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
                leintes: leintes.len(),
                potlek: Potlek {
                    de: de_potlek.len(),
                    du: du_potlek.len(),
                },
            }));
        }
    };
    Ok(Json(Callz {
        app: None,
        leintes: leintes.len(),
        potlek: Potlek {
            de: de_potlek.len(),
            du: du_potlek.len(),
        },
    }))
}
