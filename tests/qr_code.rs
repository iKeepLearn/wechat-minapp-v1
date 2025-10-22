use dotenv::dotenv;
use std::env;
use wechat_minapp_v1::{Client, MinappEnvVersion, QrCodeArgs, Rgb};

/// 初始化测试客户端
fn setup_client() -> Client {
    dotenv().ok();

    let app_id = env::var("WECHAT_APP_ID").expect("请设置 WECHAT_APP_ID 环境变量");
    let secret = env::var("WECHAT_APP_SECRET").expect("请设置 WECHAT_APP_SECRET 环境变量");

    Client::new(&app_id, &secret)
}

#[test]
fn test_minapp_env_version_conversion() {
    let develop: String = MinappEnvVersion::Develop.into();
    let release: String = MinappEnvVersion::Release.into();
    let trial: String = MinappEnvVersion::Trial.into();

    assert_eq!(develop, "develop");
    assert_eq!(release, "release");
    assert_eq!(trial, "trial");
}

#[test]
fn test_qr_code_args_build_success() {
    let args = QrCodeArgs::builder()
        .path("pages/index/index")
        .build()
        .expect("构建应该成功");

    assert_eq!(args.path(), "pages/index/index");
    assert!(args.width().is_none());
    assert!(args.auto_color().is_none());
    assert!(args.line_color().is_none());
    assert!(args.is_hyaline().is_none());
    assert!(args.env_version().is_none());
}

#[test]
fn test_qr_code_args_build_with_all_fields() {
    let args = QrCodeArgs::builder()
        .path("pages/detail/detail")
        .width(400)
        .with_auto_color()
        .line_color(Rgb::new(0, 255, 0))
        .with_is_hyaline()
        .env_version(MinappEnvVersion::Develop)
        .build()
        .expect("构建应该成功");

    assert_eq!(args.path(), "pages/detail/detail");
    assert_eq!(args.width(), Some(400));
    assert_eq!(args.auto_color(), Some(true));
    assert!(args.line_color().is_some());
    assert_eq!(args.is_hyaline(), Some(true));
    assert!(matches!(
        args.env_version(),
        Some(MinappEnvVersion::Develop)
    ));
}

#[test]
fn test_qr_code_args_build_missing_path() {
    let result = QrCodeArgs::builder().build();
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.to_string().contains("小程序页面路径不能为空"));
    }
}

#[test]
fn test_qr_code_args_build_path_too_long() {
    let long_path = "a".repeat(1025);
    let result = QrCodeArgs::builder().path(long_path).build();
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.to_string().contains("页面路径最大长度 1024 个字符"));
    }
}

#[test]
fn test_qr_code_args_build_path_boundary() {
    // 测试边界情况：正好 1024 个字符
    let boundary_path = "a".repeat(1024);
    let result = QrCodeArgs::builder().path(boundary_path).build();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_qr_code_with_all_parameters() {
    let client = setup_client();

    let args = QrCodeArgs::builder()
        .path("pages/index/index")
        .width(300)
        .with_auto_color()
        .line_color(Rgb::new(255, 0, 0))
        .with_is_hyaline()
        .env_version(MinappEnvVersion::Release)
        .build()
        .unwrap();

    let result = client.qr_code(args).await;

    assert!(result.is_ok());
    let qr_code = result.unwrap();
    assert!(!qr_code.buffer().is_empty());
}

#[tokio::test]
async fn test_qr_code_with_only_width() {
    let client = setup_client();

    let args = QrCodeArgs::builder()
        .path("pages/index/index")
        .width(200)
        .build()
        .unwrap();

    let result = client.qr_code(args).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_qr_code_with_only_env_version() {
    let client = setup_client();

    let args = QrCodeArgs::builder()
        .path("pages/index/index")
        .env_version(MinappEnvVersion::Develop)
        .build()
        .unwrap();

    let result = client.qr_code(args).await;

    assert!(result.is_ok());
}
