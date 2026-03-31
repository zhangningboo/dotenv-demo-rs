use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use tokio;
use tracing::info;

#[tokio::main(worker_threads = 4)]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();  // 默认只加载一次，从不同地方再次执行也只有第一次生效
    
    // 创建初始 .env 文件
    let _ = modify_env_file("FIX", "fixed_value");
    let _ = modify_env_file("CHANGE", "0");
    
    for i in 0..20 {
        // 先修改文件
        let _ = modify_env_file("CHANGE", i.to_string().as_str());
        
        // 重新加载环境变量
        reload_env_from_file();
        
        info!("第 {} 次: FIX={:?}, CHANGE={:?}", 
              i, env::var("FIX"), env::var("CHANGE"));

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}

fn modify_env_file(key: &str, value: &str) -> std::io::Result<()> {
    let content = std::fs::read_to_string(".env").unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    
    let mut found = false;
    for line in &mut lines {
        if line.starts_with(&format!("{}=", key)) {
            *line = format!("{}={}", key, value);
            found = true;
            break;
        }
    }
    
    if !found {
        lines.push(format!("{}={}", key, value));
    }
    
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(".env")?;
    
    file.write_all(lines.join("\n").as_bytes())?;
    Ok(())
}

fn reload_env_from_file() {
    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();
                // 移除引号
                let value = value.trim_matches('"').trim_matches('\'');
                unsafe { env::set_var(key, value); }
            }
        }
    }
}