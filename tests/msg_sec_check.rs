use dotenv::dotenv;
use std::env;
use wechat_minapp_v1::Client;
use wechat_minapp_v1::minapp_security::{Args, Scene};

/// 初始化测试客户端
fn setup_client() -> Client {
    dotenv().ok();

    let app_id = env::var("WECHAT_APP_ID").expect("请设置 WECHAT_APP_ID 环境变量");
    let secret = env::var("WECHAT_APP_SECRET").expect("请设置 WECHAT_APP_SECRET 环境变量");

    Client::new(&app_id, &secret)
}

/// 获取测试用的用户openid
fn get_test_openid() -> String {
    env::var("WECHAT_TEST_OPENID").unwrap_or_else(|_| "test_openid_placeholder".to_string())
}

#[tokio::test]
async fn test_msg_sec_check_normal_content() {
    let client = setup_client();

    let args = Args::builder()
        .content("这是一段正常的文本内容，用于测试微信内容安全检测API。今天天气真好，阳光明媚，适合出门散步。")
        .scene(Scene::Comment)
        .openid(get_test_openid())
        .build()
        .unwrap();

    let result = client.msg_sec_check(&args).await;

    assert!(result.is_ok(), "API调用失败: {:?}", result.err());

    let result = result.unwrap();
    println!("正常内容检测结果: {:?}", result);

    assert!(result.is_success());
    assert!(result.errcode == 0);
    assert_eq!(result.errmsg, "ok");

    assert!(!result.is_risky(), "正常内容被判定为有风险: {:?}", result);
}

#[tokio::test]
async fn test_msg_sec_check_with_title_and_nickname() {
    let client = setup_client();

    let args = Args::builder()
        .content("这是一个带有标题和昵称的测试内容。内容本身是正常的，用于验证可选参数的功能。")
        .scene(Scene::Forum)
        .openid(get_test_openid())
        .title("测试文章标题")
        .nickname("测试用户昵称")
        .build()
        .unwrap();

    let result = client.msg_sec_check(&args).await;

    assert!(result.is_ok(), "API调用失败: {:?}", result.err());

    let result = result.unwrap();
    println!("带标题和昵称的检测结果: {:?}", result);

    assert!(result.is_success());
}

#[tokio::test]
async fn test_msg_sec_check_profile_scene() {
    let client = setup_client();

    let args = Args::builder()
        .content("这是一个用户的个人资料描述，包含一些基本的个人信息和兴趣爱好。")
        .scene(Scene::Profile)
        .openid(get_test_openid())
        .nickname("测试用户")
        .signature("这是一个性签名，用于测试资料场景")
        .build()
        .unwrap();

    let result = client.msg_sec_check(&args).await;

    assert!(result.is_ok(), "API调用失败: {:?}", result.err());

    let result = result.unwrap();
    println!("资料场景检测结果: {:?}", result);

    assert!(result.is_success());
}

#[tokio::test]
async fn test_msg_sec_check_different_scenes() {
    let client = setup_client();
    let openid = get_test_openid();
    let test_content = "今天天气不错";

    let comment_args = Args::builder()
        .content(test_content)
        .scene(Scene::Comment)
        .openid(&openid)
        .build()
        .unwrap();

    let comment_result = client.msg_sec_check(&comment_args).await;
    assert!(comment_result.is_ok());
    assert!(comment_result.unwrap().is_success());

    let forum_args = Args::builder()
        .content(test_content)
        .scene(Scene::Forum)
        .openid(&openid)
        .build()
        .unwrap();

    let forum_result = client.msg_sec_check(&forum_args).await;
    assert!(forum_result.is_ok());
    assert!(forum_result.unwrap().is_success());

    let social_args = Args::builder()
        .content(test_content)
        .scene(Scene::SocialLog)
        .openid(&openid)
        .build()
        .unwrap();

    let social_result = client.msg_sec_check(&social_args).await;
    assert!(social_result.is_ok());
    assert!(social_result.unwrap().is_success());
}

#[tokio::test]
async fn test_msg_sec_check_content_length_boundary() {
    let client = setup_client();

    let boundary_content = "check__content__long".repeat(125);
    assert_eq!(boundary_content.len(), 2500);

    let args = Args::builder()
        .content(&boundary_content)
        .scene(Scene::Comment)
        .openid(get_test_openid())
        .build()
        .unwrap();

    let result = client.msg_sec_check(&args).await;

    assert!(result.is_ok(), "边界长度内容检测失败: {:?}", result.err());

    let result = result.unwrap();
    println!("边界长度内容检测结果: {:?}", result);

    assert!(result.is_success());
}

#[tokio::test]
async fn test_msg_sec_check_result_structure() {
    let client = setup_client();

    let args = Args::builder()
        .content("验证返回结果结构的测试内容")
        .scene(Scene::Comment)
        .openid(get_test_openid())
        .build()
        .unwrap();

    let result = client.msg_sec_check(&args).await;
    print!("返回结果结构检测结果: {:?}", result);
    let result = result.unwrap();

    assert_eq!(result.errcode, 0);
    assert_eq!(result.errmsg, "ok");

    assert!(result.result.is_some());

    if let Some(trace_id) = result.trace_id {
        assert!(!trace_id.is_empty());
    }

    if let Some(details) = result.detail {
        for detail in details {
            assert!(!detail.strategy.is_empty());
            if detail.errcode == 0 {
                assert!(detail.prob.is_none() || (0.0..=100.0).contains(&detail.prob.unwrap()));
            }
        }
    }
}
