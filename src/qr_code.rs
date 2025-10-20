//! 微信小程序小程序码生成模块
//!
//! 该模块提供了生成微信小程序小程序码的功能，支持多种类型的小程序码和自定义参数。
//!
//! # 主要功能
//!
//! - 生成小程序页面小程序码
//! - 支持自定义尺寸、颜色、透明度等参数
//! - 支持不同环境版本（开发版、体验版、正式版）
//! - 链式参数构建器模式
//!
//! # 快速开始
//!
//! ```no_run
//! use wechat_minapp::{Client, QrCodeArgs, MinappEnvVersion};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 初始化客户端
//!     let app_id = "your_app_id";
//!     let secret = "your_app_secret";
//!     let client = Client::new(app_id, secret);
//!
//!     // 构建小程序码参数
//!     let args = QrCodeArgs::builder()
//!         .path("pages/index/index")
//!         .width(300)
//!         .env_version(MinappEnvVersion::Release)
//!         .build()?;
//!
//!     // 生成小程序码
//!     let qr_code = client.qr_code(args).await?;
//!     
//!     // 获取小程序码图片数据
//!     let buffer = qr_code.buffer();
//!     println!("生成的小程序码大小: {} bytes", buffer.len());
//!
//!     // 可以将 buffer 保存为文件或直接返回给前端
//!     // std::fs::write("qrcode.png", buffer)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # 参数说明
//!
//! - `path`: 小程序页面路径，必填，最大长度 1024 字符
//! - `width`: 小程序码宽度，单位 px，最小 280px，最大 1280px
//! - `auto_color`: 是否自动配置线条颜色
//! - `line_color`: 自定义线条颜色，RGB 格式
//! - `is_hyaline`: 是否透明背景
//! - `env_version`: 环境版本，默认为正式版
//!
//! # 注意事项
//!
//! - 生成的小程序码永不过期，数量不限
//! - 接口只能生成已发布的小程序的小程序码
//! - 支持带参数路径，如 `pages/index/index?param=value`
//! - 小程序码大小限制为 128KB，请合理设置 width 参数
//!
//! # 示例
//!
//! ## 生成带颜色的小程序码
//!
//! ```no_run
//! use wechat_minapp::{QrCodeArgs, Rgb, MinappEnvVersion};
//!
//! let args = QrCodeArgs::builder()
//!     .path("pages/detail/detail?id=123")
//!     .width(400)
//!     .line_color(Rgb::new(255, 0, 0)) // 红色线条
//!     .with_is_hyaline() // 透明背景
//!     .env_version(MinappEnvVersion::Develop)
//!     .build()
//!     .unwrap();
//! ```
//!
//! ## 生成简单小程序码
//!
//! ```no_run
//! use wechat_minapp::QrCodeArgs;
//!
//! let args = QrCodeArgs::builder()
//!     .path("pages/index/index")
//!     .build()
//!     .unwrap();
//! ```
//!
//! # 错误处理
//!
//! 小程序码生成可能遇到以下错误：
//!
//! - 参数验证错误（路径为空或过长）
//! - 认证错误（access_token 无效）
//! - 网络错误
//! - 微信 API 返回错误
//!
//! 建议在生产环境中妥善处理这些错误。

use crate::{
    Client, Result, constants,
    error::Error::{self, InternalServer},
};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// 二维码图片数据
///
/// 包含生成的二维码图片的二进制数据，通常是 PNG 格式。
///
/// # 示例
///
/// ```no_run
/// use wechat_minapp::{Client, QrCodeArgs};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("app_id", "secret");
/// let args = QrCodeArgs::builder().path("pages/index/index").build()?;
/// let qr_code = client.qr_code(args).await?;
///
/// // 获取二维码数据
/// let buffer = qr_code.buffer();
/// println!("二维码大小: {} bytes", buffer.len());
///
/// // 保存到文件
/// // std::fs::write("qrcode.png", buffer)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QrCode {
    buffer: Vec<u8>,
}

impl QrCode {
    /// 获取二维码图片的二进制数据
    ///
    /// 返回的字节向量通常是 PNG 格式的图片数据，可以直接写入文件或返回给 HTTP 响应。
    ///
    /// # 返回
    ///
    /// 二维码图片的二进制数据引用
    pub fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }
}

/// 二维码生成参数
///
/// 用于配置二维码的生成选项，通过 [`QrCodeArgs::builder()`] 方法创建。
#[derive(Debug, Deserialize)]
pub struct QrCodeArgs {
    path: String,
    width: Option<i16>,
    auto_color: Option<bool>,
    line_color: Option<Rgb>,
    is_hyaline: Option<bool>,
    env_version: Option<MinappEnvVersion>,
}

/// 二维码参数构建器
///
/// 提供链式调用的方式构建二维码参数，确保参数的正确性。
///
/// # 示例
///
/// ```
/// use wechat_minapp::{QrCodeArgs, Rgb, MinappEnvVersion};
///
/// let args = QrCodeArgs::builder()
///     .path("pages/index/index")
///     .width(300)
///     .with_auto_color()
///     .line_color(Rgb::new(255, 0, 0))
///     .with_is_hyaline()
///     .env_version(MinappEnvVersion::Release)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Deserialize)]
pub struct QrCodeArgBuilder {
    path: Option<String>,
    width: Option<i16>,
    auto_color: Option<bool>,
    line_color: Option<Rgb>,
    is_hyaline: Option<bool>,
    env_version: Option<MinappEnvVersion>,
}

// RGB 颜色值
///
/// 用于自定义二维码线条颜色。
///
/// # 示例
///
/// ```
/// use wechat_minapp::Rgb;
///
/// let red = Rgb::new(255, 0, 0);      // 红色
/// let green = Rgb::new(0, 255, 0);    // 绿色
/// let blue = Rgb::new(0, 0, 255);     // 蓝色
/// let black = Rgb::new(0, 0, 0);      // 黑色
/// ```
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Rgb {
    r: i16,
    g: i16,
    b: i16,
}

impl Rgb {
    /// 创建新的 RGB 颜色
    ///
    /// # 参数
    ///
    /// - `r`: 红色分量 (0-255)
    /// - `g`: 绿色分量 (0-255)
    /// - `b`: 蓝色分量 (0-255)
    ///
    /// # 返回
    ///
    /// 新的 Rgb 实例
    pub fn new(r: i16, g: i16, b: i16) -> Self {
        Rgb { r, g, b }
    }
}

impl QrCodeArgs {
    pub fn builder() -> QrCodeArgBuilder {
        QrCodeArgBuilder::new()
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn width(&self) -> Option<i16> {
        self.width
    }

    pub fn auto_color(&self) -> Option<bool> {
        self.auto_color
    }

    pub fn line_color(&self) -> Option<Rgb> {
        self.line_color.clone()
    }

    pub fn is_hyaline(&self) -> Option<bool> {
        self.is_hyaline
    }

    pub fn env_version(&self) -> Option<MinappEnvVersion> {
        self.env_version.clone()
    }
}

impl Default for QrCodeArgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 小程序环境版本
///
/// 指定二维码生成的环境版本，不同环境版本对应不同的小程序实例。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MinappEnvVersion {
    /// 开发版，用于开发环境
    Release,
    /// 体验版，用于测试环境
    Trial,
    /// 正式版，用于生产环境
    Develop,
}

impl From<MinappEnvVersion> for String {
    fn from(value: MinappEnvVersion) -> Self {
        match value {
            MinappEnvVersion::Develop => "develop".to_string(),
            MinappEnvVersion::Release => "release".to_string(),
            MinappEnvVersion::Trial => "trial".to_string(),
        }
    }
}

impl QrCodeArgBuilder {
    pub fn new() -> Self {
        QrCodeArgBuilder {
            path: None,
            width: None,
            auto_color: None,
            line_color: None,
            is_hyaline: None,
            env_version: None,
        }
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn width(mut self, width: i16) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_auto_color(mut self) -> Self {
        self.auto_color = Some(true);
        self
    }

    pub fn line_color(mut self, color: Rgb) -> Self {
        self.line_color = Some(color);
        self
    }

    pub fn with_is_hyaline(mut self) -> Self {
        self.is_hyaline = Some(true);
        self
    }

    pub fn env_version(mut self, version: MinappEnvVersion) -> Self {
        self.env_version = Some(version);
        self
    }

    pub fn build(self) -> Result<QrCodeArgs> {
        let path = self.path.map_or_else(
            || {
                Err(Error::InvalidParameter(
                    "小程序页面路径不能为空".to_string(),
                ))
            },
            |v| {
                if v.len() > 1024 {
                    return Err(Error::InvalidParameter(
                        "页面路径最大长度 1024 个字符".to_string(),
                    ));
                }
                Ok(v)
            },
        )?;

        Ok(QrCodeArgs {
            path,
            width: self.width,
            auto_color: self.auto_color,
            line_color: self.line_color,
            is_hyaline: self.is_hyaline,
            env_version: self.env_version,
        })
    }
}

impl Client {
    /// 生成小程序二维码
    ///
    /// 调用微信小程序二维码生成接口，返回包含二维码图片数据的 [`QrCode`] 对象。
    ///
    /// # 参数
    ///
    /// - `args`: 二维码生成参数
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(QrCode)`，失败返回错误信息。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use wechat_minapp::{Client, QrCodeArgs};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("app_id", "secret");
    /// let args = QrCodeArgs::builder()
    ///     .path("pages/index/index")
    ///     .width(300)
    ///     .build()?;
    ///     
    /// let qr_code = client.qr_code(args).await?;
    /// println!("二维码生成成功，大小: {} bytes", qr_code.buffer().len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # 错误
    ///
    /// - 网络错误
    /// - 认证错误（access_token 无效）
    /// - 微信 API 返回错误
    /// - 参数序列化错误
    pub async fn qr_code(&self, args: QrCodeArgs) -> Result<QrCode> {
        debug!("get qr code args {:?}", &args);

        let mut query = HashMap::new();
        let mut body = HashMap::new();

        query.insert("access_token", self.token().await?);
        body.insert("path", args.path);

        if let Some(width) = args.width {
            body.insert("width", width.to_string());
        }

        if let Some(auto_color) = args.auto_color {
            body.insert("auto_color", auto_color.to_string());
        }

        if let Some(line_color) = args.line_color {
            let value = serde_json::to_string(&line_color)?;
            body.insert("line_color", value);
        }

        if let Some(is_hyaline) = args.is_hyaline {
            body.insert("is_hyaline", is_hyaline.to_string());
        }

        if let Some(env_version) = args.env_version {
            body.insert("env_version", env_version.into());
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("encoding", HeaderValue::from_static("null"));

        let response = self
            .request()
            .post(constants::QR_CODE_ENDPOINT)
            .headers(headers)
            .query(&query)
            .json(&body)
            .send()
            .await?;

        debug!("response: {:#?}", response);

        if response.status().is_success() {
            let response = response.bytes().await?;

            Ok(QrCode {
                buffer: response.to_vec(),
            })
        } else {
            Err(InternalServer(response.text().await?))
        }
    }
}
