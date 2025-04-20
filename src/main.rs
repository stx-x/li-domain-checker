use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Semaphore,
    time::sleep,
};

/// 命令行参数结构
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 并发数
    #[arg(short, long, default_value = "50")]
    workers: usize,

    /// 延迟时间(秒)
    #[arg(short, long, default_value = "1.0")]
    delay: f64,

    /// 输出目录
    #[arg(short, long, default_value = "li_domain_results")]
    output: String,

    /// 是否扫描4字符域名（全扫描模式）
    #[arg(short, long)]
    full_scan: bool,

    /// 是否扫描4字符纯字母域名
    #[arg(short, long)]
    letters_only: bool,
}

/// 域名扫描结果
#[derive(Debug, Serialize, Deserialize)]
struct DomainResult {
    domain: String,
    status: DomainStatus,
    reply_code: i32,
    message: String,
    timestamp: chrono::DateTime<chrono::Local>,
}

/// 域名状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum DomainStatus {
    Available,
    Registered,
    RateLimited,
    Error,
}

impl DomainStatus {
    fn from_reply_code(code: i32) -> Self {
        match code {
            1 => Self::Available,
            0 => Self::Registered,
            -95 => Self::RateLimited,
            _ => Self::Error,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Registered => "registered",
            Self::RateLimited => "rate_limited",
            Self::Error => "error",
        }
    }
}

/// 域名扫描器
struct LiDomainScanner {
    workers: usize,
    delay: f64,
    output_dir: PathBuf,
    available_domains: Arc<tokio::sync::Mutex<HashSet<String>>>,
    results: Arc<tokio::sync::Mutex<Vec<DomainResult>>>,
    host: String,
    port: u16,
}

impl LiDomainScanner {
    /// 创建新的扫描器实例
    fn new(workers: usize, delay: f64, output_dir: String) -> Result<Self> {
        let output_dir = PathBuf::from(output_dir)
            .join(chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        fs::create_dir_all(&output_dir)
            .context("Failed to create output directory")?;

        Ok(Self {
            workers,
            delay,
            output_dir,
            available_domains: Arc::new(tokio::sync::Mutex::new(HashSet::new())),
            results: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            host: "whois.nic.ch".to_string(),
            port: 4343,
        })
    }

    /// 验证域名格式
    fn is_valid_domain(&self, domain: &str) -> bool {
        if !(1..=4).contains(&domain.len()) {
            return false;
        }
        if domain.starts_with('-') || domain.ends_with('-') || domain.contains("--") {
            return false;
        }
        domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    }

    /// 查询域名状态
    async fn query_domain_check(&self, domain: &str) -> Result<DomainResult> {
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))
            .await
            .context("Failed to connect to whois server")?;
        stream.set_nodelay(true)?;

        let query = format!("{}.li\n", domain);
        stream.write_all(query.as_bytes()).await?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_to_string(&mut response).await?;
        let response = response.trim();

        let (reply_code, message) = match response.split_once(':') {
            Some((code, msg)) => (code.parse().unwrap_or(-99), msg.trim()),
            None => (-99, response),
        };

        let status = DomainStatus::from_reply_code(reply_code);

        if status == DomainStatus::Available {
            let mut available = self.available_domains.lock().await;
            available.insert(format!("{}.li", domain));
        }

        Ok(DomainResult {
            domain: format!("{}.li", domain),
            status,
            reply_code,
            message: message.to_string(),
            timestamp: chrono::Local::now(),
        })
    }

    /// 生成指定长度的域名组合
    fn generate_domains(&self, length: usize, letters_only: bool) -> Vec<String> {
        let chars = if letters_only {
            "abcdefghijklmnopqrstuvwxyz-"
        } else {
            "abcdefghijklmnopqrstuvwxyz0123456789-"
        };
        let mut domains = Vec::with_capacity(match length {
            1 => if letters_only { 27 } else { 36 },
            2 => if letters_only { 27 * 28 } else { 36 * 37 },
            3 => if letters_only { 27 * 28 * 28 } else { 36 * 37 * 37 },
            4 => if letters_only { 27 * 28 * 28 * 28 } else { 36 * 37 * 37 * 37 },
            _ => 0,
        });
        
        match length {
            1 => {
                for c in chars.chars() {
                    if c != '-' {
                        domains.push(c.to_string());
                    }
                }
            }
            2 | 3 | 4 => {
                let mut stack = vec![String::new()];
                while let Some(current) = stack.pop() {
                    if current.len() == length {
                        if self.is_valid_domain(&current) {
                            domains.push(current);
                        }
                        continue;
                    }
                    for c in chars.chars() {
                        if !current.is_empty() && c == '-' && current.ends_with('-') {
                            continue;
                        }
                        stack.push(format!("{}{}", current, c));
                    }
                }
            }
            _ => {}
        }

        domains
    }

    /// 生成重复模式的域名
    fn generate_repeat_pattern_domains(&self, letters_only: bool) -> Vec<String> {
        let chars = if letters_only {
            "abcdefghijklmnopqrstuvwxyz"
        } else {
            "abcdefghijklmnopqrstuvwxyz0123456789"
        };
        let mut domains = Vec::with_capacity(if letters_only { 27 * 28 } else { 36 * 37 }); // 预估容量

        // 四个相同字符
        for c in chars.chars() {
            domains.push(format!("{}{}{}{}", c, c, c, c));
        }

        // 三个相同字符加一个不同字符
        for c1 in chars.chars() {
            for c2 in chars.chars() {
                if c1 != c2 {
                    domains.extend_from_slice(&[
                        format!("{}{}{}{}", c1, c1, c1, c2),
                        format!("{}{}{}{}", c1, c1, c2, c1),
                        format!("{}{}{}{}", c1, c2, c1, c1),
                        format!("{}{}{}{}", c2, c1, c1, c1),
                    ]);
                }
            }
        }

        // 两个相同字符加两个相同字符
        for c1 in chars.chars() {
            for c2 in chars.chars() {
                if c1 != c2 {
                    domains.extend_from_slice(&[
                        format!("{}{}{}{}", c1, c1, c2, c2),
                        format!("{}{}{}{}", c1, c2, c2, c1),
                        format!("{}{}{}{}", c1, c2, c1, c2),
                    ]);
                }
            }
        }

        domains
    }

    /// 扫描域名列表
    async fn scan_domains(&self, domains: Vec<String>) -> Result<()> {
        let total = domains.len();
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap(),
        );

        let semaphore = Arc::new(Semaphore::new(self.workers));
        let mut tasks = Vec::with_capacity(domains.len());

        for domain in domains {
            let sem = semaphore.clone();
            let pb = pb.clone();
            let scanner = self.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let result = scanner.query_domain_check(&domain).await.unwrap();
                
                match result.status {
                    DomainStatus::Available => println!("{}", style(format!("✓ 可用: {}", result.domain)).green()),
                    DomainStatus::Registered => println!("{}", style(format!("✗ 已注册: {}", result.domain)).red()),
                    _ => println!("{}", style(format!("! 错误: {} - {}", result.domain, result.message)).yellow()),
                }

                pb.inc(1);
                result
            });

            tasks.push(task);
            sleep(Duration::from_secs_f64(self.delay)).await;
        }

        let results = futures::future::join_all(tasks).await;
        let mut all_results = self.results.lock().await;
        for result in results {
            all_results.push(result.unwrap());
        }

        pb.finish_with_message("完成");
        Ok(())
    }

    /// 保存扫描结果
    async fn save_results(&self) -> Result<()> {
        let available = self.available_domains.lock().await;
        let results = self.results.lock().await;

        // 保存可用域名
        let available_file = self.output_dir.join("available_domains.txt");
        let mut content = String::new();
        content.push_str("# 可用域名列表\n");
        content.push_str("# 扫描时间: ");
        content.push_str(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        content.push_str("\n\n");
        
        let mut sorted_domains: Vec<_> = available.iter().collect();
        sorted_domains.sort();
        for domain in sorted_domains {
            content.push_str(&format!("{}\n", domain));
        }

        fs::write(&available_file, content)
            .context("Failed to write available domains file")?;

        // 保存完整结果
        let results_file = self.output_dir.join("scan_results.json");
        let json = serde_json::to_string_pretty(&*results)
            .context("Failed to serialize results")?;
        fs::write(&results_file, json)
            .context("Failed to write results file")?;

        Ok(())
    }

    /// 运行扫描器
    async fn run(&self, full_scan: bool, letters_only: bool) -> Result<()> {
        let mut all_domains = Vec::new();
        
        // 生成所有可能的域名组合
        let max_length = if full_scan { 4 } else { 3 };
        for length in 1..=max_length {
            println!("生成 {} 字符域名...", length);
            all_domains.extend(self.generate_domains(length, letters_only));
        }

        if !full_scan {
            println!("生成重复模式域名...");
            all_domains.extend(self.generate_repeat_pattern_domains(letters_only));
        }

        println!("开始扫描 {} 个域名...", all_domains.len());
        self.scan_domains(all_domains).await?;
        self.save_results().await?;

        let available = self.available_domains.lock().await;
        println!("\n{}", style("扫描完成!").green().bold());
        println!("找到 {} 个可用域名", available.len());
        println!("结果已保存到: {}", self.output_dir.display());

        Ok(())
    }
}

impl Clone for LiDomainScanner {
    fn clone(&self) -> Self {
        Self {
            workers: self.workers,
            delay: self.delay,
            output_dir: self.output_dir.clone(),
            available_domains: self.available_domains.clone(),
            results: self.results.clone(),
            host: self.host.clone(),
            port: self.port,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let scanner = LiDomainScanner::new(args.workers, args.delay, args.output)?;
    scanner.run(args.full_scan, args.letters_only).await
}
