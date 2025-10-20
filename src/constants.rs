//! 微信小程序 API 端点常量模块
//!
//! 该模块定义了微信小程序所有官方 API 的端点 URL 常量。
//! 这些常量用于构建完整的 API 请求地址，确保 URL 的正确性和一致性。
//!
//! # API 分类
//!
//! ## 访问令牌管理
//!
//! - [`STABLE_ACCESS_TOKEN_END_POINT`] - 获取稳定版访问令牌
//! - [`ACCESS_TOKEN_END_POINT`] - 获取普通访问令牌
//!
//! ## 用户会话管理
//!
//! - [`CHECK_SESSION_KEY_END_POINT`] - 检查会话密钥有效性
//! - [`RESET_SESSION_KEY_END_POINT`] - 重置用户会话密钥
//!
//! ## 用户信息相关
//!
//! - [`PHONE_END_POINT`] - 获取用户手机号
//! - [`AUTHENTICATION_END_POINT`] - 用户登录凭证校验
//!
//! ## 内容与媒体
//!
//! - [`QR_CODE_ENDPOINT`] - 生成小程序二维码
//! - [`MSG_SEC_CHECK_END_POINT`] - 内容安全检测
//!
//! # 版本信息
//!
//! 这些端点对应微信小程序最新的 API 版本，会随着微信官方 API 的更新而维护。

/// 获取稳定版访问令牌的 API 端点
/// # 官方文档
///
/// [获取稳定版接口调用凭据](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/mp-access-token/getStableAccessToken.html)
pub const STABLE_ACCESS_TOKEN_END_POINT: &str = "https://api.weixin.qq.com/cgi-bin/stable_token";

/// 获取普通访问令牌的 API 端点
/// # 官方文档
///
/// [获取接口调用凭据](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/mp-access-token/getAccessToken.html)
pub const ACCESS_TOKEN_END_POINT: &str = "https://api.weixin.qq.com/cgi-bin/token";

/// 检查会话密钥有效性的 API 端点
///
/// # 官方文档
///
/// [检查加密信息是否由微信生成](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-login/checkSessionKey.html)
pub const CHECK_SESSION_KEY_END_POINT: &str = "https://api.weixin.qq.com/wxa/checksession";

/// 重置用户会话密钥的 API 端点
/// # 官方文档
///
/// [重置用户会话密钥](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-login/resetSessionKey.html)
pub const RESET_SESSION_KEY_END_POINT: &str = "https://api.weixin.qq.com/wxa/resetusersessionkey";

/// 获取用户手机号的 API 端点
///
/// # 官方文档
///
/// [获取手机号](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-info/phone-number/getPhoneNumber.html)
pub const PHONE_END_POINT: &str = "https://api.weixin.qq.com/wxa/business/getuserphonenumber";

/// 用户登录凭证校验的 API 端点
///
/// # 官方文档
///
/// [code2Session](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-login/code2Session.html)
pub const AUTHENTICATION_END_POINT: &str = "https://api.weixin.qq.com/sns/jscode2session";

/// 生成小程序小程序码的 API 端点
///
/// # 官方文档
///
/// [获取小程序码](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/qrcode-link/qr-code/getQRCode.html)
pub const QR_CODE_ENDPOINT: &str = "https://api.weixin.qq.com/wxa/getwxacode";

/// 内容安全检测的 API 端点
///
/// # 官方文档
///
/// [文本安全检测](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/sec-center/sec-check/msgSecCheck.html)
pub const MSG_SEC_CHECK_END_POINT: &str = "https://api.weixin.qq.com/wxa/msg_sec_check";
