use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

// ==================== 数据模型 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    pub id: i32,
    pub word: String,
    pub phonetic_en: String,
    pub phonetic_us: String,
    pub definition: String,
    pub example: String,
    pub level: String,
    pub frequency: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub answer: usize,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PassageQuiz {
    pub passage: String,
    pub questions: Vec<PassageQuestionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PassageQuestionItem {
    pub blank_index: usize,
    pub options: Vec<String>,
    pub answer: usize,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_learned: String,
    pub mastery_rate: String,
    pub streak_days: String,
    pub total_time: String,
    pub progress: Vec<ProgressItem>,
    pub level_mastery: Vec<LevelMastery>,
    pub daily_trend: Vec<DailyTrend>,
    pub wrong_words: Vec<WrongWord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressItem {
    pub name: String,
    pub value: i32,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelMastery {
    pub name: String,
    pub value: i32,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyTrend {
    pub date: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WrongWord {
    pub word: String,
    pub count: i32,
}

// ==================== 配置管理 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub api_url: String,
    pub api_key: String,
    pub model_name: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    pub enabled: bool,
    pub search_count: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub model: ModelConfig,
    pub search: SearchConfig,
    pub audio_expire_hours: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                api_url: "https://api.openai.com/v1/chat/completions".into(),
                api_key: "".into(),
                model_name: "gpt-4o".into(),
                max_tokens: 2000,
                temperature: 0.7,
            },
            search: SearchConfig {
                enabled: false,
                search_count: 5,
                timeout_seconds: 15,
            },
            audio_expire_hours: 5,
        }
    }
}

// ==================== 类型别名 ====================

type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

// ==================== SQLite 数据库辅助 ====================

fn get_db_path() -> PathBuf {
    let mut path = dirs::data_dir().expect("无法获取数据目录");
    path.push("vocab-master");
    path.push("vocab.db");
    path
}

fn create_pool() -> Result<DbPool, String> {
    let db_path = get_db_path();
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let manager = r2d2_sqlite::SqliteConnectionManager::file(&db_path)
        .map_err(|e| e.to_string())?;
    r2d2::Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(|e| e.to_string())
}

fn init_tables(pool: &DbPool) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS word_progress (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            word_id INTEGER UNIQUE NOT NULL,
            word TEXT UNIQUE NOT NULL,
            level TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'new' CHECK(status IN ('new', 'learning', 'mastered', 'hard')),
            last_seen TEXT,
            review_count INTEGER DEFAULT 0,
            next_review TEXT,
            correct_count INTEGER DEFAULT 0,
            wrong_count INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_wp_level ON word_progress(level);
        CREATE INDEX IF NOT EXISTS idx_wp_status ON word_progress(status);
        CREATE INDEX IF NOT EXISTS idx_wp_next_review ON word_progress(next_review);

        CREATE TABLE IF NOT EXISTS study_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            new_words INTEGER DEFAULT 0,
            reviewed_words INTEGER DEFAULT 0,
            correct_count INTEGER DEFAULT 0,
            incorrect_count INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_ss_date ON study_sessions(date);
        ",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 从 Tauri 状态或创建新连接池获取数据库
fn get_db(app: &tauri::AppHandle) -> Result<DbPool, String> {
    if let Ok(state) = app.state::<DbPool>().get() {
        return Ok(state.inner().clone());
    }
    let pool = create_pool()?;
    init_tables(&pool)?;
    Ok(pool)
}

// ==================== Tauri 命令 ====================

/// 获取词库目录路径
#[tauri::command]
fn get_words_dir() -> String {
    let mut path = dirs::data_dir().expect("无法获取数据目录");
    path.push("vocab-master");
    path.push("words");
    path.to_string_lossy().to_string()
}

/// 确保词库目录存在
#[tauri::command]
fn ensure_words_dir() -> Result<String, String> {
    let mut path = dirs::data_dir().expect("无法获取数据目录");
    path.push("vocab-master");
    path.push("words");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// 加载指定学段的词库
#[tauri::command]
fn load_words(level: String) -> Result<Vec<Word>, String> {
    let mut path = dirs::data_dir().expect("无法获取数据目录");
    path.push("vocab-master");
    path.push("words");
    path.push(format!("{}.json", level));

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let words: Vec<Word> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(words)
}

/// 播放单词发音（Free Dictionary API）
#[tauri::command]
async fn play_word_audio(word: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        urlencoding::encode(&word)
    );

    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("未找到单词 '{}' 的发音", word));
    }

    let entries: Vec<Value> = resp.json().await.map_err(|e| e.to_string())?;
    if entries.is_empty() {
        return Err(format!("未找到单词 '{}' 的发音", word));
    }

    // 尝试获取英音或美音音频 URL
    let phonetics = &entries[0]["phonetics"];
    if let Some(arr) = phonetics.as_array() {
        for p in arr {
            if let (Some(audio), true) = (p["audio"].as_str(), p["listen"].as_bool().unwrap_or(false)) {
                if !audio.is_empty() {
                    return Ok(audio.to_string());
                }
            }
            if let (Some(_), true) = (
                p["sourceUrl"].as_str(),
                !p["sourceUrl"].as_str().unwrap_or("").is_empty(),
            ) {
                if let Some(url_val) = p["url"].as_str() {
                    if !url_val.is_empty() {
                        return Ok(url_val.to_string());
                    }
                }
            }
        }
    }

    Err(format!("单词 '{}' 没有可用的音频", word))
}

/// 标记单词为已掌握
#[tauri::command]
fn mark_word_learned(app: tauri::AppHandle, word_id: i32) -> Result<(), String> {
    let pool = get_db(&app)?;
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO word_progress (word_id, status, last_seen, review_count, correct_count, next_review)
         VALUES (?, 'mastered', ?, 1, 1, datetime('now', '+30 days'))",
        [word_id, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 标记单词为困难
#[tauri::command]
fn mark_word_hard(app: tauri::AppHandle, word_id: i32) -> Result<(), String> {
    let pool = get_db(&app)?;
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO word_progress (word_id, status, last_seen, review_count, wrong_count, next_review)
         VALUES (?, 'hard', ?, 1, 1, datetime('now', '+1 day'))",
        [word_id, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 记录学习进度到 SQLite
#[tauri::command]
fn save_progress(app: tauri::AppHandle, data: Value) -> Result<(), String> {
    let pool = get_db(&app)?;
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT INTO study_sessions (date, new_words, reviewed_words, correct_count, incorrect_count)
         VALUES (?, ?, ?, ?, ?)",
        [
            &today,
            data.get("new_words").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("reviewed_words").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("correct_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("incorrect_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        ],
    )
    .map_err(|e| e.to_string())?;

    // 保存旧版 JSON 兼容格式（用于向后兼容）
    let mut path = dirs::data_dir().expect("无法获取数据目录");
    path.push("vocab-master");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("progress.json");
    let json_str = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(path, json_str).map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取仪表盘数据（从 SQLite + JSON 混合读取）
#[tauri::command]
fn get_dashboard_data(app: tauri::AppHandle) -> Result<DashboardData, String> {
    match get_db(&app) {
        Ok(pool) => get_dashboard_from_sqlite(&pool),
        Err(_) => get_dashboard_from_json(),
    }
}

fn get_dashboard_from_sqlite(pool: &DbPool) -> Result<DashboardData, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    // 总学习数
    let total_learned: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress WHERE status != 'new'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 掌握率
    let mastered: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress WHERE status = 'mastered'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let mastery_rate = if total_learned > 0 {
        (mastered as f64 / total_learned as f64 * 100.0) as f32
    } else {
        0.0
    };

    // 连续学习天数
    let streak_days = compute_streak(&conn)?;

    // 总学习时长（基于会话记录估算）
    let total_time: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(new_words + reviewed_words), 0) FROM study_sessions",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_time_str = if total_time > 0 {
        format!("{}h", total_time / 10)
    } else {
        "0h".to_string()
    };

    // 各学段掌握度
    let level_rows: Vec<(String, i32)> = conn
        .query_map(
            "SELECT level, COUNT(*) FROM word_progress GROUP BY level ORDER BY level",
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 每日趋势
    let daily_rows: Vec<(String, i32)> = conn
        .query_map(
            "SELECT date, (new_words + reviewed_words) as cnt FROM study_sessions ORDER BY date DESC LIMIT 30",
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 错词 Top 10
    let wrong_rows: Vec<(String, i32)> = conn
        .query_map(
            "SELECT word, wrong_count FROM word_progress WHERE wrong_count > 0 ORDER BY wrong_count DESC LIMIT 10",
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 构建响应
    let progress = vec![
        ProgressItem {
            name: "已掌握".into(),
            value: mastered as i32,
            color: "#10b981".into(),
        },
        ProgressItem {
            name: "学习中".into(),
            value: (total_learned - mastered) as i32,
            color: "#3b82f6".into(),
        },
        ProgressItem {
            name: "未学习".into(),
            value: 100 - total_learned as i32,
            color: "#e2e8f0".into(),
        },
    ];

    let level_colors = HashMap::from([
        ("primary", "#a78bfa"),
        ("junior", "#60a5fa"),
        ("high", "#34d399"),
        ("cet4", "#fbbf24"),
        ("cet6", "#f87171"),
    ]);
    let level_mastery: Vec<LevelMastery> = level_rows
        .iter()
        .map(|(name, value)| LevelMastery {
            name: name.clone(),
            value: *value as i32,
            color: level_colors
                .get(name.as_str())
                .map(|c| c.to_string())
                .unwrap_or("#6366f1".into()),
        })
        .collect();

    let daily_trend: Vec<DailyTrend> = daily_rows
        .iter()
        .rev()
        .map(|(date, count)| DailyTrend {
            date: date.clone(),
            count: *count as i32,
        })
        .collect();

    let wrong_words: Vec<WrongWord> = wrong_rows
        .iter()
        .map(|(word, count)| WrongWord {
            word: word.clone(),
            count: *count as i32,
        })
        .collect();

    Ok(DashboardData {
        total_learned: total_learned.to_string(),
        mastery_rate: format!("{:.0}", mastery_rate),
        streak_days: format!("{}天", streak_days),
        total_time: total_time_str,
        progress,
        level_mastery,
        daily_trend,
        wrong_words,
    })
}

fn compute_streak(conn: &r2d2::Connection) -> Result<i64, String> {
    let dates: Vec<String> = conn
        .query_map(
            "SELECT DISTINCT date FROM study_sessions ORDER BY date DESC",
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if dates.is_empty() {
        return Ok(0);
    }

    let mut streak = 0i64;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut expected = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d")
        .ok()
        .unwrap_or_else(|| chrono::NaiveDate::from_timestamp_opt(0, 0).unwrap());

    for date_str in dates {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            while d < expected {
                expected = expected.pred_opt().unwrap_or(expected);
                streak += 1;
                if d == expected {
                    streak += 1;
                    expected = expected.pred_opt().unwrap_or(expected);
                    break;
                }
            }
            if d == expected {
                streak += 1;
                expected = expected.pred_opt().unwrap_or(expected);
            }
        }
    }

    Ok(streak)
}

fn get_dashboard_from_json() -> Result<DashboardData, String> {
    let mut path = dirs::data_dir().expect("无法获取数据目录");
    path.push("vocab-master");
    path.push("progress.json");

    let (total_learned, mastery_rate, streak_days, total_time, level_data, daily_data, wrong_data) =
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            (
                json.get("total_learned")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                json.get("mastery_rate")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32,
                json.get("streak_days")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                json.get("total_time")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0h")
                    .to_string(),
                json.get("level_mastery")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
                json.get("daily_trend")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
                json.get("wrong_words")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
            )
        } else {
            (
                0,
                0.0,
                0,
                "0h".to_string(),
                Value::Array(vec![]),
                Value::Array(vec![]),
                Value::Array(vec![]),
            )
        };

    let progress = vec![
        ProgressItem {
            name: "已掌握".into(),
            value: (total_learned as f32 * mastery_rate / 100.0) as i32,
            color: "#10b981".into(),
        },
        ProgressItem {
            name: "学习中".into(),
            value: (total_learned as f32 * (1.0 - mastery_rate / 100.0)) as i32,
            color: "#3b82f6".into(),
        },
        ProgressItem {
            name: "未学习".into(),
            value: 100 - total_learned,
            color: "#e2e8f0".into(),
        },
    ];

    let level_mastery = if level_data.as_array().map(|a| a.len()).unwrap_or(0) > 0 {
        level_data
            .as_array()
            .unwrap()
            .iter()
            .map(|v| LevelMastery {
                name: v
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                value: v
                    .get("value")
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32,
                color: v
                    .get("color")
                    .and_then(|n| n.as_str())
                    .unwrap_or("#6366f1")
                    .to_string(),
            })
            .collect()
    } else {
        vec![
            LevelMastery {
                name: "小学".into(),
                value: 0,
                color: "#a78bfa".into(),
            },
            LevelMastery {
                name: "初中".into(),
                value: 0,
                color: "#60a5fa".into(),
            },
            LevelMastery {
                name: "高中".into(),
                value: 0,
                color: "#34d399".into(),
            },
            LevelMastery {
                name: "四级".into(),
                value: 0,
                color: "#fbbf24".into(),
            },
            LevelMastery {
                name: "六级".into(),
                value: 0,
                color: "#f87171".into(),
            },
        ]
    };

    let daily_trend = if daily_data.as_array().map(|a| a.len()).unwrap_or(0) > 0 {
        daily_data
            .as_array()
            .unwrap()
            .iter()
            .map(|v| DailyTrend {
                date: v
                    .get("date")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                count: v
                    .get("count")
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32,
            })
            .collect()
    } else {
        vec![DailyTrend {
            date: "暂无数据".into(),
            count: 0,
        }]
    };

    let wrong_words = if wrong_data.as_array().map(|a| a.len()).unwrap_or(0) > 0 {
        wrong_data
            .as_array()
            .unwrap()
            .iter()
            .map(|v| WrongWord {
                word: v
                    .get("word")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                count: v
                    .get("count")
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32,
            })
            .collect()
    } else {
        vec![]
    };

    Ok(DashboardData {
        total_learned: total_learned.to_string(),
        mastery_rate: format!("{:.0}", mastery_rate),
        streak_days: format!("{}天", streak_days),
        total_time,
        progress,
        level_mastery,
        daily_trend,
        wrong_words,
    })
}

/// 获取配置
#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    let mut path = dirs::config_dir().expect("无法获取配置目录");
    path.push("vocab-master");
    path.push("config.toml");

    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    toml::from_str(&content).map_err(|e| e.to_string())
}

/// 保存配置
#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    let mut path = dirs::config_dir().expect("无法获取配置目录");
    path.push("vocab-master");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("config.toml");

    let toml = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, toml).map_err(|e| e.to_string())?;
    Ok(())
}

/// 调用模型 API（OpenAI 兼容格式）
#[tauri::command]
async fn call_model_api(config: AppConfig, messages: Vec<Value>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let body = json!({
        "model": config.model.model_name,
        "messages": messages,
        "max_tokens": config.model.max_tokens,
        "temperature": config.model.temperature,
    });

    let resp = client
        .post(&config.model.api_url)
        .header("Authorization", format!("Bearer {}", config.model.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("模型返回为空")?
        .trim()
        .to_string();

    Ok(content)
}

/// 生成单词选择题（AI）
#[tauri::command]
async fn generate_word_quiz(
    config: AppConfig,
    word: String,
    definition: String,
    level: String,
) -> Result<QuizQuestion, String> {
    let prompt = format!(
        "请为单词 '{}' 生成一道英语选择题。要求：\n\
        - 题目是中文释义，选项是英文单词\n\
        - 提供4个选项（A/B/C/D），其中只有一个是正确的 '{}'\n\
        - 附带简短解释\n\
        - 严格返回 JSON 格式，不要其他内容\n\
        格式：{{\"question\":\"题目\",\"options\":[\"A选项\",\"B选项\",\"C选项\",\"D选项\"],\"answer\":正确选项索引(0-3),\"explanation\":\"解释\"}}",
        word, word
    );

    let messages = vec![
        json!({"role": "system", "content": "你是一个专业的英语出题助手。只返回 JSON，不返回任何其他内容。"}),
        json!({"role": "user", "content": prompt}),
    ];

    let content = call_model_api(config, messages).await?;

    // 尝试从 AI 回复中提取 JSON
    let quiz = extract_json_from_response(&content);
    serde_json::from_value(quiz).map_err(|e| format!("解析题目失败: {}", e))
}

/// 生成完形填空练习（AI）
#[tauri::command]
async fn generate_passage_quiz(
    config: AppConfig,
    words: Vec<String>,
) -> Result<PassageQuiz, String> {
    let word_list = words.join(", ");
    let prompt = format!(
        "请使用以下英语单词写一段简短的英文故事（约100词），并将其中5个单词替换为 ___1___, ___2___ 等占位符。\n\
        单词列表：{}\n\n\
        然后为每个空提供4个选项（其中一个是原单词），并给出正确答案和解释。\n\
        严格返回 JSON 格式：\n\
        {{\"passage\":\"故事文本（用 ___1___ 等占位符）\", \"questions\":[\n\
        {{\"blank_index\":1, \"options\":[\"选项A\",\"选项B\",\"选项C\",\"选项D\"], \"answer\":正确索引, \"explanation\":\"解释\"}},\n\
        ...更多题目...\n\
        ]}}",
        word_list
    );

    let messages = vec![
        json!({"role": "system", "content": "你是一个英语完形填空出题助手。只返回 JSON，不返回任何其他内容。"}),
        json!({"role": "user", "content": prompt}),
    ];

    let content = call_model_api(config, messages).await?;
    let quiz = extract_json_from_response(&content);
    serde_json::from_value(quiz).map_err(|e| format!("解析完形填空失败: {}", e))
}

/// 从 AI 回复中提取 JSON 对象
fn extract_json_from_response(response: &str) -> Value {
    // 尝试直接解析
    if let Ok(v) = serde_json::from_str::<Value>(response) {
        return v;
    }
    // 尝试在代码块中提取
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                if let Ok(v) = serde_json::from_str(&response[start..=end]) {
                    return v;
                }
            }
        }
    }
    // 返回空对象
    json!({})
}

/// 联网搜索
#[tauri::command]
async fn web_search(config: AppConfig, query: String) -> Result<Vec<SearchResult>, String> {
    if !config.search.enabled {
        return Ok(vec![]);
    }

    let client = reqwest::Client::new();

    // Try Bing first
    let bing_url = format!(
        "https://www.bing.com/search?q={}",
        urlencoding::encode(&format!("{} definition examples etymology", query))
    );

    let results = match client
        .get(&bing_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
        )
        .timeout(std::time::Duration::from_secs(config.search.timeout_seconds))
        .send()
        .await
    {
        Ok(resp) => {
            let html = resp.text().await.map_err(|e| e.to_string())?;
            parse_bing_html(&html)
        }
        Err(_) => {
            // Fallback to DuckDuckGo
            let ddg_url = format!(
                "https://html.duckduckgo.com/html/?q={}",
                urlencoding::encode(&format!("{} definition examples", query))
            );
            let resp = client
                .get(&ddg_url)
                .header(
                    "User-Agent",
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
                )
                .timeout(std::time::Duration::from_secs(config.search.timeout_seconds))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let html = resp.text().await.map_err(|e| e.to_string())?;
            parse_duckduckgo_html(&html)
        }
    };

    Ok(results.into_iter().take(config.search.search_count).collect())
}

fn parse_bing_html(html: &str) -> Vec<SearchResult> {
    let document = scraper::Html::parse_document(html);
    let li_selector = scraper::Selector::parse("li.b_algo").unwrap();
    let title_selector = scraper::Selector::parse("h2 a").unwrap();
    let snippet_selector = scraper::Selector::parse(".b_caption p, .b_algoSlug").unwrap();

    let mut results = Vec::new();
    for item in document.select(&li_selector) {
        let title = item
            .select(&title_selector)
            .next()
            .map(|a| a.text().collect::<String>())
            .unwrap_or_default();

        let url = item
            .select(&title_selector)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("")
            .to_string();

        let snippet = item
            .select(&snippet_selector)
            .next()
            .map(|p| p.text().collect::<String>())
            .unwrap_or_default();

        if !title.is_empty() {
            results.push(SearchResult { title, url, snippet });
        }
    }
    results
}

fn parse_duckduckgo_html(html: &str) -> Vec<SearchResult> {
    let document = scraper::Html::parse_document(html);
    let result_selector = scraper::Selector::parse(".result").unwrap();
    let title_selector = scraper::Selector::parse(".result__title a").unwrap();
    let snippet_selector = scraper::Selector::parse(".result__snippet").unwrap();

    let mut results = Vec::new();
    for item in document.select(&result_selector) {
        let title = item
            .select(&title_selector)
            .next()
            .map(|a| a.text().collect::<String>())
            .unwrap_or_default();

        let url = item
            .select(&title_selector)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("")
            .to_string();

        let snippet = item
            .select(&snippet_selector)
            .next()
            .map(|p| p.text().collect::<String>())
            .unwrap_or_default();

        if !title.is_empty() {
            results.push(SearchResult { title, url, snippet });
        }
    }
    results
}

/// 清理过期音频文件
#[tauri::command]
fn cleanup_audio_cache(expire_hours: u64) -> Result<(), String> {
    let mut path = dirs::cache_dir().ok_or("无法获取缓存目录")?;
    path.push("vocab-master-audio");

    if !path.exists() {
        return Ok(());
    }

    let expire_secs = expire_hours * 3600;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(modified_secs) = modified.duration_since(std::time::UNIX_EPOCH) {
                            if now.saturating_sub(modified_secs.as_secs()) > expire_secs {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// ==================== 应用入口 ====================

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化数据库连接池并注册为共享状态
            let pool = create_pool().map_err(|e| e.to_string())?;
            init_tables(&pool).map_err(|e| e.to_string())?;
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_words_dir,
            ensure_words_dir,
            load_words,
            play_word_audio,
            mark_word_learned,
            mark_word_hard,
            get_dashboard_data,
            save_progress,
            get_config,
            save_config,
            call_model_api,
            generate_word_quiz,
            generate_passage_quiz,
            web_search,
            cleanup_audio_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
