use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration as StdDuration;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, Duration, OffsetDateTime};
use tokio::process::Command;

pub const APP_ID: &str = "dev.funinkina.weekly-commits.cosmic";
pub const CONFIG_DIR: &str = "cosmic-weekly-commits";
pub const CACHE_FILE: &str = "commits-cache-v1.json";
pub const BOX_SIZE: f32 = 14.0;
pub const BOX_MARGIN: f32 = 4.0;
pub const BORDER_RADIUS: f32 = 3.0;
pub const DEFAULT_OPACITY: f32 = 50.0 / 255.0;
pub const OPACITY_PER_COMMIT: f32 = 20.0 / 255.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceType {
    GitHub,
    Gitea,
    GitLab,
}

impl ServiceType {
    pub const ALL: [ServiceType; 3] = [Self::GitHub, Self::Gitea, Self::GitLab];

    pub fn label(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::Gitea => "Gitea",
            Self::GitLab => "GitLab",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorMode {
    Opacity,
    Grade,
}

impl ColorMode {
    pub const ALL: [ColorMode; 2] = [Self::Opacity, Self::Grade];

    pub fn label(self) -> &'static str {
        match self {
            Self::Opacity => "Opacity",
            Self::Grade => "Grade",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WeekdayStart {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl WeekdayStart {
    pub const ALL: [WeekdayStart; 7] = [
        Self::Sunday,
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Sunday => "Sunday",
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
        }
    }

    fn number_from_sunday(self) -> u8 {
        match self {
            Self::Sunday => 0,
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeName {
    Standard,
    Classic,
    GitHubDark,
    Halloween,
    Teal,
    LeftPad,
    Dracula,
    Blue,
    Panda,
    Sunny,
    Pink,
    YlGnBu,
    SolarizedDark,
    SolarizedLight,
}

impl ThemeName {
    pub const ALL: [ThemeName; 14] = [
        Self::Standard,
        Self::Classic,
        Self::GitHubDark,
        Self::Halloween,
        Self::Teal,
        Self::LeftPad,
        Self::Dracula,
        Self::Blue,
        Self::Panda,
        Self::Sunny,
        Self::Pink,
        Self::YlGnBu,
        Self::SolarizedDark,
        Self::SolarizedLight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "GitHub",
            Self::Classic => "GitHub Classic",
            Self::GitHubDark => "GitHub Dark",
            Self::Halloween => "Halloween",
            Self::Teal => "Teal",
            Self::LeftPad => "@left_pad",
            Self::Dracula => "Dracula",
            Self::Blue => "Blue",
            Self::Panda => "Panda",
            Self::Sunny => "Sunny",
            Self::Pink => "Pink",
            Self::YlGnBu => "YlGnBu",
            Self::SolarizedDark => "Solarized Dark",
            Self::SolarizedLight => "Solarized Light",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub grade0: &'static str,
    pub grade1: &'static str,
    pub grade2: &'static str,
    pub grade3: &'static str,
    pub grade4: &'static str,
}

pub fn theme(name: ThemeName) -> Theme {
    match name {
        ThemeName::Standard => Theme {
            grade4: "#216e39",
            grade3: "#30a14e",
            grade2: "#40c463",
            grade1: "#9be9a8",
            grade0: "#ebedf0",
        },
        ThemeName::Classic => Theme {
            grade4: "#196127",
            grade3: "#239a3b",
            grade2: "#7bc96f",
            grade1: "#c6e48b",
            grade0: "#ebedf0",
        },
        ThemeName::GitHubDark => Theme {
            grade4: "#27d545",
            grade3: "#10983d",
            grade2: "#00602d",
            grade1: "#003820",
            grade0: "#161b22",
        },
        ThemeName::Halloween => Theme {
            grade4: "#03001C",
            grade3: "#FE9600",
            grade2: "#FFC501",
            grade1: "#FFEE4A",
            grade0: "#ebedf0",
        },
        ThemeName::Teal => Theme {
            grade4: "#458B74",
            grade3: "#66CDAA",
            grade2: "#76EEC6",
            grade1: "#7FFFD4",
            grade0: "#ebedf0",
        },
        ThemeName::LeftPad => Theme {
            grade4: "#F6F6F6",
            grade3: "#DDDDDD",
            grade2: "#A5A5A5",
            grade1: "#646464",
            grade0: "#2F2F2F",
        },
        ThemeName::Dracula => Theme {
            grade4: "#ff79c6",
            grade3: "#bd93f9",
            grade2: "#6272a4",
            grade1: "#44475a",
            grade0: "#282a36",
        },
        ThemeName::Blue => Theme {
            grade4: "#4F83BF",
            grade3: "#416895",
            grade2: "#344E6C",
            grade1: "#263342",
            grade0: "#222222",
        },
        ThemeName::Panda => Theme {
            grade4: "#FF4B82",
            grade3: "#19f9d8",
            grade2: "#6FC1FF",
            grade1: "#34353B",
            grade0: "#242526",
        },
        ThemeName::Sunny => Theme {
            grade4: "#a98600",
            grade3: "#dab600",
            grade2: "#e9d700",
            grade1: "#f8ed62",
            grade0: "#fff9ae",
        },
        ThemeName::Pink => Theme {
            grade4: "#61185f",
            grade3: "#a74aa8",
            grade2: "#ca5bcc",
            grade1: "#e48bdc",
            grade0: "#ebedf0",
        },
        ThemeName::YlGnBu => Theme {
            grade4: "#253494",
            grade3: "#2c7fb8",
            grade2: "#41b6c4",
            grade1: "#a1dab4",
            grade0: "#ebedf0",
        },
        ThemeName::SolarizedDark => Theme {
            grade4: "#d33682",
            grade3: "#b58900",
            grade2: "#2aa198",
            grade1: "#268bd2",
            grade0: "#073642",
        },
        ThemeName::SolarizedLight => Theme {
            grade4: "#6c71c4",
            grade3: "#dc322f",
            grade2: "#cb4b16",
            grade1: "#b58900",
            grade0: "#eee8d5",
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub service_type: ServiceType,
    pub custom_instance_url: String,
    pub github_username: String,
    pub github_token: String,
    pub refresh_interval: u64,
    pub show_current_week_only: bool,
    pub week_start_day: WeekdayStart,
    pub highlight_current_day: bool,
    pub theme_name: ThemeName,
    pub color_mode: ColorMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            service_type: ServiceType::GitHub,
            custom_instance_url: String::new(),
            github_username: String::new(),
            github_token: String::new(),
            refresh_interval: 21_600,
            show_current_week_only: false,
            week_start_day: WeekdayStart::Monday,
            highlight_current_day: false,
            theme_name: ThemeName::Standard,
            color_mode: ColorMode::Opacity,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub context: CacheContext,
    pub counts: [u32; 7],
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheContext {
    pub service_type: ServiceType,
    pub username: String,
    pub instance_url: String,
    pub show_current_week_only: bool,
    pub week_start_day: WeekdayStart,
}

#[derive(Debug, Clone)]
pub struct Contributions {
    pub dates: [Date; 7],
    pub counts: [u32; 7],
    pub cached_at: Option<String>,
}

#[derive(Debug)]
pub enum FetchError {
    MissingCredentials,
    Http(String),
    Parse(String),
    Io(std::io::Error),
    Timeout,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCredentials => write!(f, "missing credentials"),
            Self::Http(e) => write!(f, "http error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Timeout => write!(f, "request timed out"),
        }
    }
}

impl From<std::io::Error> for FetchError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_DIR)
        .join("config.json")
}

pub fn cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_DIR)
        .join(CACHE_FILE)
}

pub fn load_settings() -> Settings {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) -> std::io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings).unwrap_or_default();
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &path)
}

pub fn dates_for(settings: &Settings) -> [Date; 7] {
    let today = OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .date();

    let start = if settings.show_current_week_only {
        let current = today.weekday().number_days_from_sunday();
        let desired = settings.week_start_day.number_from_sunday();
        let subtract = if current >= desired {
            current - desired
        } else {
            current + 7 - desired
        };
        today - Duration::days(i64::from(subtract))
    } else {
        today - Duration::days(6)
    };

    std::array::from_fn(|i| start + Duration::days(i as i64))
}

pub fn date_key(date: Date) -> String {
    date.format(format_description!("[year]-[month]-[day]"))
        .unwrap_or_default()
}

pub fn date_label(date: Date) -> String {
    let today = OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .date();
    if date == today {
        return "Today".to_string();
    }

    let month = match date.month() {
        time::Month::January => "January",
        time::Month::February => "February",
        time::Month::March => "March",
        time::Month::April => "April",
        time::Month::May => "May",
        time::Month::June => "June",
        time::Month::July => "July",
        time::Month::August => "August",
        time::Month::September => "September",
        time::Month::October => "October",
        time::Month::November => "November",
        time::Month::December => "December",
    };
    format!("{} {}", month, date.day())
}

pub fn cache_key(context: &CacheContext) -> String {
    serde_json::to_string(context).unwrap_or_default()
}

pub fn cache_context(settings: &Settings) -> CacheContext {
    CacheContext {
        service_type: settings.service_type,
        username: settings.github_username.clone(),
        instance_url: settings.custom_instance_url.clone(),
        show_current_week_only: settings.show_current_week_only,
        week_start_day: settings.week_start_day,
    }
}

pub async fn fetch_with_cache(settings: Settings) -> Contributions {
    let dates = dates_for(&settings);
    if settings.github_username.trim().is_empty() || settings.github_token.trim().is_empty() {
        return Contributions {
            dates,
            counts: [0; 7],
            cached_at: None,
        };
    }

    let context = cache_context(&settings);
    let key = cache_key(&context);
    match fetch_live(&settings, dates).await {
        Ok(counts) => {
            if let Err(e) = save_cache(CacheEntry {
                key,
                context,
                counts,
                updated_at: OffsetDateTime::now_utc()
                    .format(&Rfc3339)
                    .unwrap_or_default(),
            }) {
                tracing::warn!("failed to save cache: {e}");
            }
            Contributions {
                dates,
                counts,
                cached_at: None,
            }
        }
        Err(error) => {
            tracing::warn!("live fetch failed, trying cache fallback: {error}");
            if let Some(entry) = load_cache().filter(|entry| entry.key == key) {
                Contributions {
                    dates,
                    counts: entry.counts,
                    cached_at: Some(entry.updated_at),
                }
            } else {
                Contributions {
                    dates,
                    counts: [0; 7],
                    cached_at: None,
                }
            }
        }
    }
}

fn load_cache() -> Option<CacheEntry> {
    std::fs::read_to_string(cache_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn save_cache(entry: CacheEntry) -> std::io::Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&entry).unwrap_or_default();
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &path)
}

async fn fetch_live(settings: &Settings, dates: [Date; 7]) -> Result<[u32; 7], FetchError> {
    match settings.service_type {
        ServiceType::GitHub => fetch_github(settings, dates).await,
        ServiceType::Gitea => fetch_gitea(settings, dates).await,
        ServiceType::GitLab => fetch_gitlab(settings, dates).await,
    }
}

async fn curl(args: Vec<String>, body: Option<String>) -> Result<String, FetchError> {
    let mut command = Command::new("curl");
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if body.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn()?;
    if let Some(body) = body {
        use tokio::io::AsyncWriteExt;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(body.as_bytes()).await?;
        }
    }
    let output = tokio::time::timeout(StdDuration::from_secs(30), child.wait_with_output())
        .await
        .map_err(|_| FetchError::Timeout)?
        .map_err(FetchError::Io)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let msg = if stdout.trim().is_empty() {
            stderr
        } else {
            format!("{stderr}\n{stdout}")
        };
        let msg = msg.trim().to_string();
        return Err(FetchError::Http(msg));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn fetch_github(settings: &Settings, dates: [Date; 7]) -> Result<[u32; 7], FetchError> {
    let username = settings.github_username.trim();
    let token = settings.github_token.trim();
    if username.is_empty() || token.is_empty() {
        return Err(FetchError::MissingCredentials);
    }

    let from = (OffsetDateTime::now_utc() - Duration::days(10))
        .format(&Rfc3339)
        .unwrap_or_default();
    let to = (OffsetDateTime::now_utc() + Duration::days(3))
        .format(&Rfc3339)
        .unwrap_or_default();
    let query = format!(
        r#"query {{
            user(login: "{username}") {{
                contributionsCollection(from: "{from}", to: "{to}") {{
                    contributionCalendar {{
                        weeks {{
                            contributionDays {{
                                date
                                contributionCount
                            }}
                        }}
                    }}
                }}
            }}
        }}"#
    );

    let body = serde_json::json!({ "query": query }).to_string();
    let response = curl(
        vec![
            "-fsS".into(),
            "-X".into(),
            "POST".into(),
            "https://api.github.com/graphql".into(),
            "-H".into(),
            format!("Authorization: bearer {token}"),
            "-H".into(),
            "Content-Type: application/json".into(),
            "-H".into(),
            "User-Agent: COSMIC Weekly Commits".into(),
            "--data-binary".into(),
            "@-".into(),
        ],
        Some(body),
    )
    .await?;

    let result: Value =
        serde_json::from_str(&response).map_err(|e| FetchError::Parse(e.to_string()))?;
    if let Some(errors) = result.get("errors") {
        return Err(FetchError::Http(errors.to_string()));
    }
    let days = result
        .pointer("/data/user/contributionsCollection/contributionCalendar/weeks")
        .and_then(Value::as_array)
        .ok_or_else(|| FetchError::Parse("unexpected GitHub response".into()))?;

    let mut map = BTreeMap::new();
    for week in days {
        if let Some(contribution_days) = week.get("contributionDays").and_then(Value::as_array) {
            for day in contribution_days {
                if let (Some(date), Some(count)) = (
                    day.get("date").and_then(Value::as_str),
                    day.get("contributionCount").and_then(Value::as_u64),
                ) {
                    map.insert(date.to_string(), count as u32);
                }
            }
        }
    }

    Ok(counts_from_map(dates, &map))
}

async fn fetch_gitea(settings: &Settings, dates: [Date; 7]) -> Result<[u32; 7], FetchError> {
    let username = settings.github_username.trim();
    let token = settings.github_token.trim();
    let base = settings.custom_instance_url.trim().trim_end_matches('/');
    if username.is_empty() || base.is_empty() {
        return Err(FetchError::MissingCredentials);
    }
    let url = format!("{base}/api/v1/users/{}/heatmap", url_encode(username));
    let mut args = vec![
        "-fsS".into(),
        url,
        "-H".into(),
        "Content-Type: application/json".into(),
        "-H".into(),
        "User-Agent: COSMIC Weekly Commits".into(),
    ];
    if !token.is_empty() {
        args.push("-H".into());
        args.push(format!("Authorization: token {token}"));
    }
    let response = curl(args, None).await?;
    let entries: Value =
        serde_json::from_str(&response).map_err(|e| FetchError::Parse(e.to_string()))?;
    let array = entries
        .as_array()
        .ok_or_else(|| FetchError::Parse("unexpected Gitea response".into()))?;
    let mut map = BTreeMap::new();
    for entry in array {
        let Some(timestamp) = entry.get("timestamp").and_then(Value::as_i64) else {
            continue;
        };
        let count = entry
            .get("contributions")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let date = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| FetchError::Parse(e.to_string()))?
            .date();
        *map.entry(date_key(date)).or_insert(0) += count;
    }
    Ok(counts_from_map(dates, &map))
}

async fn fetch_gitlab(settings: &Settings, dates: [Date; 7]) -> Result<[u32; 7], FetchError> {
    let username = settings.github_username.trim();
    let token = settings.github_token.trim();
    if username.is_empty() {
        return Err(FetchError::MissingCredentials);
    }
    let base = if settings.custom_instance_url.trim().is_empty() {
        "https://gitlab.com".to_string()
    } else {
        settings
            .custom_instance_url
            .trim()
            .trim_end_matches('/')
            .to_string()
    };
    let url = format!(
        "{base}/api/v4/users/{}/events?action=pushed&per_page=100",
        url_encode(username)
    );
    let mut args = vec![
        "-fsS".into(),
        url,
        "-H".into(),
        "Content-Type: application/json".into(),
        "-H".into(),
        "User-Agent: COSMIC Weekly Commits".into(),
    ];
    if !token.is_empty() {
        args.push("-H".into());
        args.push(format!("PRIVATE-TOKEN: {token}"));
    }
    let response = curl(args, None).await?;
    let entries: Value =
        serde_json::from_str(&response).map_err(|e| FetchError::Parse(e.to_string()))?;
    let array = entries
        .as_array()
        .ok_or_else(|| FetchError::Parse("unexpected GitLab response".into()))?;
    let target: std::collections::BTreeSet<_> = dates.iter().map(|d| date_key(*d)).collect();
    let mut map: BTreeMap<String, u32> = target.iter().map(|d| (d.clone(), 0)).collect();
    for event in array {
        if event.get("action_name").and_then(Value::as_str) != Some("pushed to") {
            continue;
        }
        let Some(created_at) = event.get("created_at").and_then(Value::as_str) else {
            continue;
        };
        let Ok(date_time) = OffsetDateTime::parse(created_at, &Rfc3339) else {
            continue;
        };
        let key = date_key(date_time.date());
        if !target.contains(&key) {
            continue;
        }
        let count = event
            .pointer("/push_data/commit_count")
            .and_then(Value::as_u64)
            .unwrap_or(1) as u32;
        *map.entry(key).or_insert(0) += count;
    }
    Ok(counts_from_map(dates, &map))
}

fn counts_from_map(dates: [Date; 7], map: &BTreeMap<String, u32>) -> [u32; 7] {
    std::array::from_fn(|i| map.get(&date_key(dates[i])).copied().unwrap_or(0))
}

pub fn grade(count: u32) -> &'static str {
    match count {
        0 => "grade0",
        1..=2 => "grade1",
        3..=5 => "grade2",
        6..=10 => "grade3",
        _ => "grade4",
    }
}

pub fn box_hex_color(count: u32, theme_name: ThemeName, color_mode: ColorMode) -> &'static str {
    if count == 0 {
        return "#ffffff";
    }

    let theme = theme(theme_name);
    match color_mode {
        ColorMode::Opacity => theme.grade3,
        ColorMode::Grade => match grade(count) {
            "grade1" => theme.grade1,
            "grade2" => theme.grade2,
            "grade3" => theme.grade3,
            "grade4" => theme.grade4,
            _ => theme.grade0,
        },
    }
}

pub fn box_alpha(count: u32, color_mode: ColorMode) -> f32 {
    if count == 0 || color_mode == ColorMode::Grade {
        return 1.0;
    }
    (DEFAULT_OPACITY + (count as f32 * OPACITY_PER_COMMIT)).min(1.0)
}

pub fn hex_to_rgba(hex: &str, alpha: f32) -> cosmic::iced::Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return cosmic::iced::Color::from_rgba8(255, 255, 255, alpha);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    cosmic::iced::Color::from_rgba8(r, g, b, alpha)
}

pub fn cached_label(cached_at: &str) -> String {
    let Ok(timestamp) = OffsetDateTime::parse(cached_at, &Rfc3339) else {
        return "Cached".to_string();
    };
    let now = OffsetDateTime::now_utc();
    let diff = now - timestamp;
    let seconds = diff.whole_seconds().max(0);
    if seconds < 60 {
        return "Cached just now".to_string();
    }
    let (value, unit) = if seconds < 3600 {
        (seconds / 60, "minute")
    } else if seconds < 86_400 {
        (seconds / 3600, "hour")
    } else if seconds < 604_800 {
        (seconds / 86_400, "day")
    } else if seconds < 2_592_000 {
        (seconds / 604_800, "week")
    } else if seconds < 31_536_000 {
        (seconds / 2_592_000, "month")
    } else {
        (seconds / 31_536_000, "year")
    };
    let suffix = if value == 1 { "" } else { "s" };
    format!("Cached {value} {unit}{suffix} ago")
}

fn url_encode(input: &str) -> String {
    input
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![b as char]
            }
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_thresholds_match_extension() {
        assert_eq!(grade(0), "grade0");
        assert_eq!(grade(1), "grade1");
        assert_eq!(grade(2), "grade1");
        assert_eq!(grade(3), "grade2");
        assert_eq!(grade(5), "grade2");
        assert_eq!(grade(6), "grade3");
        assert_eq!(grade(10), "grade3");
        assert_eq!(grade(11), "grade4");
    }

    #[test]
    fn last_seven_days_are_ordered_oldest_to_today() {
        let settings = Settings::default();
        let dates = dates_for(&settings);
        for window in dates.windows(2) {
            assert_eq!(window[0] + Duration::days(1), window[1]);
        }
    }
}
