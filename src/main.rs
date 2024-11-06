use actix_web::{web, App, HttpServer, Responder, HttpResponse, post, get};
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use dotenv::dotenv;
use std::net::TcpListener;
use std::fs;


#[derive(serde::Deserialize)]
struct SetApiKeyRequest {
    apikey: String,
}

// .envファイルにAPIキーを書き込む関数
fn write_api_key_to_env(apikey: &str) -> io::Result<()> {
    // .envファイルを開く、存在しない場合は新たに作成
    let file_path = ".env";
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)  // 既存の内容をクリアする
        .open(file_path)?;

    // APIキーを書き込む
    writeln!(file, "OPENAI_API_KEY={}", apikey)?;
    Ok(())
}

#[post("/api/set_apikey")]
async fn set_apikey(web::Json(request): web::Json<SetApiKeyRequest>) -> impl Responder {
    // 環境変数に API_KEY を設定
    let _ = write_api_key_to_env(&request.apikey);
    env::set_var("OPENAI_API_KEY", &request.apikey);
    HttpResponse::Ok().append_header(("Access-Control-Allow-Origin", "*")).body("API key set successfully")
}

#[get("/api/apikey")]
async fn get_apikey() -> impl Responder {
    // 環境変数から API_KEY を取得
    match env::var("OPENAI_API_KEY") {
        Ok(api_key) => HttpResponse::Ok().append_header(("Access-Control-Allow-Origin", "*")).body(api_key),
        Err(_) => HttpResponse::NotFound().body("API key not found"),
    }
}

#[get("/api/version")]
async fn get_version() -> impl Responder {
    HttpResponse::Ok().append_header(("Access-Control-Allow-Origin", "*")).body("yukari-engine: 0.1.0")
}

// 新しいGETハンドラを追加
#[get("/{filename:.*}")]
async fn get_file(path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner(); // Pathを取り出してStringを取得

    // filename が "api/" で始まる場合は204 No Contentを返す
    if filename.starts_with("api/") {
        return HttpResponse::NoContent().finish(); // 204 No Contentを返す
    }

    let mut filepath = format!("../yukari-ui/{}", filename);
    if fs::metadata("../yukari/build/yukari-ui").is_ok() {
        filepath = format!("../yukari/build/yukari-ui/{}", filename);
    }
    match fs::read(&filepath) {
        Ok(contents) => {
            return HttpResponse::Ok()
                .body(contents);
        }
        Err(_) => {
            // ファイルが見つからない場合は404レスポンスを返す
            return HttpResponse::NotFound().body("File not found");
        }
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let base_port = 50027; // 基本ポート
    let mut port = base_port;

    // ポートが使用中でないことを確認する
    loop {
        // TcpListenerを使用してポートを試す
        let address = format!("127.0.0.1:{}", port);
        match TcpListener::bind(&address) {
            Ok(_) => {
                println!("Starting server on {}", address); // 使用するポートを表示
                break; // ポートがバインドできたのでループを抜ける
            },
            Err(err) => {
                println!("Port {} is occupied: {}", port, err);
                if port == 50050 {
                    break;
                }
                port += 1; // 次のポートに移動
            },
        }
    }

    HttpServer::new(|| {
        App::new()
            .service(set_apikey)
            .service(get_apikey)
            .service(get_version)
            .service(get_file)
    })
    .bind(format!("127.0.0.1:{}", port))? // Bind to 127.0.0.1:8080
    .run()
    .await
}