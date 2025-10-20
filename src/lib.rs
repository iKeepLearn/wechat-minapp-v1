//! `wechat_minapp` - 微信小程序服务端 API 封装库
//!
//! 这是一个为微信小程序服务端 API 提供的 Rust 封装库，旨在简化与微信小程序后端的交互。
//! 提供了诸如用户登录、内容安全检测、小程序码生成等常用功能的易用接口。
//!
//!
//! ## 核心特性
//!
//! - **易用性**: 提供简洁的 API 和链式构建器，简化开发流程。
//! - **安全性**: 自动处理访问令牌的获取和刷新，保障数据安全。
//! - **可靠性**: 针对网络请求和 API 错误进行处理，提供稳定的服务。
//! - **灵活性**: 支持自定义 HTTP 客户端配置，方便集成和测试。
//! - **并发性**: 支持在并发环境中使用。
//!
//!
//! [更多示例](https://github.com/iKeepLearn/wechat-minapp)

mod access_token;
mod client;
mod credential;
mod qr_code;
mod response;

pub mod constants;
pub mod error;
pub mod minapp_security;
pub mod user;

pub type Result<T> = std::result::Result<T, error::Error>;
pub use client::Client;
pub use qr_code::{MinappEnvVersion, QrCode, QrCodeArgs, Rgb};
