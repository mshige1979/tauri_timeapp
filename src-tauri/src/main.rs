// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Tauriプロジェクトの標準的なインポート
use tauri_plugin_notification::NotificationExt;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Timelike;  
use serde::{Deserialize, Serialize};

// 通知の状態を管理する構造体
#[derive(Debug, Default)]
struct NotificationState {
    enabled: bool,
}

// 天気情報を格納する構造体
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct WeatherInfo {
    description: String,
    temperature: f64,
    weather_code: String,
    humidity: i32,
    icon: String,
}

// 現在時刻を取得するコマンド
#[tauri::command]
fn get_current_time() -> String {
    use chrono::Local;
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// 通知を送信するコマンド
#[tauri::command]
async fn send_notification(
    app_handle: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    app_handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

// 通知設定を切り替えるコマンド
#[tauri::command]
fn toggle_notification(
    app_state: tauri::State<'_, Arc<Mutex<NotificationState>>>,
    enabled: bool,
) -> Result<(), String> {
    let mut state = app_state.lock().map_err(|e| e.to_string())?;
    state.enabled = enabled;
    Ok(())
}

// 通知の状態を取得するコマンド
#[tauri::command]
fn get_notification_state(
    app_state: tauri::State<'_, Arc<Mutex<NotificationState>>>,
) -> Result<bool, String> {
    let state = app_state.lock().map_err(|e| e.to_string())?;
    Ok(state.enabled)
}

// 気象庁APIから天気情報を取得する共通関数
async fn fetch_weather_info(url: &str) -> Result<WeatherInfo, String> {
    match reqwest::get(url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let weather_data = &data[0]["timeSeries"][0]["areas"][0];
                        let weather_code = weather_data["weatherCodes"][0]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let weather_description = weather_data["weathers"][0]
                            .as_str()
                            .unwrap_or("不明")
                            .to_string();
                        let temp_data = &data[0]["timeSeries"][2]["areas"][0];
                        let temperature = temp_data["temps"][0]
                            .as_str()
                            .unwrap_or("0")
                            .parse::<f64>()
                            .unwrap_or(0.0);
                        let humidity = 50; // デフォルト値
                        let icon = match weather_code.as_str() {
                            "100" | "123" | "124" | "130" | "131" => "01d",
                            "101" | "132" | "140" | "160" | "170" => "02d",
                            "102" | "104" | "115" | "116" | "141" | "142" => "03d",
                            "103" | "106" | "107" | "108" | "128" | "143" | "150" => "04d",
                            "110" | "111" | "112" | "113" | "114" | "118" | "119" | "125"
                            | "126" | "127" | "153" | "154" | "155" | "181" => "09d",
                            "117" | "181" => "11d",
                            "120" | "121" | "122" | "156" | "157" | "160" => "13d",
                            _ => "50d",
                        };

                        Ok(WeatherInfo {
                            description: weather_description,
                            temperature,
                            weather_code,
                            humidity,
                            icon: icon.to_string(),
                        })
                    }
                    Err(e) => Err(format!("JSONパースエラー: {}", e)),
                }
            } else {
                Err(format!("API呼び出しエラー: {}", response.status()))
            }
        }
        Err(e) => Err(format!("ネットワークエラー: {}", e)),
    }
}

// 東京の天気を取得するコマンド
#[tauri::command]
async fn get_tokyo_weather() -> Result<WeatherInfo, String> {
    fetch_weather_info("https://www.jma.go.jp/bosai/forecast/data/forecast/130000.json").await
}

// 福岡の天気を取得するコマンド
#[tauri::command]
async fn get_fukuoka_weather() -> Result<WeatherInfo, String> {
    fetch_weather_info("https://www.jma.go.jp/bosai/forecast/data/forecast/400000.json").await
}

// 気象庁の天気コードを日本語の説明に変換するヘルパー（必要に応じて）
fn get_weather_description_from_code(code: &str) -> &str {
    match code {
        "100" => "晴れ",
        "101" => "晴れ時々曇り",
        "102" => "晴れ一時雨",
        "103" => "晴れ時々雨",
        "104" => "晴れ一時雪",
        "105" => "晴れ時々雪",
        "106" => "晴れ一時雨か雪",
        "107" => "晴れ時々雨か雪",
        "108" => "晴れ一時雨か雷雨",
        "110" => "曇り",
        "111" => "曇り時々晴れ",
        "112" => "曇り一時雨",
        "113" => "曇り時々雨",
        "114" => "曇り一時雪",
        "115" => "曇り時々雪",
        "116" => "曇り一時雨か雪",
        "117" => "曇り時々雨か雪",
        "118" => "曇り一時雨か雷雨",
        "119" => "曇り時々雨か雷雨",
        "120" => "雨",
        "121" => "雨時々晴れ",
        "122" => "雨時々曇り",
        "123" => "雨一時雪",
        "124" => "雨時々雪",
        "125" => "雨一時雪か雷雨",
        "126" => "雨時々雪か雷雨",
        "127" => "雨か雷雨",
        "130" => "雪",
        "131" => "雪時々晴れ",
        "132" => "雪時々曇り",
        "140" => "晴れ",
        "141" => "晴れ時々曇り",
        "142" => "晴れ一時雨",
        "150" => "曇り",
        "160" => "雨",
        "170" => "雪",
        "181" => "雷",
        _ => "不明",
    }
}

// APIキーがない場合のデモ用の天気データを返す関数
#[tauri::command]
async fn get_tokyo_weather_demo() -> Result<WeatherInfo, String> {
    // デモ用のモックデータ
    let mock_weathers = vec![
        WeatherInfo {
            description: "晴れ".to_string(),
            temperature: 22.5,
            weather_code: "100".to_string(),
            humidity: 45,
            icon: "01d".to_string(),
        },
        WeatherInfo {
            description: "曇り".to_string(),
            temperature: 18.3,
            weather_code: "110".to_string(),
            humidity: 65,
            icon: "03d".to_string(),
        },
        WeatherInfo {
            description: "小雨".to_string(),
            temperature: 15.8,
            weather_code: "120".to_string(),
            humidity: 78,
            icon: "10d".to_string(),
        },
    ];

    // 現在の時刻から適当なデータを選択
    use chrono::Local;
    let now = Local::now();
    let index = (now.minute() % 3) as usize;
    
    Ok(mock_weathers[index].clone())
}

// APIキーがない場合のデモ用の福岡天気データを返す関数
#[tauri::command]
async fn get_fukuoka_weather_demo() -> Result<WeatherInfo, String> {
    // デモ用のモックデータ
    let mock_weathers = vec![
        WeatherInfo {
            description: "福岡 - 晴れ".to_string(),
            temperature: 23.5,
            weather_code: "100".to_string(),
            humidity: 42,
            icon: "01d".to_string(),
        },
        WeatherInfo {
            description: "福岡 - 曇り".to_string(),
            temperature: 19.8,
            weather_code: "110".to_string(),
            humidity: 60,
            icon: "03d".to_string(),
        },
        WeatherInfo {
            description: "福岡 - 小雨".to_string(),
            temperature: 17.2,
            weather_code: "120".to_string(),
            humidity: 75,
            icon: "10d".to_string(),
        },
    ];

    // 現在の時刻から適当なデータを選択
    use chrono::Local;
    let now = Local::now();
    let index = (now.minute() % 3) as usize;
    
    Ok(mock_weathers[index].clone())
}

fn main() {
    // 通知状態を管理するための共有状態
    let notification_state = Arc::new(Mutex::new(NotificationState { enabled: false }));
    let notification_state_clone = notification_state.clone();

    tauri::Builder::default()
        // 通知状態を管理する
        .manage(notification_state)
        // プラグインを登録
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        // コマンドを登録
        .invoke_handler(tauri::generate_handler![
            get_current_time,
            send_notification,
            toggle_notification,
            get_notification_state,
            get_tokyo_weather,
            get_tokyo_weather_demo,
            get_fukuoka_weather,
            get_fukuoka_weather_demo
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // 通知を担当するバックグラウンドスレッド
            thread::spawn(move || {
                loop {
                    // 現在時刻を取得
                    let now = chrono::Local::now();
                    
                    // 次の5分刻みの時刻までの待機時間を計算
                    let current_minute = now.minute();
                    let current_second = now.second();
                    
                    // 次の5分刻みまでの分数 (0→5, 1→5, 6→10, など)
                    let minutes_to_next = 5 - (current_minute % 5);
                    
                    // 秒数も考慮した待機時間（ミリ秒）
                    let wait_millis = if minutes_to_next == 5 && current_second == 0 {
                        // 既に5分刻みの時刻である場合は0秒待機
                        0
                    } else {
                        // 次の5分刻みまでの待機時間
                        ((minutes_to_next * 60) - current_second) * 1000
                    };
                    
                    // 待機時間（最低でも1秒は待機）
                    let wait_duration = Duration::from_millis(wait_millis.max(1000) as u64);
                    thread::sleep(wait_duration);
                    
                    // 通知が有効かチェック
                    let is_enabled = {
                        let state = notification_state_clone.lock().unwrap();
                        state.enabled
                    };
                    
                    // 通知が有効な場合は通知を送信
                    if is_enabled {
                        // 現在時刻を再取得（待機後）                        
                        let now = chrono::Local::now();
                        // 分が5の倍数かどうか確認（念のため）
                        if now.minute() % 5 == 0 {
                            let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
                            
                            // 通知を送信
                            let _ = app_handle.notification()
                                .builder()
                                .title("現在時刻（5分刻み）")
                                .body(format!("現在の時刻は {} です", time_str))
                                .show();
                        }
                    }
                    
                    // 少し待機して次のループへ（通知直後に再度通知されるのを防ぐ）
                    thread::sleep(Duration::from_secs(2));
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
