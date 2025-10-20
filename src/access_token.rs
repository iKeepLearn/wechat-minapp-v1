use crate::{Result, constants, error::Error::InternalServer, response::Response};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use tracing::{debug, instrument};

#[derive(Clone)]
pub struct AccessToken {
    pub access_token: String,
    pub expired_at: DateTime<Utc>,
    pub force_refresh: Option<bool>,
}

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StableAccessToken")
            .field("access_token", &"********")
            .field("expired_at", &self.expired_at)
            .field("force_refresh", &self.force_refresh)
            .finish()
    }
}

#[derive(Deserialize)]
pub(crate) struct AccessTokenBuilder {
    pub access_token: String,
    #[serde(
        deserialize_with = "AccessTokenBuilder::deserialize_expired_at",
        rename = "expires_in"
    )]
    pub expired_at: DateTime<Utc>,
}

impl AccessTokenBuilder {
    fn deserialize_expired_at<'de, D>(
        deserializer: D,
    ) -> std::result::Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = Duration::seconds(i64::deserialize(deserializer)?);

        Ok(Utc::now() + seconds)
    }
}

impl std::fmt::Debug for AccessTokenBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessTokenBuilder")
            .field("access_token", &"********")
            .field("expired_at", &self.expired_at)
            .finish()
    }
}

/// 获取小程序全局唯一后台接口调用凭据（access_token）
/// https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/access-token/auth.getAccessToken.html
#[instrument(skip(client))]
pub(crate) async fn get_access_token(
    client: Client,
    appid: &str,
    secret: &str,
) -> Result<AccessTokenBuilder> {
    let mut map: HashMap<&str, &str> = HashMap::new();

    map.insert("grant_type", "client_credential");
    map.insert("appid", appid);
    map.insert("secret", secret);

    let response = client
        .get(constants::ACCESS_TOKEN_END_POINT)
        .query(&map)
        .send()
        .await?;

    debug!("response: {:#?}", response);

    if response.status().is_success() {
        let res = response.json::<Response<AccessTokenBuilder>>().await?;

        let builder = res.extract()?;

        debug!("access token builder: {:#?}", builder);

        Ok(builder)
    } else {
        Err(InternalServer(response.text().await?))
    }
}

/// 获取小程序全局唯一后台接口调用凭据（access_token）
/// https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/mp-access-token/getStableAccessToken.html
#[instrument(skip(client, force_refresh))]
pub(crate) async fn get_stable_access_token(
    client: Client,
    appid: &str,
    secret: &str,
    force_refresh: impl Into<Option<bool>>,
) -> Result<AccessTokenBuilder> {
    let mut map: HashMap<&str, String> = HashMap::new();

    map.insert("grant_type", "client_credential".into());
    map.insert("appid", appid.to_string());
    map.insert("secret", secret.to_string());

    if let Some(force_refresh) = force_refresh.into() {
        debug!("force_refresh: {}", force_refresh);

        map.insert("force_refresh", force_refresh.to_string());
    }

    let response = client
        .post(constants::STABLE_ACCESS_TOKEN_END_POINT)
        .json(&map)
        .send()
        .await?;

    debug!("response: {:#?}", response);

    if response.status().is_success() {
        let response = response.json::<Response<AccessTokenBuilder>>().await?;

        let builder = response.extract()?;

        debug!("stable access token builder: {:#?}", builder);

        Ok(builder)
    } else {
        Err(InternalServer(response.text().await?))
    }
}
