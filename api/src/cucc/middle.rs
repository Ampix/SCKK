#![allow(non_snake_case)]

use axum::{extract::Request, http::HeaderMap, middleware::Next, response::IntoResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::get_discord_envs;

use super::api::get_api_envs;

#[derive(Debug, Deserialize, Clone)]
pub struct DiscordUser {
    pub id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetUserRes {
    pub Id: i64,
    pub PermissionGroup: Option<u32>,
    pub PlayerName: String,
    pub PositionId: i8,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub admin: bool,
    pub am: bool,
}

pub async fn basic_auth(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // do something with `request`...
    let auth = headers.get("cookie");
    let ds = get_discord_envs();
    let envs = get_api_envs();
    if auth.is_some() {
        let client = reqwest::Client::new();
        let dcuserget = client
            .get(format!("{}/users/@me", ds.api_endpoint))
            .header(
                "Authorization",
                format!("Bearer {}", auth.unwrap().to_str().unwrap()),
            )
            .send()
            .await
            .expect("Lekérés sikertelen");
        if dcuserget.status().as_u16() == 200 {
            let handled_user = dcuserget.text().await.expect("Átalakítás sikertelen");
            let parsed_user: DiscordUser =
                serde_json::from_str(&handled_user).expect("User object létrehozása sikertelen");
            let getuser: String = client
                .get(format!("{}/appauth/login/{}", envs.patrik, parsed_user.id))
                .send()
                .await
                .expect("Lekérés sikertelen")
                .text()
                .await
                .expect("Átalakítás sikertelen");
            let parsed_tag: GetUserRes =
                serde_json::from_str(&getuser).expect("User object létrehozása sikertelen");
            let tag = Tag {
                id: parsed_user.id,
                name: parsed_tag.PlayerName,
                admin: parsed_tag.PermissionGroup.is_some_and(|x| x == 1),
                am: false,
            };
            request.extensions_mut().insert(tag);
            return Ok(next.run(request).await);
        } else {
            return Err((StatusCode::NOT_ACCEPTABLE, "Sikertelen lekérés".to_string()));
        }
    } else {
        return Err((StatusCode::NOT_FOUND, "Nincs kuki".to_string()));
    };
}

pub async fn admin_auth(
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let exts: Option<&Tag> = req.extensions_mut().get();
    let uwrp = exts.expect("Tag lekérése sikertelen, basic_auth megtörtént?");
    if uwrp.admin == true {
        return Ok(next.run(req).await);
    } else {
        return Err((StatusCode::UNAUTHORIZED, "Nem vagy admin".to_string()));
    }
}
