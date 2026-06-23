use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Author {
    name: String,
    email: String,
    count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitCommit {
    hash: String,
    author_name: String,
    author_email: String,
    date: DateTime<Utc>,
    message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct DateRange {
    start: Option<String>,
    end: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExportRecord {
    pub timestamp: String,
    pub project_path: String,
    pub filename: String,
    pub format: String,
    pub author_count: usize,
    pub commit_count: usize,
    pub incremental: bool,
    #[serde(default)]
    pub file_path: String,
}

#[tauri::command]
fn get_git_authors(project_path: &str) -> Result<Vec<Author>, String> {
    use git2::Repository;

    let repo =
        Repository::open(project_path).map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut walk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    walk.push_head()
        .map_err(|e| format!("Failed to push HEAD: {}", e))?;

    let mut author_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for oid_result in walk {
        let oid = oid_result.map_err(|e| format!("Revwalk error: {}", e))?;

        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let author = commit.author();
        let name = author.name().unwrap_or("Unknown").to_string();
        let email = author.email().unwrap_or("").to_string();

        let key = format!("{} <{}>", name, email);
        *author_counts
            .entry(key)
            .or_insert_with(|| HashMap::new())
            .entry(email)
            .or_insert(0) += 1;
    }

    let mut authors: Vec<Author> = Vec::new();
    for (display_name, email_count_map) in author_counts {
        for (email, count) in email_count_map {
            let name = display_name
                .split(" <")
                .next()
                .unwrap_or(&display_name)
                .to_string();
            authors.push(Author { name, email, count });
        }
    }

    Ok(authors)
}

#[tauri::command]
fn export_to_txt(content: &str, suggested_filename: &str) -> Result<String, String> {
    use std::fs::File;
    use std::io::Write;
    
    // 获取用户桌面目录
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let desktop_dir = home_dir.join("Desktop");
    let file_path = desktop_dir.join(suggested_filename);
    
    let mut file = File::create(&file_path).map_err(|e| format!("无法创建文件: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_directory(dir_path: &str) -> Result<(), String> {
    use std::process::Command;
    let path = PathBuf::from(dir_path);
    if !path.exists() {
        return Err(format!("目录不存在: {}", dir_path));
    }
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(dir_path).status();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(dir_path).status();
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").arg("/C").arg("start").arg(dir_path).status();
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("打开目录失败: {}", e)),
    }
}

#[tauri::command]
fn get_export_history() -> Result<Vec<ExportRecord>, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let history_file = home_dir.join(".git-author-analyzer-history.json");
    if !history_file.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&history_file).map_err(|e| format!("读取历史记录失败: {}", e))?;
    let records: Vec<ExportRecord> = serde_json::from_str(&content).map_err(|e| format!("解析历史记录失败: {}", e))?;
    Ok(records)
}

#[tauri::command]
fn save_export_record(record: ExportRecord) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let history_file = home_dir.join(".git-author-analyzer-history.json");
    
    let mut records: Vec<ExportRecord> = if history_file.exists() {
        let content = fs::read_to_string(&history_file).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    records.insert(0, record);
    if records.len() > 100 {
        records.truncate(100);
    }
    
    let json = serde_json::to_string_pretty(&records).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&history_file, json).map_err(|e| format!("写入历史记录失败: {}", e))?;
    Ok(())
}

#[tauri::command]
fn clear_export_history() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let history_file = home_dir.join(".git-author-analyzer-history.json");
    if history_file.exists() {
        fs::remove_file(&history_file).map_err(|e| format!("删除历史记录失败: {}", e))?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExportConfig {
    pub project_path: String,
    pub selected_author_uids: Vec<String>,
    pub date_mode: String,
    pub single_date: String,
    pub date_ranges: Vec<DateRange>,
    pub export_format: String,
    pub export_template: String,
    pub incremental_export: bool,
    pub export_dir: String,
}

#[tauri::command]
fn save_export_config(config: ExportConfig) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let config_file = home_dir.join(".git-author-analyzer-config.json");
    let json = serde_json::to_string_pretty(&config).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&config_file, json).map_err(|e| format!("写入配置失败: {}", e))?;
    Ok(())
}

#[tauri::command]
fn load_export_config() -> Result<Option<ExportConfig>, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let config_file = home_dir.join(".git-author-analyzer-config.json");
    if !config_file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&config_file).map_err(|e| format!("读取配置失败: {}", e))?;
    let config: ExportConfig =
        serde_json::from_str(&content).map_err(|e| format!("解析配置失败: {}", e))?;
    Ok(Some(config))
}

#[tauri::command]
fn write_export_file(path: &str, content: &str) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create(path).map_err(|e| format!("无法创建文件: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(())
}
#[tauri::command]
fn get_git_commits_for_incremental(
    project_path: &str,
    incremental_since: Option<String>,
    author_emails: Vec<String>,
) -> Result<Vec<GitCommit>, String> {
    use chrono::{LocalResult, TimeZone};
    use git2::Repository;

    let repo =
        Repository::open(project_path).map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut walk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    walk.push_head()
        .map_err(|e| format!("Failed to push HEAD: {}", e))?;

    let since_ts = if let Some(since_str) = incremental_since {
        let parts: Vec<&str> = since_str.split('T').collect();
        if parts.len() != 2 {
            return Err("无效的增量时间格式".to_string());
        }
        let date_part = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
            .map_err(|_| "无效日期格式")?;
        let time_part = parts[1].trim_start_matches('Z');
        let time_parts: Vec<&str> = time_part.split(':').collect();
        if time_parts.len() < 2 {
            return Err("无效时间格式".to_string());
        }
        let hour: u32 = time_parts[0].parse().map_err(|_| "无效小时")?;
        let min: u32 = time_parts[1].parse().map_err(|_| "无效分钟")?;
        let sec = if time_parts.len() > 2 {
            time_parts[2].parse().map_err(|_| "无效秒")?
        } else {
            0
        };
        let dt = date_part.and_hms_opt(hour, min, sec)
            .and_then(|d| d.and_utc().into())
            .ok_or("无效日期时间")?;
        Some(dt)
    } else {
        None
    };

    let mut commits = Vec::new();

    for oid_result in walk {
        let oid = oid_result.map_err(|e| format!("Revwalk error: {}", e))?;

        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let author = commit.author();
        let author_email = author.email().unwrap_or("").to_string();

        // 过滤 merge commits（两个或以上父提交）
        if commit.parent_count() > 1 {
            continue;
        }

        if !author_emails.is_empty() && !author_emails.contains(&author_email) {
            continue;
        }

        // 使用 author date（而非 committer date）
        let author_time = author.when();
        let commit_timestamp = author_time.seconds();
        let commit_datetime = match Utc.timestamp_opt(commit_timestamp, 0) {
            LocalResult::Single(dt) => dt,
            _ => continue,
        };

        if let Some(since) = since_ts {
            if commit_datetime <= since {
                continue;
            }
        }

        let short_hash: String = oid.to_string().chars().take(7).collect();

        commits.push(GitCommit {
            hash: short_hash,
            author_name: author.name().unwrap_or("Unknown").to_string(),
            author_email,
            date: commit_datetime,
            message: commit.message().unwrap_or("").lines().next().unwrap_or("").to_string(),
        });
    }

    Ok(commits)
}

#[tauri::command]
fn get_git_commits_in_date_range(
    project_path: &str,
    date_ranges: Vec<DateRange>,
    author_emails: Vec<String>,
) -> Result<Vec<GitCommit>, String> {
    use chrono::{LocalResult, TimeZone};
    use git2::Repository;

    let repo =
        Repository::open(project_path).map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut walk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    walk.push_head()
        .map_err(|e| format!("Failed to push HEAD: {}", e))?;

    let mut commits = Vec::new();

    // Parse date ranges
    let mut parsed_ranges = Vec::new();
    for range in date_ranges {
        let start = match &range.start {
            Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .and_then(|date| {
                    let dt = date.and_hms_opt(0, 0, 0)?;
                    Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                }),
            None => None,
        };

        let end = match &range.end {
            Some(e) => NaiveDate::parse_from_str(e, "%Y-%m-%d")
                .ok()
                .and_then(|date| {
                    let dt = date.and_hms_opt(23, 59, 59)?;
                    Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                }),
            None => None,
        };

        parsed_ranges.push((start, end));
    }

    for oid_result in walk {
        let oid = oid_result.map_err(|e| format!("Revwalk error: {}", e))?;

        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let author = commit.author();
        let author_email = author.email().unwrap_or("").to_string();

        // 过滤 merge commits（两个或以上父提交）
        if commit.parent_count() > 1 {
            continue;
        }

        // Check if author is in the selected list
        if !author_emails.is_empty() && !author_emails.contains(&author_email) {
            continue;
        }

        // Parse commit date (使用 author date)
        let commit_time = author.when();
        let commit_timestamp = commit_time.seconds();
        let commit_datetime = match Utc.timestamp_opt(commit_timestamp, 0) {
            LocalResult::Single(dt) => dt,
            _ => continue,
        };

        // Check if commit date is in any of the specified ranges
        let in_any_range = parsed_ranges.iter().any(|(start, end)| match (start, end) {
            (Some(start_date), Some(end_date)) => {
                commit_datetime >= *start_date && commit_datetime <= *end_date
            }
            (Some(start_date), None) => commit_datetime >= *start_date,
            (None, Some(end_date)) => commit_datetime <= *end_date,
            (None, None) => true,
        });

        if !in_any_range {
            continue;
        }

        let short_hash: String = oid.to_string().chars().take(7).collect();

        commits.push(GitCommit {
            hash: short_hash,
            author_name: author.name().unwrap_or("Unknown").to_string(),
            author_email,
            date: commit_datetime,
            message: commit.message().unwrap_or("").lines().next().unwrap_or("").to_string(),
        });
    }

    Ok(commits)
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_git_authors,
            get_git_commits_in_date_range,
            get_git_commits_for_incremental,
            export_to_txt,
            open_directory,
            get_export_history,
            save_export_record,
            clear_export_history,
            save_export_config,
            load_export_config,
            write_export_file,
            get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
