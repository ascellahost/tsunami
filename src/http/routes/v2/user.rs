use validator::Validate;

use crate::{
    http::models::{stats::DisplayEmbed, user::ModifyUser},
    prelude::*,
};

#[derive(Serialize, Deserialize, Clone, Apiv2Schema, TS)]
#[ts(export)]
pub struct UserResponse {
    pub discord_id: String,
    pub domain: String,
    pub id: i32,
    pub key: String,
    pub name: String,
    pub autodelete: Option<i32>,
    pub deleteall: Option<i32>,
    pub upload_key: Option<String>,
    pub url_style: i32,
    pub invite_code: Option<String>,
    pub invited_by: i32,
    pub lang: String,
    pub flags: i32,
    #[serde(default)]
    pub embed: Option<DisplayEmbed>,
    #[serde(default)]
    pub uploads: i64,
}

/// verify a user
///
/// Used to check if a user is verified
#[api_v2_operation(
  tags(Dashboard)
  produces = "application/json"
)]
#[get("/user")]
pub async fn get(data: AccessToken) -> Result<OkResponse<UserResponse>, Error> {
    let sdata = data.inner();
    let embed = get_embed::exec(sdata.id).await.ok().map(DisplayEmbed::from);
    let response = UserResponse {
        discord_id: sdata.discord_id,
        domain: sdata.domain,
        id: sdata.id,
        key: sdata.key,
        name: sdata.name,
        autodelete: sdata.autodelete,
        deleteall: sdata.deleteall,
        upload_key: sdata.upload_key,
        url_style: sdata.url_style,
        invite_code: sdata.invite_code,
        invited_by: sdata.invited_by,
        lang: sdata.lang,
        flags: sdata.flags,
        embed,
        uploads: get_user_image_count::exec(sdata.id).await.unwrap_or_default(),
    };
    Ok(OkResponse(response))
}

#[api_v2_operation(
  tags(Dashboard)
  produces = "application/json"
)]
#[post("/user")]
pub async fn post(data: AccessToken, modify: web::Json<ModifyUser>) -> Result<OkResponse<Users>, Error> {
    modify.validate()?;

    if let Some(name) = &modify.name {
        set_user_name::exec(data.id(), name.clone()).await?;
    }
    if let Some(key) = &modify.key {
        set_user_key::exec(data.id(), key.clone()).await?;
    }
    if let Some(domain) = &modify.domain {
        set_domain::exec(data.id(), domain.clone()).await?;
    }
    if let Some(autodelete) = &modify.autodelete {
        set_autodelete::exec(data.id(), *autodelete).await?;
    }
    if let Some(deleteall) = &modify.deleteall {
        set_deleteall::exec(data.id(), *deleteall).await?;
    }
    if let Some(upload_key) = &modify.upload_key {
        set_upload_key::exec(data.id(), upload_key).await?;
    }
    if let Some(lang) = &modify.lang {
        set_language::exec(data.id(), lang.clone()).await?;
    }
    if let Some(embed) = &modify.embed {
        // not gonna deal with this
        let embed = embed.clone();
        set_embed::exec(
            data.id(),
            embed.description,
            embed.title,
            embed.url,
            embed.color,
            embed.author,
        )
        .await?;
    }

    Ok(OkResponse(data.inner()))
}
