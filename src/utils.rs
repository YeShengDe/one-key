// src/utils.rs
use std::fs::File;
use std::io::{self, Write};

/// 提示用户输入，并提供默认值
pub fn prompt_input(prompt: &str, default: &str) -> String {
    if default.is_empty() {
        print!("  - {}: ", prompt);
    } else {
        print!("  - {} [默认: {}]: ", prompt, default);
    }
    
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let value = input.trim();
    
    if value.is_empty() {
        default.to_string()
    } else {
        value.to_string()
    }
}

/// 将任务输出内容保存到文件
pub fn save_output(task_name: &str, content: &str) {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("{}_{}.log", task_name, timestamp);
    
    match File::create(&filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.as_bytes()) {
                eprintln!("❌ 写入文件失败: {}", e);
            } else {
                println!("💾 日志已保存到: {}", filename);
            }
        }
        Err(e) => {
            eprintln!("❌ 创建文件失败: {}", e);
        }
    }
}

/// 获取格式化的当前时间
pub fn get_current_time() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 等待用户按键继续
pub fn wait_for_key() {
    println!("\n按 Enter 键返回主菜单...");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}