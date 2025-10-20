use crate::{
    Result,
    access_token::{AccessToken, get_access_token, get_stable_access_token},
    constants,
    credential::{Credential, CredentialBuilder},
    error::Error::InternalServer,
    response::Response,
};
use chrono::{Duration, Utc};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Notify, RwLock};
use tracing::{debug, instrument};

///
/// 提供与微信小程序后端 API 交互的核心功能，包括用户登录、访问令牌管理等。
///
/// # 功能特性
///
/// - 用户登录凭证校验
/// - 访问令牌自动管理（支持普通令牌和稳定版令牌）
/// - 线程安全的令牌刷新机制
/// - 内置 HTTP 客户端
///
/// # 快速开始
///
/// ```no_run
/// use wechat_minapp::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // 初始化客户端
///     let app_id = "your_app_id";
///     let secret = "your_app_secret";
///     let client = Client::new(app_id, secret);
///
///     // 用户登录
///     let code = "user_login_code_from_frontend";
///     let credential = client.login(code).await?;
///     println!("用户OpenID: {}", credential.open_id());
///
///     // 获取访问令牌
///     let access_token = client.access_token().await?;
///     println!("访问令牌: {}", access_token);
///
///     Ok(())
/// }
/// ```
///
/// # 令牌管理
///
/// 客户端自动管理访问令牌的生命周期：
///
/// - 令牌过期前自动刷新
/// - 多线程环境下的安全并发访问
/// - 避免重复刷新（令牌锁机制）
/// - 支持强制刷新选项
///
/// # 线程安全
///
/// `Client` 实现了 `Send` 和 `Sync`，可以在多线程环境中安全使用。
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
    access_token: Arc<RwLock<AccessToken>>,
    refreshing: Arc<AtomicBool>,
    notify: Arc<Notify>,
    use_stable_token: bool,
}

impl Client {
    /// 创建新的微信小程序客户端
    ///
    /// # 参数
    ///
    /// - `app_id`: 小程序 AppID
    /// - `secret`: 小程序 AppSecret
    ///
    /// # 返回
    ///
    /// 新的 `Client` 实例
    ///
    /// # 示例
    ///
    /// ```
    /// use wechat_minapp::Client;
    ///
    /// let client = Client::new("your_appid", "your_app_secret_here");
    /// ```
    pub fn new(app_id: &str, secret: &str) -> Self {
        let client = reqwest::Client::new();

        Self {
            inner: Arc::new(ClientInner {
                app_id: app_id.into(),
                secret: secret.into(),
                client,
            }),
            access_token: Arc::new(RwLock::new(AccessToken {
                access_token: "".to_string(),
                expired_at: Utc::now(),
                force_refresh: None,
            })),
            refreshing: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
            use_stable_token: true,
        }
    }

    pub fn with_non_stable(app_id: &str, secret: &str) -> Self {
        let client = reqwest::Client::new();

        Self {
            inner: Arc::new(ClientInner {
                app_id: app_id.into(),
                secret: secret.into(),
                client,
            }),
            access_token: Arc::new(RwLock::new(AccessToken {
                access_token: "".to_string(),
                expired_at: Utc::now(),
                force_refresh: None,
            })),
            refreshing: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
            use_stable_token: false,
        }
    }

    pub(crate) fn request(&self) -> &reqwest::Client {
        &self.inner.client
    }

    /// 用户登录凭证校验
    ///
    /// 通过微信前端获取的临时登录凭证 code，换取用户的唯一标识 OpenID 和会话密钥。
    ///
    /// # 参数
    ///
    /// - `code`: 微信前端通过 `wx.login()` 获取的临时登录凭证
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(Credential)`，包含用户身份信息
    ///
    /// # 错误
    ///
    /// - 网络错误
    /// - 微信 API 返回错误
    /// - 响应解析错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use wechat_minapp::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new("app_id", "secret");
    ///     let code = "0816abc123def456";
    ///     let credential = client.login(code).await?;
    ///
    ///     println!("用户OpenID: {}", credential.open_id());
    ///     println!("会话密钥: {}", credential.session_key());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # API 文档
    ///
    /// [微信官方文档 - code2Session](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-login/code2Session.html)
    #[instrument(skip(self, code))]
    pub async fn login(&self, code: &str) -> Result<Credential> {
        debug!("code: {}", code);

        let mut map: HashMap<&str, &str> = HashMap::new();

        map.insert("appid", &self.inner.app_id);
        map.insert("secret", &self.inner.secret);
        map.insert("js_code", code);
        map.insert("grant_type", "authorization_code");

        let response = self
            .inner
            .client
            .get(constants::AUTHENTICATION_END_POINT)
            .query(&map)
            .send()
            .await?;

        debug!("authentication response: {:#?}", response);

        if response.status().is_success() {
            let response = response.json::<Response<CredentialBuilder>>().await?;

            let credential = response.extract()?.build();

            debug!("credential: {:#?}", credential);

            Ok(credential)
        } else {
            Err(InternalServer(response.text().await?))
        }
    }

    pub async fn token(&self) -> Result<String> {
        if self.use_stable_token {
            self.stable_access_token(None).await
        } else {
            self.access_token().await
        }
    }

    /// 获取访问令牌
    ///
    /// 获取用于调用微信小程序接口的访问令牌。如果当前令牌已过期或即将过期，会自动刷新。
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(String)`，包含有效的访问令牌
    ///
    /// # 错误
    ///
    /// - 网络错误
    /// - 微信 API 返回错误
    /// - 令牌刷新失败
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use wechat_minapp::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new("app_id", "secret");
    ///     let access_token = client.access_token().await?;
    ///     
    ///     println!("访问令牌: {}", access_token);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # 注意
    ///
    /// - 令牌有效期为 2 小时
    /// - 客户端会自动管理令牌刷新，无需手动处理
    /// - 多线程环境下安全
    pub async fn access_token(&self) -> Result<String> {
        // 第一次检查：快速路径
        {
            let guard = self.access_token.read().await;
            if !is_token_expired(&guard) {
                return Ok(guard.access_token.clone());
            }
        }

        // 使用CAS竞争刷新权
        if self
            .refreshing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            // 获得刷新权
            match self.refresh_access_token().await {
                Ok(token) => {
                    self.refreshing.store(false, Ordering::Release);
                    self.notify.notify_waiters();
                    Ok(token)
                }
                Err(e) => {
                    self.refreshing.store(false, Ordering::Release);
                    self.notify.notify_waiters();
                    Err(e)
                }
            }
        } else {
            // 等待其他线程刷新完成
            self.notify.notified().await;
            // 刷新完成后重新读取
            let guard = self.access_token.read().await;
            Ok(guard.access_token.clone())
        }
    }

    async fn refresh_access_token(&self) -> Result<String> {
        let mut guard = self.access_token.write().await;

        if !is_token_expired(&guard) {
            debug!("token already refreshed by another thread");
            return Ok(guard.access_token.clone());
        }

        debug!("performing network request to refresh token");

        let builder = get_access_token(
            self.inner.client.clone(),
            &self.inner.app_id,
            &self.inner.secret,
        )
        .await?;

        guard.access_token = builder.access_token.clone();
        guard.expired_at = builder.expired_at;

        debug!("fresh access token: {:#?}", guard);

        Ok(guard.access_token.clone())
    }

    /// 获取稳定版访问令牌
    ///
    /// 获取稳定版的访问令牌，相比普通令牌有更长的有效期和更好的稳定性。
    ///
    /// # 参数
    ///
    /// - `force_refresh`: 是否强制刷新令牌
    ///   - `Some(true)`: 强制从微信服务器获取最新令牌
    ///   - `Some(false)` 或 `None`: 仅在令牌过期时刷新
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(String)`，包含有效的稳定版访问令牌
    ///
    /// # 错误
    ///
    /// - 网络错误
    /// - 微信 API 返回错误
    /// - 令牌刷新失败
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use wechat_minapp::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new("app_id", "secret");
    ///     
    ///     // 仅在过期时刷新
    ///     let token1 = client.stable_access_token(None).await?;
    ///     
    ///     // 强制刷新
    ///     let token2 = client.stable_access_token(true).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # 注意
    ///
    /// - 稳定版令牌有效期更长，推荐在生产环境使用
    /// - 强制刷新会忽略本地缓存，直接请求新令牌
    pub async fn stable_access_token(
        &self,
        force_refresh: impl Into<Option<bool>> + Clone + Send,
    ) -> Result<String> {
        // 第一次检查：快速路径
        {
            let guard = self.access_token.read().await;
            if !is_token_expired(&guard) {
                return Ok(guard.access_token.clone());
            }
        }

        // 使用CAS竞争刷新权
        if self
            .refreshing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            // 获得刷新权
            match self.refresh_stable_access_token(force_refresh).await {
                Ok(token) => {
                    self.refreshing.store(false, Ordering::Release);
                    self.notify.notify_waiters();
                    Ok(token)
                }
                Err(e) => {
                    self.refreshing.store(false, Ordering::Release);
                    self.notify.notify_waiters();
                    Err(e)
                }
            }
        } else {
            // 等待其他线程刷新完成
            self.notify.notified().await;
            // 刷新完成后重新读取
            let guard = self.access_token.read().await;
            Ok(guard.access_token.clone())
        }
    }

    async fn refresh_stable_access_token(
        &self,
        force_refresh: impl Into<Option<bool>> + Clone + Send,
    ) -> Result<String> {
        // 1. Acquire the write lock. This blocks if another thread won CAS but is refreshing.
        let mut guard = self.access_token.write().await;

        // 2. Double-check expiration under the write lock (CRITICAL)
        // If another CAS-winner refreshed the token while we were waiting for the write lock,
        // we return the new token without performing a new network call.
        if !is_token_expired(&guard) {
            // Token is now fresh, return it
            debug!("token already refreshed by another thread");
            return Ok(guard.access_token.clone());
        }

        // 3. Perform the network request since the token is still stale
        debug!("performing network request to refresh token");

        let builder = get_stable_access_token(
            self.inner.client.clone(),
            &self.inner.app_id,
            &self.inner.secret,
            force_refresh,
        )
        .await?;

        // 4. Update the token
        guard.access_token = builder.access_token.clone();
        guard.expired_at = builder.expired_at;

        debug!("fresh access token: {:#?}", guard);

        // Return the newly fetched token (cloned here for consistency)
        Ok(guard.access_token.clone())
    }
}

#[derive(Debug)]
struct ClientInner {
    app_id: String,
    secret: String,
    client: reqwest::Client,
}

/// 检查令牌是否过期
///
/// 添加安全边界，在令牌过期前5分钟就认为需要刷新
fn is_token_expired(token: &AccessToken) -> bool {
    // 添加安全边界，提前刷新
    let now = Utc::now();
    token.expired_at.signed_duration_since(now) < Duration::minutes(5)
}
