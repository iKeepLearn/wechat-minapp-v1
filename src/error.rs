//! 微信小程序错误处理模块
//!
//! 该模块定义了与微信小程序 API 交互过程中可能遇到的所有错误类型，
//! 包括微信官方错误码映射和第三方库错误转换。
//!
//! # 错误类型
//!
//! 模块包含两种主要的错误类型：
//!
//! - [`Error`][]: 主要的错误枚举，包含所有可能的错误情况
//! - [`ErrorCode`]: 微信官方错误码的 Rust 枚举表示
//!
//!
//! // 处理网络错误
//! async fn make_api_request() -> Result<(), Error> {
//!     let client = reqwest::Client::new();
//!     let response = client.get("https://api.weixin.qq.com/some/endpoint")
//!         .send()
//!         .await?; // 自动转换为 Error::Reqwest
//!     Ok(())
//! }
//! ```
//!
//! # 错误转换
//!
//! 模块自动实现了从常见第三方库错误到 [`Error`] 的转换：
//!
//! - `reqwest::Error` → `Error::Reqwest`
//! - `serde_json::Error` → `Error::SerdeJson`
//! - `base64::DecodeError` → `Error::Base64Decode`
//! - `aes::cipher::InvalidLength` → `Error::AesInvalidLength`
//!
//! 这使得错误处理更加方便，可以使用 `?` 操作符自动转换。

use serde_repr::Deserialize_repr;

use aes::cipher::InvalidLength as AesInvalidLength;
use aes::cipher::block_padding::UnpadError;
use base64::DecodeError as Base64DecodeError;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use strum::Display;

/// 微信小程序 SDK 错误枚举
///
/// 包含了所有可能遇到的错误类型，包括微信 API 错误、网络错误、加解密错误等。
///
/// # 错误分类
///
/// ## 微信 API 错误
///
/// 这些错误对应微信官方文档中的错误码：
///
/// - `InvalidCredential`: 凭证无效
/// - `InvalidCode`: 登录 code 无效
/// - `RateLimitExceeded`: API 调用频率限制
/// - 等等...
///
/// ## 第三方库错误
///
/// 自动转换的第三方库错误：
///
/// - `Reqwest`: HTTP 请求错误
/// - `SerdeJson`: JSON 序列化/反序列化错误
/// - `Base64Decode`: Base64 解码错误
/// - `AesInvalidLength`: AES 加解密长度错误
///
/// ## 系统错误
///
/// - `System`: 微信系统繁忙
/// - `InternalServer`: 内部服务器错误
///
///
/// # 序列化
///
/// 此枚举使用 `thiserror` 派生宏，提供了良好的错误消息格式。
/// 每个变体都包含描述性的错误信息。
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 微信系统繁忙，请稍候再试
    #[error("system error: {0}")]
    System(String),

    /// 获取 access_token 时 AppSecret 错误，或者 access_token 无效
    #[error("invalid credential: {0}")]
    InvalidCredential(String),

    /// 不合法的凭证类型
    #[error("invalid grant type: {0}")]
    InvalidGrantType(String),

    /// 不合法的 AppID，请检查 AppID 的正确性
    #[error("invalid app id: {0}")]
    InvalidAppId(String),

    /// 登录 code 无效或已过期
    #[error("invalid code: {0}")]
    InvalidCode(String),

    /// 请求参数错误
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// 无效的 appsecret，请检查 appsecret 的正确性
    #[error("invalid secret: {0}")]
    InvalidSecret(String),

    /// IP 地址不在白名单中
    #[error("forbidden ip: {0}")]
    ForbiddenIp(String),

    /// 高风险等级用户，小程序登录被拦截
    #[error("code blocked: {0}")]
    CodeBlocked(String),

    /// AppSecret 已被冻结，请登录小程序平台解冻
    #[error("secret frozen: {0}")]
    SecretFrozen(String),

    /// 缺少 access_token 参数
    #[error("missing access token: {0}")]
    MissingAccessToken(String),

    /// 缺少 appid 参数
    #[error("missing app id: {0}")]
    MissingAppId(String),

    /// 缺少 secret 参数
    #[error("missing secret: {0}")]
    MissingSecret(String),

    /// 缺少 code 参数
    #[error("missing code: {0}")]
    MissingCode(String),

    /// 需要 POST 请求
    #[error("required post method: {0}")]
    RequiredPostMethod(String),

    /// 调用超过天级别频率限制
    #[error("daily request limit exceeded: {0}")]
    DailyRequestLimitExceeded(String),

    /// API 调用太频繁，请稍候再试
    #[error("rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// 禁止使用 token 接口
    #[error("forbidden token: {0}")]
    ForbiddenToken(String),

    /// 账号已冻结
    #[error("account frozen: {0}")]
    AccountFrozen(String),

    /// 第三方平台 API 需要使用第三方平台专用 token
    #[error("third party token: {0}")]
    ThirdPartyToken(String),

    /// session_key 不存在或已过期
    #[error("session key not existed or expired: {0}")]
    SessionKeyNotExistedOrExpired(String),

    /// 无效的签名方法
    #[error("invalid signature method: {0}")]
    InvalidSignatureMethod(String),

    /// 无效的签名
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// 此次调用需要管理员确认，请耐心等候
    #[error("confirm required: {0}")]
    ConfirmRequired(String),

    /// 该IP调用请求已被公众号管理员拒绝，请24小时后再试
    #[error("request denied one day: {0}")]
    RequestDeniedOneDay(String),

    /// 该IP调用请求已被公众号管理员拒绝，请1小时后再试
    #[error("request denied one hour: {0}")]
    RequestDeniedOneHour(String),

    /// AES 解密时数据填充错误
    #[error("unpad error: {0}")]
    Unpad(UnpadError),

    /// AES 加解密长度错误
    #[error("aes invalid length: {0}")]
    AesInvalidLength(#[from] AesInvalidLength),

    /// Base64 解码错误
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] Base64DecodeError),

    /// HTTP 请求错误
    #[error("reqwest: {0}")]
    Reqwest(#[from] ReqwestError),

    /// JSON 序列化/反序列化错误
    #[error("json error: {0}")]
    SerdeJson(#[from] SerdeJsonError),

    /// 内部服务器错误
    #[error("internal error: {0}")]
    InternalServer(String),
}

impl From<UnpadError> for Error {
    fn from(error: UnpadError) -> Self {
        Error::Unpad(error)
    }
}

/// 微信官方错误码枚举
///
/// 对应微信小程序 API 返回的错误码，每个错误码都有对应的中文描述。
///
///
/// # 错误码说明
///
/// 完整的错误码列表请参考：
/// [微信官方文档 - 全局返回码说明](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/#%E5%85%A8%E5%B1%80%E8%BF%94%E5%9B%9E%E7%A0%81%E8%AF%B4%E6%98%8E)
#[derive(Debug, Deserialize_repr, Display)]
#[repr(i32)]
pub enum ErrorCode {
    #[strum(serialize = "系统繁忙，此时请开发者稍候再试")]
    System = -1,
    #[strum(
        serialize = "获取 access_token 时 AppSecret 错误，或者 access_token 无效。请开发者认真比对 AppSecret 的正确性，或查看是否正在为恰当的公众号调用接口"
    )]
    InvalidCredential = 40001,
    #[strum(serialize = "不合法的凭证类型")]
    InvalidGrantType = 40002,
    #[strum(serialize = "不合法的 AppID ，请开发者检查 AppID 的正确性，避免异常字符，注意大小写")]
    InvalidAppId = 40013,
    #[strum(serialize = "code 无效")]
    InvalidCode = 40029,
    #[strum(serialize = "参数错误")]
    InvalidParameter = 40097,
    #[strum(serialize = "无效的appsecret，请检查appsecret的正确性")]
    InvalidSecret = 40125,
    #[strum(serialize = "将ip添加到ip白名单列表即可")]
    ForbiddenIp = 40164,
    #[strum(serialize = "高风险等级用户，小程序登录拦截 。风险等级详见用户安全解方案")]
    CodeBlocked = 40226,
    #[strum(serialize = "AppSecret已被冻结，请登录小程序平台解冻后再次调用")]
    SecretFrozen = 40243,
    #[strum(serialize = "缺少 access token 参数")]
    MissingAccessToken = 41001,
    #[strum(serialize = "缺少 appid 参数")]
    MissingAppId = 41002,
    #[strum(serialize = "缺少 secret 参数")]
    MissingSecret = 41004,
    MissingCode = 41008,
    #[strum(serialize = "需要 POST 请求")]
    RequiredPostMethod = 43002,
    #[strum(serialize = "调用超过天级别频率限制。可调用clear_quota接口恢复调用额度。")]
    DailyRequestLimitExceeded = 45009,
    #[strum(serialize = "API 调用太频繁，请稍候再试")]
    RateLimitExceeded = 45011,
    #[strum(serialize = "禁止使用 token 接口")]
    ForbiddenToken = 50004,
    #[strum(serialize = "账号已冻结")]
    AccountFrozen = 50007,
    #[strum(serialize = "第三方平台 API 需要使用第三方平台专用 token")]
    ThirdPartyToken = 61024,
    #[strum(serialize = "session_key is not existed or expired")]
    SessionKeyNotExistedOrExpired = 87007,
    #[strum(serialize = "invalid sig_method")]
    InvalidSignatureMethod = 87008,
    #[strum(serialize = "无效的签名")]
    InvalidSignature = 87009,
    #[strum(serialize = "此次调用需要管理员确认，请耐心等候")]
    ConfirmRequired = 89503,
    #[strum(
        serialize = "该IP调用求请求已被公众号管理员拒绝，请24小时后再试，建议调用前与管理员沟通确认"
    )]
    RequestDeniedOneDay = 89506,
    #[strum(
        serialize = "该IP调用求请求已被公众号管理员拒绝，请1小时后再试，建议调用前与管理员沟通确认"
    )]
    RequestDeniedOneHour = 89507,
}

impl From<(ErrorCode, String)> for Error {
    /// 从微信错误码和消息创建 Error
    ///
    /// # 参数
    ///
    /// - `(code, message)`: 微信错误码和对应的错误消息
    ///
    /// # 返回
    ///
    /// 对应的 `Error` 枚举变体
    fn from((code, message): (ErrorCode, String)) -> Self {
        use ErrorCode::*;

        match code {
            System => Error::System(message),
            InvalidCredential => Error::InvalidCredential(message),
            InvalidGrantType => Error::InvalidGrantType(message),
            InvalidAppId => Error::InvalidAppId(message),
            InvalidCode => Error::InvalidCode(message),
            InvalidParameter => Error::InvalidParameter(message),
            InvalidSecret => Error::InvalidSecret(message),
            ForbiddenIp => Error::ForbiddenIp(message),
            CodeBlocked => Error::CodeBlocked(message),
            SecretFrozen => Error::SecretFrozen(message),
            MissingAccessToken => Error::MissingAccessToken(message),
            MissingAppId => Error::MissingAppId(message),
            MissingSecret => Error::MissingSecret(message),
            MissingCode => Error::MissingCode(message),
            RequiredPostMethod => Error::RequiredPostMethod(message),
            DailyRequestLimitExceeded => Error::DailyRequestLimitExceeded(message),
            RateLimitExceeded => Error::RateLimitExceeded(message),
            ForbiddenToken => Error::ForbiddenToken(message),
            AccountFrozen => Error::AccountFrozen(message),
            ThirdPartyToken => Error::ThirdPartyToken(message),
            SessionKeyNotExistedOrExpired => Error::SessionKeyNotExistedOrExpired(message),
            InvalidSignatureMethod => Error::InvalidSignatureMethod(message),
            InvalidSignature => Error::InvalidSignature(message),
            ConfirmRequired => Error::ConfirmRequired(message),
            RequestDeniedOneDay => Error::RequestDeniedOneDay(message),
            RequestDeniedOneHour => Error::RequestDeniedOneHour(message),
        }
    }
}
