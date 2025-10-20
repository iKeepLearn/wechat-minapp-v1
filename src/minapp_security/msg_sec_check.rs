//! 微信小程序内容安全检测模块
//!
//! 该模块提供了微信小程序内容安全检测功能，用于检测文本内容是否包含违规信息。
//!
//! # 主要功能
//!
//! - 文本内容安全检测
//! - 多场景检测支持（资料、评论、论坛、社交日志）
//! - 详细的检测结果分析
//! - 置信度评分和关键词命中
//!
//! # 使用场景
//!
//! 适用于需要用户生成内容的场景：
//!
//! - 用户昵称、个性签名
//! - 评论、留言
//! - 论坛帖子、文章
//! - 社交动态、日志
//!
//! # 快速开始
//!
//! ```no_run
//! use wechat_minapp::{Client, minapp_security::{Args, Scene}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("app_id", "secret");
//!     
//!     let args = Args::builder()
//!         .content("需要检测的文本内容")
//!         .scene(Scene::Comment)
//!         .openid("user_openid")
//!         .build()?;
//!     
//!     let result = client.msg_sec_check(&args).await?;
//!     
//!     if result.is_pass() {
//!         println!("内容安全，可以发布");
//!     } else if result.needs_review() {
//!         println!("内容需要人工审核");
//!     } else {
//!         println!("内容有风险，建议修改");
//!     }
//!     
//!     Ok(())
//! }
//! ```

use super::{Label, Suggest};
use crate::{Result, client::Client, constants, error::Error};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// 内容安全检测场景
///
/// 定义不同的内容检测场景，不同场景有不同的检测策略和敏感度。
///
/// # 场景说明
///
/// - **资料**: 用户昵称、头像、个性签名等个人信息
/// - **评论**: 用户评论、留言等互动内容  
/// - **论坛**: 论坛帖子、文章等长文本内容
/// - **社交日志**: 朋友圈、动态等社交内容
///
/// # 示例
///
/// ```
/// use wechat_minapp::minapp_security::Scene;
///
/// let profile_scene = Scene::Profile;
/// let comment_scene = Scene::Comment;
/// let forum_scene = Scene::Forum;
/// let social_scene = Scene::SocialLog;
///
/// assert_eq!(profile_scene as u32, 1);
/// assert_eq!(profile_scene.description(), "资料");
/// ```
#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub enum Scene {
    /// 资料
    Profile = 1,
    /// 评论
    Comment = 2,
    /// 论坛
    Forum = 3,
    /// 社交日志
    SocialLog = 4,
}

/// 微信内容安全检测请求参数
///
/// 用于配置内容安全检测的各项参数，包括检测内容、场景、用户信息等。
///
/// # 字段说明
///
/// - `content`: 待检测的文本内容，最大长度2500字符
/// - `version`: 接口版本号，固定为2
/// - `scene`: 检测场景，不同场景有不同的检测策略
/// - `openid`: 用户openid，用户需在近两小时访问过小程序
/// - `title`: 文本标题（可选）
/// - `nickname`: 用户昵称（可选）
/// - `signature`: 个性签名，仅在资料场景有效（可选）
///
/// # 示例
///
/// ```
/// use wechat_minapp::minapp_security::{Args, Scene};
///
/// let args = Args::new("待检测的文本内容", Scene::Comment, "user_openid");
/// assert_eq!(args.content_length(), 18);
/// assert!(args.is_profile_scene());
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct Args {
    /// 需检测的文本内容，文本字数的上限为2500字，需使用UTF-8编码
    pub content: String,
    /// 接口版本号，2.0版本为固定值2
    pub version: u32,
    /// 场景枚举值
    pub scene: Scene,
    /// 用户的openid（用户需在近两小时访问过小程序）
    pub openid: String,
    /// 文本标题，需使用UTF-8编码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 用户昵称，需使用UTF-8编码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// 个性签名，该参数仅在资料类场景有效(scene=1)，需使用UTF-8编码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Args 构建器，提供链式调用和验证
///
/// 用于构建内容安全检测参数，提供参数验证和便捷的链式调用。
///
/// # 示例
///
/// ```
/// use wechat_minapp::minapp_security::{Args, Scene};
///
/// let args = Args::builder()
///     .content("待检测内容")
///     .scene(Scene::Comment)
///     .openid("user_openid")
///     .title("文章标题")
///     .nickname("用户昵称")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ArgsBuilder {
    content: Option<String>,
    version: Option<u32>,
    scene: Option<Scene>,
    openid: Option<String>,
    title: Option<String>,
    nickname: Option<String>,
    signature: Option<String>,
}

impl ArgsBuilder {
    /// 创建新的构建器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置检测文本内容
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// 设置接口版本号（通常为2）
    pub fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    /// 设置场景
    pub fn scene(mut self, scene: Scene) -> Self {
        self.scene = Some(scene);
        self
    }

    /// 设置用户openid
    pub fn openid(mut self, openid: impl Into<String>) -> Self {
        self.openid = Some(openid.into());
        self
    }

    /// 设置文本标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置用户昵称
    pub fn nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    /// 设置个性签名（仅在资料场景有效）
    pub fn signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    /// 构建 Args，验证必填字段
    pub fn build(self) -> Result<Args> {
        let content = self
            .content
            .ok_or(Error::InvalidParameter("content 是必填参数".to_string()))?;
        let version = self.version.unwrap_or(2); // 默认版本为2
        let scene = self
            .scene
            .ok_or(Error::InvalidParameter("scene 是必填参数".to_string()))?;
        let openid = self
            .openid
            .ok_or(Error::InvalidParameter("openid 是必填参数".to_string()))?;

        // 内容长度验证
        if content.len() > 2500 {
            return Err(Error::InvalidParameter(
                "content 长度不能超过2500字".to_string(),
            ));
        }

        // 场景与签名的关联验证
        if self.signature.is_some() && scene != Scene::Profile {
            return Err(Error::InvalidParameter(
                "signature 仅在资料场景(scene=1)下有效".to_string(),
            ));
        }

        Ok(Args {
            content,
            version,
            scene,
            openid,
            title: self.title,
            nickname: self.nickname,
            signature: self.signature,
        })
    }
}

// 为 Args 实现便捷的构建方法
impl Args {
    /// 创建构建器
    pub fn builder() -> ArgsBuilder {
        ArgsBuilder::new()
    }

    /// 快速创建基本参数（使用默认版本2）
    pub fn new(content: impl Into<String>, scene: Scene, openid: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            version: 2,
            scene,
            openid: openid.into(),
            title: None,
            nickname: None,
            signature: None,
        }
    }

    /// 检查是否为资料场景
    pub fn is_profile_scene(&self) -> bool {
        self.scene == Scene::Profile
    }

    /// 获取内容长度
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// 验证参数是否有效
    pub fn validate(&self) -> Result<()> {
        if self.content.len() > 2500 {
            return Err(Error::InvalidParameter(
                "content 长度不能超过2500字".to_string(),
            ));
        }

        if self.signature.is_some() && !self.is_profile_scene() {
            return Err(Error::InvalidParameter(
                "signature 仅在资料场景(scene=1)下有效".to_string(),
            ));
        }

        Ok(())
    }
}

// Scene 枚举的便捷方法
impl Scene {
    /// 从数值创建场景
    pub fn from_value(value: u32) -> Option<Self> {
        match value {
            1 => Some(Scene::Profile),
            2 => Some(Scene::Comment),
            3 => Some(Scene::Forum),
            4 => Some(Scene::SocialLog),
            _ => None,
        }
    }

    /// 获取场景描述
    pub fn description(&self) -> &'static str {
        match self {
            Scene::Profile => "资料",
            Scene::Comment => "评论",
            Scene::Forum => "论坛",
            Scene::SocialLog => "社交日志",
        }
    }
}

/// 详细检测结果
///
/// 包含具体的检测策略、建议、标签和置信度等信息。
///
/// # 字段说明
///
/// - `strategy`: 使用的检测策略类型
/// - `errcode`: 错误码，0表示该项结果有效
/// - `suggest`: 检测建议
/// - `label`: 命中的标签类型
/// - `keyword`: 命中的自定义关键词
/// - `prob`: 置信度，0-100，越高越可能属于当前标签
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DetailResult {
    /// 策略类型
    pub strategy: String,
    /// 错误码，仅当该值为0时，该项结果有效
    pub errcode: i32,
    /// 建议
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggest: Option<Suggest>,
    /// 命中标签枚举值（可能不存在）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,
    /// 命中的自定义关键词（可能不存在）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// 0-100，代表置信度，越高代表越有可能属于当前返回的标签（label）（可能不存在）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prob: Option<f64>,
}

/// 综合结果
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComprehensiveResult {
    /// 建议
    pub suggest: Suggest,
    /// 命中标签枚举值
    pub label: Label,
}

/// 内容安全检测返回结果
///
/// 包含内容安全检测的完整结果信息。
///
/// # 字段说明
///
/// - `errcode`: 全局错误码，0表示请求成功
/// - `errmsg`: 错误信息
/// - `detail`: 详细的检测结果列表
/// - `result`: 综合检测结果
/// - `trace_id`: 唯一请求标识，用于问题排查
///
/// # 示例
///
/// ```no_run
/// use wechat_minapp::minapp_security::MsgSecCheckResult;
///
/// # fn process_result(result: MsgSecCheckResult) {
/// if result.is_success() {
///     if result.is_pass() {
///         println!("内容安全");
///     } else if result.needs_review() {
///         println!("需要人工审核");
///     } else {
///         println!("内容有风险");
///     }
///     
///     for detail in result.get_valid_details() {
///         println!("策略: {}, 置信度: {:?}", detail.strategy, detail.prob);
///     }
/// }
/// # }
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MsgSecCheckResult {
    /// 错误码
    pub errcode: i32,
    /// 错误信息
    pub errmsg: String,
    /// 详细检测结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Vec<DetailResult>>,
    /// 综合结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ComprehensiveResult>,
    /// 唯一请求标识，标记单次请求
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

// 为 MsgSecCheckResult 实现一些便捷方法
impl MsgSecCheckResult {
    /// 检查请求是否成功（errcode 为 0）
    pub fn is_success(&self) -> bool {
        self.errcode == 0
    }

    /// 获取综合建议
    pub fn get_suggest(&self) -> Option<&Suggest> {
        self.result.as_ref().map(|r| &r.suggest)
    }

    /// 获取综合标签
    pub fn get_label(&self) -> Option<&Label> {
        self.result.as_ref().map(|r| &r.label)
    }

    /// 检查是否通过
    pub fn is_pass(&self) -> bool {
        self.get_suggest().map(|s| s.is_pass()).unwrap_or(false)
    }

    /// 检查是否有风险
    pub fn is_risky(&self) -> bool {
        self.get_suggest().map(|s| s.is_risky()).unwrap_or(false)
    }

    /// 检查是否需要审核
    pub fn needs_review(&self) -> bool {
        self.get_suggest()
            .map(|s| s.needs_review())
            .unwrap_or(false)
    }

    /// 获取有效的详细检测结果（errcode 为 0 的项）
    pub fn get_valid_details(&self) -> Vec<&DetailResult> {
        self.detail
            .as_ref()
            .map(|details| details.iter().filter(|d| d.errcode == 0).collect())
            .unwrap_or_default()
    }
}

impl Client {
    /// 内容安全检测
    ///
    /// 对文本内容进行安全检测，识别违规内容。
    ///
    /// # 参数
    ///
    /// - `args`: 内容安全检测参数
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(MsgSecCheckResult)`，包含检测结果
    ///
    /// # 错误
    ///
    /// - 参数验证错误
    /// - 网络错误
    /// - 微信 API 返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use wechat_minapp::{Client, minapp_security::{Args, Scene}};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new("app_id", "secret");
    ///     
    ///     let args = Args::builder()
    ///         .content("需要检测的文本内容")
    ///         .scene(Scene::Comment)
    ///         .openid("user_openid")
    ///         .build()?;
    ///     
    ///     let result = client.msg_sec_check(&args).await?;
    ///     
    ///     match (result.is_pass(), result.needs_review(), result.is_risky()) {
    ///         (true, _, _) => println!("内容安全，可以发布"),
    ///         (_, true, _) => println!("内容需要人工审核"),
    ///         (_, _, true) => println!("内容有风险，建议修改"),
    ///         _ => println!("未知状态"),
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # API 文档
    ///
    /// [文本安全检测](https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/sec-center/sec-check/msgSecCheck.html)
    pub async fn msg_sec_check(&self, args: &Args) -> Result<MsgSecCheckResult> {
        debug!("msg_sec_check args: {:?}", &args);

        // 验证参数
        args.validate()?;
        let access_token = self.access_token().await?;
        let mut query = HashMap::new();
        let mut body = HashMap::new();
        let version = args.version.to_string();
        let scene = (args.scene as u32).to_string();
        // URL 参数：access_token
        query.insert("access_token", &access_token);

        // Body 参数
        body.insert("content", &args.content);
        body.insert("version", &version);
        body.insert("scene", &scene);
        body.insert("openid", &args.openid);

        if let Some(title) = &args.title {
            body.insert("title", title);
        }

        if let Some(nickname) = &args.nickname {
            body.insert("nickname", nickname);
        }

        if let Some(signature) = &args.signature {
            body.insert("signature", signature);
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .request()
            .post(constants::MSG_SEC_CHECK_END_POINT)
            .headers(headers)
            .query(&query)
            .json(&body)
            .send()
            .await?;

        debug!("msg_sec_check response: {:#?}", response);

        if response.status().is_success() {
            let response_text = response.text().await?;
            debug!("msg_sec_check response body: {}", response_text);

            let result: MsgSecCheckResult = serde_json::from_str(&response_text)?;

            if result.is_success() {
                Ok(result)
            } else {
                // 微信API返回错误
                Err(Error::InternalServer(format!(
                    "微信内容安全检测API错误: {} - {}",
                    result.errcode, result.errmsg
                )))
            }
        } else {
            // HTTP 请求错误
            Err(Error::InternalServer(response.text().await?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_builder() {
        let args = Args::builder()
            .content("测试内容")
            .scene(Scene::Comment)
            .openid("test_openid")
            .build()
            .unwrap();

        assert_eq!(args.content, "测试内容");
        assert_eq!(args.version, 2);
        assert_eq!(args.scene, Scene::Comment);
        assert_eq!(args.openid, "test_openid");
    }

    #[test]
    fn test_args_builder_validation() {
        // 测试缺少必填参数
        let result = Args::builder()
            .scene(Scene::Comment)
            .openid("test_openid")
            .build();
        assert!(result.is_err());

        // 测试内容超长
        let long_content = "a".repeat(2501);
        let result = Args::builder()
            .content(long_content)
            .scene(Scene::Comment)
            .openid("openid")
            .build();
        assert!(result.is_err());

        // 测试场景与签名验证
        let result = Args::builder()
            .content("内容")
            .scene(Scene::Comment)
            .openid("openid")
            .signature("签名")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_enum() {
        assert_eq!(Scene::from_value(1), Some(Scene::Profile));
        assert_eq!(Scene::Profile.description(), "资料");
        assert_eq!(Scene::Profile as u32, 1);
    }

    #[test]
    fn test_msg_sec_check_result() {
        let json = r#"
        {
            "errcode": 0,
            "errmsg": "ok",
            "detail": [
                {
                    "strategy": "content_model",
                    "errcode": 0,
                    "suggest": "pass",
                    "label": 100,
                    "prob": 90.5
                }
            ],
            "result": {
                "suggest": "pass",
                "label": 100
            },
            "trace_id": "test_trace_id"
        }"#;

        let result: MsgSecCheckResult = serde_json::from_str(json).unwrap();

        assert!(result.is_success());
        assert!(result.is_pass());
        assert!(!result.is_risky());
        assert!(!result.needs_review());
        assert_eq!(result.get_valid_details().len(), 1);
        assert_eq!(result.trace_id, Some("test_trace_id".to_string()));
    }

    #[test]
    fn test_msg_sec_check_result_with_risk() {
        let json = r#"
        {
            "errcode": 0,
            "errmsg": "ok",
            "detail": [
                {
                    "strategy": "content_model",
                    "errcode": 0,
                    "suggest": "risky",
                    "label": 20001,
                    "keyword": "敏感词",
                    "prob": 95.0
                }
            ],
            "result": {
                "suggest": "risky",
                "label": 20001
            }
        }"#;

        let result: MsgSecCheckResult = serde_json::from_str(json).unwrap();

        assert!(result.is_success());
        assert!(!result.is_pass());
        assert!(result.is_risky());
        assert!(!result.needs_review());
        assert_eq!(
            result.get_valid_details()[0].keyword,
            Some("敏感词".to_string())
        );
    }
}
