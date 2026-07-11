use base64::{engine::general_purpose, Engine as _};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::Manager;

// ==================== 数据模型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Word {
    pub id: i32,
    pub word: String,
    pub phonetic_en: String,
    pub phonetic_us: String,
    pub definition: String,
    pub example: String,
    #[serde(default)]
    pub example_translation: String,
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
    pub active_level: String,
    pub total_words: String,
    pub total_learned: String,
    pub unlearned_words: String,
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

#[derive(Debug, Serialize)]
pub struct StudyQueueItem {
    pub word: Word,
    pub kind: String,
}

#[derive(Debug, Clone)]
struct ExamplePatch {
    example: String,
    translation: String,
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
    #[serde(default = "default_record_retention_days")]
    pub record_retention_days: u32,
    #[serde(default = "default_daily_new_words")]
    pub daily_new_words: u32,
    #[serde(default = "default_daily_review_words")]
    pub daily_review_words: u32,
    #[serde(default = "default_card_advance_mode")]
    pub card_advance_mode: String,
    #[serde(default = "default_card_detail_seconds")]
    pub card_detail_seconds: u32,
    #[serde(default = "default_active_level")]
    pub active_level: String,
    #[serde(default)]
    pub setup_complete: bool,
    #[serde(default)]
    pub prompts: HashMap<String, String>,
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
            record_retention_days: default_record_retention_days(),
            daily_new_words: default_daily_new_words(),
            daily_review_words: default_daily_review_words(),
            card_advance_mode: default_card_advance_mode(),
            card_detail_seconds: default_card_detail_seconds(),
            active_level: default_active_level(),
            setup_complete: false,
            prompts: HashMap::new(),
        }
    }
}

fn default_active_level() -> String {
    "junior".into()
}

fn default_daily_new_words() -> u32 {
    20
}

fn default_record_retention_days() -> u32 {
    7
}

fn default_daily_review_words() -> u32 {
    30
}

fn default_card_advance_mode() -> String {
    "auto".into()
}

fn default_card_detail_seconds() -> u32 {
    2
}

// ==================== 类型别名 ====================

type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

// ==================== SQLite 数据库辅助 ====================

fn install_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn app_data_dir() -> PathBuf {
    install_dir().join("vocab-master-data")
}

fn app_data_file(name: &str) -> PathBuf {
    app_data_dir().join(name)
}

fn config_path() -> PathBuf {
    app_data_file("config.toml")
}

fn progress_path() -> PathBuf {
    app_data_file("progress.json")
}

fn next_plan_path() -> PathBuf {
    app_data_file("next-plan.json")
}

fn audio_cache_dir() -> PathBuf {
    app_data_dir().join("audio")
}

fn logs_dir() -> PathBuf {
    app_data_dir().join("logs")
}

fn today_log_path() -> PathBuf {
    logs_dir().join(format!("app-{}.log", local_date()))
}

fn word_bank_enrichment_status_path(level: &str) -> Result<PathBuf, String> {
    validate_level(level)?;
    Ok(app_data_dir().join(format!("word-enrichment-{}.json", level)))
}

fn local_now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn local_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn local_iso_days_from_now(days: i64) -> String {
    (chrono::Local::now() + chrono::Duration::days(days))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}

fn local_iso_days_ago(days: i64) -> String {
    (chrono::Local::now() - chrono::Duration::days(days))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}

fn append_app_log(scope: &str, message: impl AsRef<str>) {
    let _ = cleanup_app_logs();
    let path = today_log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(
            file,
            "{} [{}] {}",
            local_now_iso(),
            scope,
            message.as_ref().replace('\n', "\\n")
        );
    }
}

fn cleanup_app_logs() -> Result<(), String> {
    let path = logs_dir();
    if !path.exists() {
        return Ok(());
    }
    let expire_secs = 24 * 3600;
    let now = std::time::SystemTime::now();
    for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if !metadata.is_file() {
            continue;
        }
        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(_) => continue,
        };
        if now
            .duration_since(modified)
            .map(|duration| duration.as_secs() > expire_secs)
            .unwrap_or(false)
        {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

fn get_db_path() -> PathBuf {
    app_data_file("vocab.db")
}

fn create_pool() -> Result<DbPool, String> {
    let db_path = get_db_path();
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let manager = r2d2_sqlite::SqliteConnectionManager::file(&db_path);
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
            fuzzy_count INTEGER DEFAULT 0,
            correct_count INTEGER DEFAULT 0,
            incorrect_count INTEGER DEFAULT 0,
            duration_seconds INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_ss_date ON study_sessions(date);

        CREATE TABLE IF NOT EXISTS quiz_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            level TEXT NOT NULL,
            score INTEGER DEFAULT 0,
            summary TEXT,
            advice TEXT,
            payload TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_qr_created_at ON quiz_records(created_at);
        CREATE INDEX IF NOT EXISTS idx_qr_level ON quiz_records(level);

        CREATE TABLE IF NOT EXISTS daily_plan_status (
            date TEXT NOT NULL,
            level TEXT NOT NULL,
            completed_at TEXT NOT NULL,
            PRIMARY KEY(date, level)
        );
        ",
    )
    .map_err(|e| e.to_string())?;
    let _ = conn.execute(
        "ALTER TABLE study_sessions ADD COLUMN duration_seconds INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE study_sessions ADD COLUMN fuzzy_count INTEGER DEFAULT 0",
        [],
    );
    Ok(())
}

/// 从 Tauri 状态或创建新连接池获取数据库
fn get_db(app: &tauri::AppHandle) -> Result<DbPool, String> {
    if let Some(state) = app.try_state::<DbPool>() {
        return Ok(state.inner().clone());
    }
    let pool = create_pool()?;
    init_tables(&pool)?;
    Ok(pool)
}

// ==================== Tauri 命令 ====================

const PRESET_WORD_BANKS: &[(&str, &str)] = &[
    ("junior", include_str!("../../data/words/junior.json")),
    ("high", include_str!("../../data/words/high.json")),
    ("cet4", include_str!("../../data/words/cet4.json")),
    ("cet6", include_str!("../../data/words/cet6.json")),
];

fn words_dir_path() -> PathBuf {
    app_data_dir().join("words")
}

fn validate_level(level: &str) -> Result<(), String> {
    if level
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err("学段名称只能包含字母、数字、下划线或连字符".into())
    }
}

fn word_bank_path(level: &str) -> Result<PathBuf, String> {
    validate_level(level)?;
    let mut path = words_dir_path();
    path.push(format!("{}.json", level));
    Ok(path)
}

fn preset_words_content(level: &str) -> Option<&'static str> {
    PRESET_WORD_BANKS
        .iter()
        .find(|(key, _)| *key == level)
        .map(|(_, content)| *content)
}

fn word_from_value(value: Value, index: usize, fallback_level: &str) -> Result<Word, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| format!("第 {} 条词库数据不是对象", index + 1))?;
    let word = obj
        .get("word")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("第 {} 条词库数据缺少 word 字段", index + 1))?;

    let definition = match obj.get("definition") {
        Some(Value::String(v)) => v.clone(),
        Some(v) => serde_json::to_string(v).map_err(|e| e.to_string())?,
        None => obj
            .get("meaning")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };

    Ok(Word {
        id: obj
            .get("id")
            .and_then(|v| v.as_i64())
            .unwrap_or((index + 1) as i64) as i32,
        word: word.to_string(),
        phonetic_en: obj
            .get("phonetic_en")
            .or_else(|| obj.get("phonetic"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        phonetic_us: obj
            .get("phonetic_us")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        definition,
        example: obj
            .get("example")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        example_translation: obj
            .get("example_translation")
            .or_else(|| obj.get("example_zh"))
            .or_else(|| obj.get("translation"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        level: obj
            .get("level")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(fallback_level)
            .to_string(),
        frequency: obj
            .get("frequency")
            .and_then(|v| v.as_i64())
            .unwrap_or((index + 1) as i64) as i32,
    })
}

fn parse_word_bank(level: &str, content: &str) -> Result<Vec<Word>, String> {
    let raw_words: Vec<Value> = serde_json::from_str(content).map_err(|e| e.to_string())?;
    raw_words
        .into_iter()
        .enumerate()
        .map(|(idx, value)| word_from_value(value, idx, level))
        .collect()
}

fn save_word_bank(level: &str, content: &str) -> Result<usize, String> {
    let words = parse_word_bank(level, content)?;
    fs::create_dir_all(words_dir_path()).map_err(|e| e.to_string())?;
    let path = word_bank_path(level)?;
    let json = serde_json::to_string_pretty(&words).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(words.len())
}

fn write_word_bank(level: &str, words: &[Word]) -> Result<(), String> {
    fs::create_dir_all(words_dir_path()).map_err(|e| e.to_string())?;
    let path = word_bank_path(level)?;
    let json = serde_json::to_string_pretty(words).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// 获取词库目录路径
#[tauri::command]
fn get_words_dir() -> String {
    let path = words_dir_path();
    path.to_string_lossy().to_string()
}

/// 确保词库目录存在
#[tauri::command]
fn ensure_words_dir() -> Result<String, String> {
    let path = words_dir_path();
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_logs_dir() -> Result<String, String> {
    let path = logs_dir();
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    cleanup_app_logs()?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn clear_logs() -> Result<(), String> {
    let path = logs_dir();
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

/// 加载指定学段的词库
#[tauri::command]
fn load_words(level: String) -> Result<Vec<Word>, String> {
    let path = word_bank_path(&level)?;

    if !path.exists() {
        if let Some(content) = preset_words_content(&level) {
            save_word_bank(&level, content)?;
        } else {
            return Ok(Vec::new());
        }
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    parse_word_bank(&level, &content)
}

#[tauri::command]
fn get_study_words_between(
    app: tauri::AppHandle,
    level: String,
    start_iso: String,
    end_iso: String,
) -> Result<Vec<Word>, String> {
    validate_level(&level)?;
    let words = load_words(level.clone())?;
    let words_by_id: HashMap<i32, Word> = words.into_iter().map(|word| (word.id, word)).collect();
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT word_id
             FROM word_progress
             WHERE level = ?1
               AND last_seen IS NOT NULL
               AND datetime(last_seen) >= datetime(?2)
               AND datetime(last_seen) < datetime(?3)
             ORDER BY datetime(last_seen) DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![level, start_iso, end_iso], |row| {
            row.get::<_, i32>(0)
        })
        .map_err(|e| e.to_string())?;

    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for row in rows {
        let word_id = row.map_err(|e| e.to_string())?;
        if seen.insert(word_id) {
            if let Some(word) = words_by_id.get(&word_id) {
                result.push(word.clone());
            }
        }
    }

    Ok(result)
}

/// 导入或覆盖指定学段的词库
#[tauri::command]
fn import_words(level: String, content: String) -> Result<usize, String> {
    save_word_bank(&level, &content)
}

#[tauri::command]
async fn auto_enrich_word_bank_examples(config: AppConfig, level: String) -> Result<Value, String> {
    validate_level(&level)?;
    if !has_usable_model_key(&config) {
        append_app_log(
            "word-enrich",
            format!("skip level={} reason=no-api-key", level),
        );
        return Ok(json!({
            "status": "skipped",
            "message": "未配置可用 API Key"
        }));
    }

    let mut words = load_words(level.clone())?;
    let candidates: Vec<(usize, Word)> = words
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, word)| example_needs_enrichment(word))
        .collect();
    append_app_log(
        "word-enrich",
        format!("start level={} candidates={}", level, candidates.len()),
    );

    if candidates.is_empty() {
        write_word_bank_enrichment_status(&level, "completed", 0, 0, "当前学段例句已补全")?;
        append_app_log(
            "word-enrich",
            format!("completed level={} enriched=0", level),
        );
        return Ok(json!({
            "status": "completed",
            "enriched": 0,
            "total": 0
        }));
    }

    write_word_bank_enrichment_status(&level, "running", 0, candidates.len(), "正在后台补充例句")?;

    let mut enriched = 0usize;
    for chunk in candidates.chunks(12) {
        let generated = enrich_example_chunk(&config, chunk).await?;
        for (index, word) in chunk {
            let key = word.word.to_ascii_lowercase();
            if let Some(patch) = generated.get(&key) {
                if !patch.example.trim().is_empty() {
                    words[*index].example = patch.example.trim().to_string();
                    words[*index].example_translation = patch.translation.trim().to_string();
                    enriched += 1;
                }
            }
        }
        write_word_bank(&level, &words)?;
        write_word_bank_enrichment_status(
            &level,
            "running",
            enriched,
            candidates.len(),
            "正在后台补充例句",
        )?;
    }

    write_word_bank_enrichment_status(
        &level,
        "completed",
        enriched,
        candidates.len(),
        "例句补充完成",
    )?;
    append_app_log(
        "word-enrich",
        format!(
            "completed level={} enriched={}/{}",
            level,
            enriched,
            candidates.len()
        ),
    );
    Ok(json!({
        "status": "completed",
        "enriched": enriched,
        "total": candidates.len()
    }))
}

async fn enrich_example_chunk(
    config: &AppConfig,
    chunk: &[(usize, Word)],
) -> Result<HashMap<String, ExamplePatch>, String> {
    let words_payload: Vec<Value> = chunk
        .iter()
        .map(|(_, word)| {
            json!({
                "word": word.word,
                "definition": word.definition,
                "current_example": word.example,
                "current_translation": word.example_translation,
                "level": word.level
            })
        })
        .collect();
    let prompt = serde_json::to_string(&json!({
        "instruction": "请为每个单词补充或改写一个自然、简短、适合中国英语学习者的英文例句，并给出准确中文翻译。英文例句必须包含目标词原形或合理变形；不要改变单词本身；中文翻译不要解释语法，只翻译句意。",
        "schema": {
            "items": [
                {
                    "word": "abandon",
                    "example": "He decided to abandon the old plan.",
                    "translation": "他决定放弃这个旧计划。"
                }
            ]
        },
        "words": words_payload
    }))
    .map_err(|e| e.to_string())?;

    let content = call_model_api(
        config.clone(),
        vec![
            json!({
                "role": "system",
                "content": "你是英语词库编辑。只返回 JSON，不要返回 Markdown。"
            }),
            json!({
                "role": "user",
                "content": prompt
            }),
        ],
    )
    .await?;

    let parsed = extract_json_from_response(&content);
    let items = parsed
        .get("items")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut result = HashMap::new();
    for item in items {
        let word = item
            .get("word")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let example = item
            .get("example")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let translation = item
            .get("translation")
            .or_else(|| item.get("example_translation"))
            .or_else(|| item.get("example_zh"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if !word.is_empty() && !example.is_empty() {
            result.insert(
                word,
                ExamplePatch {
                    example,
                    translation,
                },
            );
        }
    }
    Ok(result)
}

fn example_needs_enrichment(word: &Word) -> bool {
    let example = word.example.trim();
    if example.is_empty() {
        return true;
    }
    if word.example_translation.trim().is_empty() {
        return true;
    }
    let has_sentence_punctuation =
        example.contains('.') || example.contains('!') || example.contains('?');
    let has_space = example.split_whitespace().count() >= 4;
    !has_sentence_punctuation || !has_space
}

fn has_usable_model_key(config: &AppConfig) -> bool {
    let key = config.model.api_key.trim();
    !key.is_empty()
        && !matches!(
            key.to_ascii_lowercase().as_str(),
            "api_key" | "your_api_key" | "your-api-key" | "sk-xxx" | "sk-xxxx"
        )
}

fn write_word_bank_enrichment_status(
    level: &str,
    status: &str,
    enriched: usize,
    total: usize,
    message: &str,
) -> Result<(), String> {
    let path = word_bank_enrichment_status_path(level)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(&json!({
        "level": level,
        "status": status,
        "enriched": enriched,
        "total": total,
        "message": message,
        "updated_at": local_now_iso()
    }))
    .map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

/// 获取今日学习队列：到期复习词 + 新词
#[tauri::command]
fn get_study_queue(
    app: tauri::AppHandle,
    level: String,
    new_count: u32,
    review_count: u32,
) -> Result<Vec<StudyQueueItem>, String> {
    let mut words = load_words(level.clone())?;
    words.sort_by_key(|word| (word.frequency, word.id));
    let word_map: HashMap<i32, Word> = words.iter().cloned().map(|word| (word.id, word)).collect();
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut queue = Vec::new();
    let mut used_ids = HashSet::new();
    let now = local_now_iso();

    if review_count > 0 {
        let mut stmt = conn
            .prepare(
                "SELECT word_id FROM word_progress
                 WHERE level = ?1
                   AND status != 'new'
                   AND (next_review IS NULL OR next_review <= ?2 OR status = 'hard')
                 ORDER BY CASE status WHEN 'hard' THEN 0 WHEN 'learning' THEN 1 ELSE 2 END ASC,
                          COALESCE(next_review, '1970-01-01 00:00:00') ASC,
                          wrong_count DESC,
                          last_seen ASC
                 LIMIT ?3",
            )
            .map_err(|e| e.to_string())?;
        let review_ids: Vec<i32> = stmt
            .query_map(params![level.as_str(), now, review_count as i64], |row| {
                row.get(0)
            })
            .map_err(|e| e.to_string())?
            .filter_map(|row| row.ok())
            .collect();

        for id in review_ids {
            if let Some(word) = word_map.get(&id) {
                used_ids.insert(id);
                queue.push(StudyQueueItem {
                    word: word.clone(),
                    kind: "review".into(),
                });
            }
        }
    }

    let learned_ids: HashSet<i32> = conn
        .prepare("SELECT word_id FROM word_progress WHERE level = ?1")
        .map_err(|e| e.to_string())?
        .query_map(params![level.as_str()], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|row| row.ok())
        .collect();

    for word in words
        .into_iter()
        .filter(|word| !learned_ids.contains(&word.id) && !used_ids.contains(&word.id))
        .take(new_count as usize)
    {
        queue.push(StudyQueueItem {
            word,
            kind: "new".into(),
        });
    }

    Ok(queue)
}

/// 播放单词发音（Free Dictionary API）
#[tauri::command]
async fn play_word_audio(word: String) -> Result<String, String> {
    cache_word_audio(&word).await
}

/// 标记单词为已掌握
async fn cache_word_audio(word: &str) -> Result<String, String> {
    let _ = cleanup_audio_cache(1);
    let client = reqwest::Client::new();
    append_app_log("audio", format!("lookup word={}", word));
    let audio_url = lookup_audio_url(&client, word).await?;

    let resp = client
        .get(&audio_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| {
            append_app_log(
                "audio",
                format!("download failed word={} url={} err={}", word, audio_url, e),
            );
            format!("发音下载失败：{}", e)
        })?;

    if !resp.status().is_success() {
        append_app_log(
            "audio",
            format!("download http word={} status={}", word, resp.status()),
        );
        return Err(format!("发音下载失败：HTTP {}", resp.status()));
    }

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().unwrap_or(value).trim().to_string())
        .filter(|value| value.starts_with("audio/"));
    let bytes = resp.bytes().await.map_err(|e| {
        append_app_log("audio", format!("read failed word={} err={}", word, e));
        format!("发音读取失败：{}", e)
    })?;
    if bytes.is_empty() {
        append_app_log("audio", format!("empty audio word={}", word));
        return Err("发音文件为空".into());
    }

    let mime = content_type.unwrap_or_else(|| guess_audio_mime(&audio_url));
    append_app_log(
        "audio",
        format!("ready word={} mime={} bytes={}", word, mime, bytes.len()),
    );
    Ok(format!(
        "data:{};base64,{}",
        mime,
        general_purpose::STANDARD.encode(bytes.as_ref())
    ))
}

async fn lookup_audio_url(client: &reqwest::Client, word: &str) -> Result<String, String> {
    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        urlencoding::encode(word)
    );

    let resp = match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return Ok(fallback_tts_url(word)),
    };

    if !resp.status().is_success() {
        return Ok(fallback_tts_url(word));
    }

    let entries: Vec<Value> = resp.json().await.map_err(|e| e.to_string())?;
    if entries.is_empty() {
        return Ok(fallback_tts_url(word));
    }

    let phonetics = &entries[0]["phonetics"];
    let mut candidates = Vec::new();
    if let Some(arr) = phonetics.as_array() {
        for p in arr {
            if let Some(audio) = p["audio"].as_str() {
                if !audio.is_empty() {
                    candidates.push(normalize_audio_url(audio));
                }
            }
        }
    }

    if let Some(url) = candidates.iter().find(|url| is_preferred_audio_url(url)) {
        return Ok(url.clone());
    }

    Ok(fallback_tts_url(word))
}

fn normalize_audio_url(url: &str) -> String {
    if url.starts_with("//") {
        format!("https:{}", url)
    } else {
        url.to_string()
    }
}

fn is_preferred_audio_url(url: &str) -> bool {
    let path = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    path.ends_with(".mp3")
        || path.ends_with(".wav")
        || path.ends_with(".m4a")
        || path.ends_with(".mp4")
}

fn fallback_tts_url(word: &str) -> String {
    format!(
        "https://dict.youdao.com/dictvoice?type=2&audio={}",
        urlencoding::encode(word)
    )
}

fn audio_cache_file_path(word: &str, audio_url: &str) -> Result<PathBuf, String> {
    let mut path = audio_cache_dir();

    let file_name = safe_audio_file_name(word);
    let extension = audio_url
        .split('?')
        .next()
        .and_then(|path| path.rsplit('.').next())
        .filter(|ext| {
            !ext.is_empty() && ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_alphanumeric())
        })
        .unwrap_or("mp3");
    path.push(format!(
        "{}-{}.{}",
        file_name,
        chrono::Utc::now().timestamp_millis(),
        extension
    ));
    Ok(path)
}

fn guess_audio_mime(audio_url: &str) -> String {
    let path = audio_url.split('?').next().unwrap_or(audio_url);
    match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "ogg" | "oga" => "audio/ogg".into(),
        "wav" => "audio/wav".into(),
        "m4a" | "mp4" => "audio/mp4".into(),
        "webm" => "audio/webm".into(),
        _ => "audio/mpeg".into(),
    }
}

fn safe_audio_file_name(word: &str) -> String {
    let sanitized: String = word
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "word".into()
    } else {
        sanitized
    }
}

fn progress_counts(conn: &Connection, word_id: i32) -> (i32, i32) {
    conn.query_row(
        "SELECT review_count, wrong_count FROM word_progress WHERE word_id = ?1",
        params![word_id],
        |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?)),
    )
    .unwrap_or((0, 0))
}

fn next_review_for_remembered(review_count: i32, wrong_count: i32) -> String {
    let intervals: &[i64] = if wrong_count > 0 {
        &[1, 2, 4, 7, 15, 30]
    } else {
        &[1, 3, 7, 15, 30, 60]
    };
    let index = review_count.saturating_sub(1) as usize;
    let days = intervals
        .get(index)
        .copied()
        .unwrap_or_else(|| *intervals.last().unwrap_or(&30));
    local_iso_days_from_now(days)
}

fn next_review_for_fuzzy(review_count: i32, wrong_count: i32) -> String {
    let days = if wrong_count > 0 || review_count <= 1 {
        1
    } else {
        2
    };
    local_iso_days_from_now(days)
}

#[tauri::command]
fn mark_word_learned(app: tauri::AppHandle, word: Word) -> Result<(), String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let word_id = word.id;
    let (review_count, wrong_count) = progress_counts(&conn, word_id);
    let next_review = next_review_for_remembered(review_count + 1, wrong_count);
    let now = local_now_iso();
    let word_text = word.word;
    let level = word.level;
    conn.execute(
        "INSERT INTO word_progress
            (word_id, word, level, status, last_seen, review_count, correct_count, wrong_count, next_review, created_at)
         VALUES (?1, ?2, ?3, 'mastered', ?4, 1, 1, 0, ?5, ?4)
         ON CONFLICT(word_id) DO UPDATE SET
            word = excluded.word,
            level = excluded.level,
            status = 'mastered',
            last_seen = excluded.last_seen,
            review_count = word_progress.review_count + 1,
            correct_count = word_progress.correct_count + 1,
            next_review = excluded.next_review",
        params![
            word_id,
            word_text.as_str(),
            level.as_str(),
            now.as_str(),
            next_review.as_str()
        ],
    )
    .map_err(|e| e.to_string())?;
    append_app_log(
        "study",
        format!("remembered word_id={} level={}", word_id, level),
    );
    Ok(())
}

#[tauri::command]
fn mark_word_fuzzy(app: tauri::AppHandle, word: Word) -> Result<(), String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let word_id = word.id;
    let (review_count, wrong_count) = progress_counts(&conn, word_id);
    let next_review = next_review_for_fuzzy(review_count + 1, wrong_count);
    let now = local_now_iso();
    let word_text = word.word;
    let level = word.level;
    conn.execute(
        "INSERT INTO word_progress
            (word_id, word, level, status, last_seen, review_count, correct_count, wrong_count, next_review, created_at)
         VALUES (?1, ?2, ?3, 'learning', ?4, 1, 0, 0, ?5, ?4)
         ON CONFLICT(word_id) DO UPDATE SET
            word = excluded.word,
            level = excluded.level,
            status = 'learning',
            last_seen = excluded.last_seen,
            review_count = word_progress.review_count + 1,
            next_review = excluded.next_review",
        params![
            word_id,
            word_text.as_str(),
            level.as_str(),
            now.as_str(),
            next_review.as_str()
        ],
    )
    .map_err(|e| e.to_string())?;
    append_app_log(
        "study",
        format!("fuzzy word_id={} level={}", word_id, level),
    );
    Ok(())
}

/// 标记单词为困难
#[tauri::command]
fn mark_word_hard(app: tauri::AppHandle, word: Word) -> Result<(), String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let now = local_now_iso();
    let next_review = local_iso_days_from_now(1);
    let word_id = word.id;
    let word_text = word.word;
    let level = word.level;
    conn.execute(
        "INSERT INTO word_progress
            (word_id, word, level, status, last_seen, review_count, correct_count, wrong_count, next_review, created_at)
         VALUES (?1, ?2, ?3, 'hard', ?4, 1, 0, 1, ?5, ?4)
         ON CONFLICT(word_id) DO UPDATE SET
            word = excluded.word,
            level = excluded.level,
            status = 'hard',
            last_seen = excluded.last_seen,
            review_count = word_progress.review_count + 1,
            wrong_count = word_progress.wrong_count + 1,
            next_review = excluded.next_review",
        params![
            word_id,
            word_text.as_str(),
            level.as_str(),
            now.as_str(),
            next_review.as_str()
        ],
    )
    .map_err(|e| e.to_string())?;
    append_app_log("study", format!("hard word_id={} level={}", word_id, level));
    Ok(())
}

/// 记录学习进度到 SQLite
#[tauri::command]
fn save_progress(app: tauri::AppHandle, data: Value) -> Result<(), String> {
    let pool = get_db(&app)?;
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let today = local_date();
    let created_at = local_now_iso();

    conn.execute(
        "INSERT INTO study_sessions (date, new_words, reviewed_words, fuzzy_count, correct_count, incorrect_count, duration_seconds, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            today,
            data.get("new_words").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("reviewed_words").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("fuzzy_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("correct_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("incorrect_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            data.get("duration_seconds")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            created_at,
        ],
    )
    .map_err(|e| e.to_string())?;

    // 保存旧版 JSON 兼容格式（用于向后兼容）
    let path = progress_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json_str = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(path, json_str).map_err(|e| e.to_string())?;
    Ok(())
}

/// 保存完整测验记录
#[tauri::command]
fn save_quiz_record(
    app: tauri::AppHandle,
    record: Value,
    retention_days: u32,
) -> Result<(), String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let today = local_date();
    let created_at = local_now_iso();
    let level = record
        .get("level")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let null_result = Value::Null;
    let result = record.get("result").unwrap_or(&null_result);
    let score = result
        .get("score")
        .or_else(|| record.get("score"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let summary = result
        .get("summary")
        .or_else(|| record.get("summary"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let advice = result
        .get("advice")
        .cloned()
        .unwrap_or_else(|| Value::Array(vec![]));
    let advice_json = serde_json::to_string(&advice).map_err(|e| e.to_string())?;
    let payload = serde_json::to_string(&record).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO quiz_records (date, level, score, summary, advice, payload, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            today,
            level,
            score,
            summary,
            advice_json,
            payload,
            created_at
        ],
    )
    .map_err(|e| e.to_string())?;

    cleanup_quiz_records(&conn, retention_days)?;
    Ok(())
}

#[tauri::command]
fn get_quiz_records(app: tauri::AppHandle) -> Result<Vec<Value>, String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, date, level, score, summary, payload, created_at
             FROM quiz_records
             ORDER BY datetime(created_at) DESC, id DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let date: String = row.get(1)?;
            let level: String = row.get(2)?;
            let score: i32 = row.get(3)?;
            let summary: String = row.get(4)?;
            let payload: String = row.get(5)?;
            let created_at: String = row.get(6)?;
            Ok((id, date, level, score, summary, payload, created_at))
        })
        .map_err(|e| e.to_string())?;

    let mut records = Vec::new();
    for row in rows {
        let (id, date, level, score, summary, payload, created_at) =
            row.map_err(|e| e.to_string())?;
        let mut record: Value = serde_json::from_str(&payload).unwrap_or_else(|_| json!({}));
        if let Some(obj) = record.as_object_mut() {
            obj.insert("id".into(), json!(id));
            obj.insert("date".into(), json!(date));
            obj.insert("level".into(), json!(level));
            obj.insert("score".into(), json!(score));
            obj.insert("summary".into(), json!(summary));
            obj.insert("created_at".into(), json!(created_at));
        }
        records.push(record);
    }

    Ok(records)
}

#[tauri::command]
fn get_wrong_book(app: tauri::AppHandle) -> Result<Vec<Value>, String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT word, level, wrong_count, COALESCE(last_seen, '')
             FROM word_progress
             WHERE wrong_count > 0
             ORDER BY wrong_count DESC, datetime(last_seen) DESC
             LIMIT 100",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (word, level, wrong_count, last_seen) = row.map_err(|e| e.to_string())?;
        items.push(json!({
            "source": "word",
            "word": word,
            "level": level,
            "wrong_count": wrong_count,
            "created_at": last_seen,
            "answer": "",
            "analysis": "词卡学习中标记为不记得。",
            "suggestion": "优先安排到复习队列，直到连续答对后再降低复习频率。"
        }));
    }

    let mut stmt = conn
        .prepare(
            "SELECT payload, created_at
             FROM quiz_records
             ORDER BY datetime(created_at) DESC, id DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (payload, created_at) = row.map_err(|e| e.to_string())?;
        let record: Value = serde_json::from_str(&payload).unwrap_or_else(|_| json!({}));
        let level = record
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        if let Some(result_items) = record
            .get("result")
            .and_then(|result| result.get("items"))
            .and_then(|items| items.as_array())
        {
            for item in result_items.iter().filter(|item| {
                !item
                    .get("is_correct")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            }) {
                let word = item.get("word").and_then(|v| v.as_str()).unwrap_or("");
                if word.is_empty() {
                    continue;
                }
                items.push(json!({
                    "source": "quiz",
                    "word": word,
                    "level": level,
                    "wrong_count": 1,
                    "created_at": created_at,
                    "answer": item.get("answer").cloned().unwrap_or(Value::String(String::new())),
                    "analysis": item.get("analysis").cloned().unwrap_or(Value::String(String::new())),
                    "suggestion": item.get("suggestion").cloned().unwrap_or(Value::String(String::new()))
                }));
            }
        }
    }

    Ok(items)
}

fn cleanup_quiz_records(conn: &Connection, retention_days: u32) -> Result<(), String> {
    let days = retention_days.clamp(1, 3650);
    let threshold = local_iso_days_ago(days as i64);
    conn.execute(
        "DELETE FROM quiz_records WHERE created_at < ?1",
        params![threshold],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_learning_records(app: tauri::AppHandle) -> Result<Value, String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut session_stmt = conn
        .prepare(
            "SELECT id, date, new_words, reviewed_words, fuzzy_count, correct_count, incorrect_count, duration_seconds, created_at
             FROM study_sessions
             ORDER BY datetime(created_at) DESC, id DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;
    let sessions = session_stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "date": row.get::<_, String>(1)?,
                "new_words": row.get::<_, i32>(2)?,
                "reviewed_words": row.get::<_, i32>(3)?,
                "fuzzy_count": row.get::<_, i32>(4)?,
                "correct_count": row.get::<_, i32>(5)?,
                "incorrect_count": row.get::<_, i32>(6)?,
                "duration_seconds": row.get::<_, i32>(7)?,
                "created_at": row.get::<_, String>(8)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut quiz_stmt = conn
        .prepare(
            "SELECT id, date, level, score, COALESCE(summary, ''), created_at
             FROM quiz_records
             ORDER BY datetime(created_at) DESC, id DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;
    let quizzes = quiz_stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "date": row.get::<_, String>(1)?,
                "level": row.get::<_, String>(2)?,
                "score": row.get::<_, i32>(3)?,
                "summary": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let progress_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM word_progress", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(json!({
        "sessions": sessions,
        "quizzes": quizzes,
        "progress_count": progress_count,
    }))
}

#[tauri::command]
fn delete_learning_record(
    app: tauri::AppHandle,
    record_type: String,
    id: i64,
) -> Result<(), String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let sql = match record_type.as_str() {
        "session" => "DELETE FROM study_sessions WHERE id = ?1",
        "quiz" => "DELETE FROM quiz_records WHERE id = ?1",
        _ => return Err("未知记录类型".into()),
    };
    conn.execute(sql, params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_learning_records(app: tauri::AppHandle, scope: String) -> Result<(), String> {
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    match scope.as_str() {
        "all" => {
            conn.execute("DELETE FROM word_progress", [])
                .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM study_sessions", [])
                .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM quiz_records", [])
                .map_err(|e| e.to_string())?;
            remove_legacy_progress_file();
        }
        "progress" => {
            conn.execute("DELETE FROM word_progress", [])
                .map_err(|e| e.to_string())?;
            remove_legacy_progress_file();
        }
        "sessions" => {
            conn.execute("DELETE FROM study_sessions", [])
                .map_err(|e| e.to_string())?;
            remove_legacy_progress_file();
        }
        "quizzes" => {
            conn.execute("DELETE FROM quiz_records", [])
                .map_err(|e| e.to_string())?;
        }
        _ => return Err("未知清理范围".into()),
    }

    Ok(())
}

fn remove_legacy_progress_file() {
    let _ = fs::remove_file(progress_path());
}

#[tauri::command]
fn mark_daily_plan_complete(
    app: tauri::AppHandle,
    level: String,
    date: String,
) -> Result<(), String> {
    validate_level(&level)?;
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let completed_at = local_now_iso();
    conn.execute(
        "INSERT INTO daily_plan_status (date, level, completed_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(date, level) DO UPDATE SET completed_at = excluded.completed_at",
        params![date, level, completed_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn is_daily_plan_complete(
    app: tauri::AppHandle,
    level: String,
    date: String,
) -> Result<bool, String> {
    validate_level(&level)?;
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM daily_plan_status WHERE date = ?1 AND level = ?2",
            params![date, level],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(count > 0)
}

#[tauri::command]
fn get_review_forecast(app: tauri::AppHandle, level: String) -> Result<Value, String> {
    validate_level(&level)?;
    let pool = get_db(&app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let now = local_now_iso();
    let tomorrow = local_iso_days_from_now(1);

    let due_now: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress
             WHERE level = ?1 AND status != 'new' AND next_review IS NOT NULL AND next_review <= ?2",
            params![level.as_str(), now],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let due_tomorrow: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress
             WHERE level = ?1 AND status != 'new' AND next_review IS NOT NULL AND next_review <= ?2",
            params![level.as_str(), tomorrow],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let hard_words: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress WHERE level = ?1 AND status = 'hard'",
            params![level.as_str()],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let fuzzy_words: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress WHERE level = ?1 AND status = 'learning'",
            params![level.as_str()],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(json!({
        "due_now": due_now,
        "due_tomorrow": due_tomorrow,
        "hard_words": hard_words,
        "fuzzy_words": fuzzy_words,
        "curve": "1/3/7/15/30/60 days, slowed for previously wrong words"
    }))
}

#[tauri::command]
fn get_next_plan() -> Result<Value, String> {
    let path = next_plan_path();
    if !path.exists() {
        return Ok(Value::Null);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_next_plan(record: Value) -> Result<(), String> {
    let path = next_plan_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_dashboard_data(app: tauri::AppHandle) -> Result<DashboardData, String> {
    let config = get_config().unwrap_or_default();
    let active_level = config.active_level;
    let total_words = load_words(active_level.clone())
        .map(|words| words.len() as i32)
        .unwrap_or(0);

    match get_db(&app) {
        Ok(pool) => get_dashboard_from_sqlite(&pool, &active_level, total_words),
        Err(_) => get_dashboard_from_json(&active_level, total_words),
    }
}

fn get_dashboard_from_sqlite(
    pool: &DbPool,
    active_level: &str,
    total_words: i32,
) -> Result<DashboardData, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    // 总学习数
    let total_learned: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress WHERE level = ?1 AND status != 'new'",
            params![active_level],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 掌握率
    let mastered: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM word_progress WHERE level = ?1 AND status = 'mastered'",
            params![active_level],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let unlearned_words = total_words.saturating_sub(total_learned);
    let mastery_rate = if total_words > 0 {
        (mastered as f64 / total_words as f64 * 100.0) as f32
    } else if total_learned > 0 {
        (mastered as f64 / total_learned as f64 * 100.0) as f32
    } else {
        0.0
    };

    // 连续学习天数
    let streak_days = compute_streak(&*conn)?;

    // 总学习时长（基于真实会话耗时）
    let total_seconds: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_seconds), 0) FROM study_sessions",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_time_str = format_duration(total_seconds);

    // 各学段掌握度
    let mut stmt = conn
        .prepare("SELECT level, COUNT(*) FROM word_progress GROUP BY level ORDER BY level")
        .map_err(|e| e.to_string())?;
    let level_rows: Vec<(String, i32)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 每日趋势
    let mut stmt = conn
        .prepare("SELECT date, (new_words + reviewed_words) as cnt FROM study_sessions ORDER BY date DESC LIMIT 30")
        .map_err(|e| e.to_string())?;
    let daily_rows: Vec<(String, i32)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 错词 Top 10
    let mut stmt = conn
        .prepare("SELECT word, wrong_count FROM word_progress WHERE level = ?1 AND wrong_count > 0 ORDER BY wrong_count DESC LIMIT 10")
        .map_err(|e| e.to_string())?;
    let wrong_rows: Vec<(String, i32)> = stmt
        .query_map(params![active_level], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
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
            value: unlearned_words,
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
        active_level: active_level.to_string(),
        total_words: total_words.to_string(),
        total_learned: total_learned.to_string(),
        unlearned_words: unlearned_words.to_string(),
        mastery_rate: format!("{:.0}", mastery_rate),
        streak_days: format!("{}天", streak_days),
        total_time: total_time_str,
        progress,
        level_mastery,
        daily_trend,
        wrong_words,
    })
}

fn compute_streak(conn: &Connection) -> Result<i64, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT date FROM study_sessions ORDER BY date DESC")
        .map_err(|e| e.to_string())?;
    let dates: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if dates.is_empty() {
        return Ok(0);
    }

    let mut streak = 0i64;
    let today = local_date();
    let mut expected = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d")
        .ok()
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

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

fn format_duration(total_seconds: i64) -> String {
    if total_seconds <= 0 {
        return "0分钟".into();
    }

    let minutes = ((total_seconds as f64) / 60.0).ceil() as i64;
    if minutes < 60 {
        format!("{}分钟", minutes)
    } else {
        let hours = minutes / 60;
        let rest = minutes % 60;
        if rest == 0 {
            format!("{}小时", hours)
        } else {
            format!("{}小时{}分钟", hours, rest)
        }
    }
}

fn get_dashboard_from_json(active_level: &str, total_words: i32) -> Result<DashboardData, String> {
    let path = progress_path();

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

    let mastered = (total_learned as f32 * mastery_rate / 100.0) as i32;
    let unlearned_words = total_words.saturating_sub(total_learned);

    let progress = vec![
        ProgressItem {
            name: "已掌握".into(),
            value: mastered,
            color: "#10b981".into(),
        },
        ProgressItem {
            name: "学习中".into(),
            value: (total_learned as f32 * (1.0 - mastery_rate / 100.0)) as i32,
            color: "#3b82f6".into(),
        },
        ProgressItem {
            name: "未学习".into(),
            value: unlearned_words,
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
                value: v.get("value").and_then(|n| n.as_i64()).unwrap_or(0) as i32,
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
                count: v.get("count").and_then(|n| n.as_i64()).unwrap_or(0) as i32,
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
                count: v.get("count").and_then(|n| n.as_i64()).unwrap_or(0) as i32,
            })
            .collect()
    } else {
        vec![]
    };

    Ok(DashboardData {
        active_level: active_level.to_string(),
        total_words: total_words.to_string(),
        total_learned: total_learned.to_string(),
        unlearned_words: unlearned_words.to_string(),
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
    let path = config_path();

    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    toml::from_str(&content).map_err(|e| e.to_string())
}

/// 保存配置
#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let toml = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, toml).map_err(|e| e.to_string())?;
    Ok(())
}

/// 调用模型 API（OpenAI 兼容格式）
#[tauri::command]
async fn call_model_api(config: AppConfig, messages: Vec<Value>) -> Result<String, String> {
    let client = reqwest::Client::new();
    append_app_log(
        "model",
        format!(
            "request model={} url={} messages={}",
            config.model.model_name,
            config.model.api_url,
            messages.len()
        ),
    );
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
        .map_err(|e| {
            append_app_log("model", format!("request failed err={}", e));
            e.to_string()
        })?;

    let status = resp.status();
    if !status.is_success() {
        append_app_log("model", format!("http status={}", status));
    }
    let json: Value = resp.json().await.map_err(|e| {
        append_app_log("model", format!("json parse failed err={}", e));
        e.to_string()
    })?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            append_app_log("model", "empty response");
            "模型返回为空"
        })?
        .trim()
        .to_string();

    append_app_log(
        "model",
        format!("response chars={}", content.chars().count()),
    );
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

    Ok(results
        .into_iter()
        .take(config.search.search_count)
        .collect())
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
            results.push(SearchResult {
                title,
                url,
                snippet,
            });
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
            results.push(SearchResult {
                title,
                url,
                snippet,
            });
        }
    }
    results
}

/// 清理过期音频文件
#[tauri::command]
fn cleanup_audio_cache(expire_hours: u64) -> Result<(), String> {
    let path = audio_cache_dir();

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
            let _ = cleanup_app_logs();
            append_app_log("app", "startup");
            // 初始化数据库连接池并注册为共享状态
            let pool = create_pool().map_err(|e| e.to_string())?;
            init_tables(&pool).map_err(|e| e.to_string())?;
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_words_dir,
            ensure_words_dir,
            get_logs_dir,
            clear_logs,
            load_words,
            get_study_words_between,
            import_words,
            auto_enrich_word_bank_examples,
            get_study_queue,
            play_word_audio,
            mark_word_learned,
            mark_word_fuzzy,
            mark_word_hard,
            get_dashboard_data,
            save_progress,
            save_quiz_record,
            get_quiz_records,
            get_wrong_book,
            get_learning_records,
            delete_learning_record,
            clear_learning_records,
            mark_daily_plan_complete,
            is_daily_plan_complete,
            get_review_forecast,
            get_next_plan,
            save_next_plan,
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
