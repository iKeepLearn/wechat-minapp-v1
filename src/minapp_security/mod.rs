//! 微信小程序内容安全检测模块
//!
//! - [`msg_sec_check`][]: 文本内容安全检测。
//!

mod msg_sec_check;

use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use strum::Display;

pub use msg_sec_check::{Args, MsgSecCheckResult, Scene};

#[derive(Debug, Deserialize_repr, Display, Serialize, PartialEq, Clone)]
#[repr(i32)]
pub enum Label {
    #[strum(serialize = "正常")]
    Normal = 100,

    #[strum(serialize = "广告")]
    Ad = 10001,

    #[strum(serialize = "时政")]
    Politics = 20001,

    #[strum(serialize = "色情")]
    Porn = 20002,

    #[strum(serialize = "辱骂")]
    Abuse = 20003,

    #[strum(serialize = "违法犯罪")]
    Illegal = 20006,

    #[strum(serialize = "欺诈")]
    Fraud = 20008,

    #[strum(serialize = "低俗")]
    Vulgar = 20012,

    #[strum(serialize = "版权")]
    Copyright = 20013,

    #[strum(serialize = "其他")]
    Other = 21000,
}

// 可选：为方便使用，可以添加一些辅助方法
impl Label {
    /// 根据数值获取对应的标签枚举
    pub fn from_value(value: i32) -> Option<Self> {
        match value {
            100 => Some(Label::Normal),
            10001 => Some(Label::Ad),
            20001 => Some(Label::Politics),
            20002 => Some(Label::Porn),
            20003 => Some(Label::Abuse),
            20006 => Some(Label::Illegal),
            20008 => Some(Label::Fraud),
            20012 => Some(Label::Vulgar),
            20013 => Some(Label::Copyright),
            21000 => Some(Label::Other),
            _ => None,
        }
    }

    /// 检查是否为正常内容
    pub fn is_normal(&self) -> bool {
        matches!(self, Label::Normal)
    }

    /// 检查是否为违规内容
    pub fn is_violation(&self) -> bool {
        !self.is_normal()
    }
}

/// 内容安全检测建议
#[derive(Debug, Deserialize, Serialize, Display, PartialEq, Clone)]
pub enum Suggest {
    #[strum(serialize = "risky")]
    #[serde(rename = "risky")]
    Risky,

    #[strum(serialize = "pass")]
    #[serde(rename = "pass")]
    Pass,

    #[strum(serialize = "review")]
    #[serde(rename = "review")]
    Review,
}

// 可选：为方便使用，可以添加一些辅助方法
impl Suggest {
    /// 检查是否通过
    pub fn is_pass(&self) -> bool {
        matches!(self, Suggest::Pass)
    }

    /// 检查是否有风险
    pub fn is_risky(&self) -> bool {
        matches!(self, Suggest::Risky)
    }

    /// 检查是否需要人工审核
    pub fn needs_review(&self) -> bool {
        matches!(self, Suggest::Review)
    }

    /// 获取建议的优先级（数值越小优先级越高）
    pub fn priority(&self) -> u8 {
        match self {
            Suggest::Risky => 1,  // 最高优先级：有风险
            Suggest::Review => 2, // 中等优先级：需要审核
            Suggest::Pass => 3,   // 最低优先级：通过
        }
    }
}

impl From<&str> for Suggest {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "risky" => Suggest::Risky,
            "pass" => Suggest::Pass,
            "review" => Suggest::Review,
            _ => Suggest::Review,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_enum() {
        // 测试显示功能
        assert_eq!(Suggest::Risky.to_string(), "risky");
        assert_eq!(Suggest::Pass.to_string(), "pass");
        assert_eq!(Suggest::Review.to_string(), "review");

        // 测试从字符串创建
        assert_eq!(Suggest::from("risky"), Suggest::Risky);
        assert_eq!(Suggest::from("PASS"), Suggest::Pass);
        assert_eq!(Suggest::from("ReViEw"), Suggest::Review);
        assert_eq!(Suggest::from("invalid"), Suggest::Review);

        // 测试辅助方法
        assert!(Suggest::Pass.is_pass());
        assert!(Suggest::Risky.is_risky());
        assert!(Suggest::Review.needs_review());

        // 测试优先级
        assert_eq!(Suggest::Risky.priority(), 1);
        assert_eq!(Suggest::Review.priority(), 2);
        assert_eq!(Suggest::Pass.priority(), 3);
    }

    #[test]
    fn test_serialization() {
        // 测试序列化
        let risky_json = serde_json::to_string(&Suggest::Risky).unwrap();
        assert_eq!(risky_json, "\"risky\"");

        let pass_json = serde_json::to_string(&Suggest::Pass).unwrap();
        assert_eq!(pass_json, "\"pass\"");

        // 测试反序列化
        let review: Suggest = serde_json::from_str("\"review\"").unwrap();
        assert_eq!(review, Suggest::Review);
    }
}
