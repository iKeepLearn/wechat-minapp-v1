# wechat-minapp

![WeChat](https://img.shields.io/badge/WeChat-07C160?style=for-the-badge&logo=wechat&logoColor=white)

A rust sdk for wechat miniprogram server api

基于 [open-wechat](https://github.com/headironc/open-wechat) 修改

首先感谢 headironc 的项目，之所以重新发一包，而不是 pr 是因为我改了很多结构，现在 wechat-minapp 的调用方式出现了很大的不同。

## 用法

### 获取 access token

```rust
use wechat_minapp::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_id = "your app id";
    let app_secret = "your app secret";

    let client = Client::new(app_id, app_secret);

    let access_token = client.access_token().await?;

    Ok(())
}
```

### 获取 stable access token

```rust
use wechat_minapp::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_id = "your app id";
    let app_secret = "your app secret";

    let client = Client::new(app_id, app_secret);
    let force_refresh = Some(true)
    let access_token = client.stable_access_token(force_refresh).await?;

    Ok(())
}
```

### 登录

```rust
use wechat_minapp::Client;
use serde::Deserialize;

use crate::{Error, state::AppState};
use actix_web::{Responder, web};

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Logger {
    code: String,
}

pub async fn login(
    state: web::Data<AppState>,
    logger: web::Json<Logger>,
) -> Result<impl Responder, Error> {
    let credential = state.client.login(&logger.code).await?;

    Ok(())
}
```

### 解码用户信息

```rust
use wechat_minapp::Client;
use serde::Deserialize;

use crate::{Error, state::AppState};
use actix_web::{Responder, web};

#[derive(Deserialize, Default)]
pub struct EncryptedPayload {
    code: String,
    encrypted_data: String,
    iv: String,
}

pub async fn decrypt(
    state: web::Data<AppState>,
    payload: web::Json<EncryptedPayload>,
) -> Result<impl Responder, Error> {
    let credential = state.client.login(&payload.code).await?;
    let user = credential.decrypt(&payload.encrypted_data, &payload.iv)?;

    Ok(())
}

```

### 生成小程序码

```rust
use wechat_minapp::{Client,QrCodeArgs};
use serde::Deserialize;

use crate::{Error, state::AppState};
use actix_web::{Responder, web};


pub async fn get_user_qr(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
) -> Result<impl Responder, Error> {

    let page:&str = "/index";
    let qr_args = QrCodeArgs::builder().path(&page).build()?;
    let buffer = state.client.qr_code(qr_args).await?;

    Ok(buffer)
}

```


### 检查文本内容安全

```rust
use wechat_minapp::minapp_security::{Args, Scene};
use wechant_minapp::Client;
use crate::{Error, state::AppState};
use actix_web::{Responder, web};
use serde::{Deserialize,Serialize};

#[derive(Deserialize,Serialize)]
pub struct ContentCheck {
    content: String,
    scene: Scene,
}

pub async fn get_user_qr(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    date:web::Json<ContentCheck>
) -> Result<impl Responder, Error> {

let args = Args::builder()
        .content(&data.content)
        .scene(data.scene)
        .openid(user.openid)
        .build()?;
    
    let result = state.client.msg_sec_check(args).await?;
    
    if result.is_pass() {
        println!("内容安全，可以发布");
    } else if result.needs_review() {
        println!("内容需要人工审核");
    } else {
        println!("内容有风险，建议修改");
    }
    
    Ok(web::Json(result))
}

```
