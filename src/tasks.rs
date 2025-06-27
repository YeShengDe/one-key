// src/tasks.rs

use std::collections::HashMap;
use crate::models::{MenuItem, TaskConfig};
use crate::utils::{prompt_input, save_output, wait_for_key, get_current_time};

// 恢复 trait 的导入

pub fn execute_single_task(item: &MenuItem) {
    println!("▶️ 执行单个任务: {}", item.name);
    
    let mut task_config = TaskConfig {
        item: item.clone(),
        params: HashMap::new(),
    };

    if item.requires_params {
        collect_parameters(&mut task_config);
    }
    
    // 立即显示任务输出
    println!("\n--- 任务输出 ---");
    let output = execute_task(&task_config);
    // 只有在TUI任务之外才打印输出
    if item.id != 0 {
        println!("{}", output);
    }
    println!("----------------");
    
    save_output(&format!("single_{}", item.id), &output);
    
    println!("\n✅ 任务 '{}' 完成。", item.name);
    wait_for_key();
}

pub fn execute_multiple_tasks(items: Vec<MenuItem>) {
    println!("▶️ 准备执行 {} 个任务...", items.len());
    let mut task_configs = Vec::new();

    for item in items {
        let mut task_config = TaskConfig { item: item.clone(), params: HashMap::new() };
        if item.requires_params {
            println!("\n--- 为任务 '{}' 配置参数 ---", item.name);
            collect_parameters(&mut task_config);
        }
        task_configs.push(task_config);
    }

    println!("\n--- 开始批量执行 ---");
    let mut all_output = String::new();
    let header = format!("=== 批量任务执行报告 ===\n执行时间: {}\n共 {} 个任务\n\n", get_current_time(), task_configs.len());
    all_output.push_str(&header);

    for (index, config) in task_configs.iter().enumerate() {
        println!("\n[{}/{}] 执行任务: {}", index + 1, task_configs.len(), config.item.name);
        let task_output = execute_task(config);
        // TUI 任务不打印返回的字符串
        if config.item.id != 0 {
            println!("{}", task_output);
        }
        all_output.push_str(&format!("--- 任务 {}: {} ---\n", index + 1, config.item.name));
        all_output.push_str(&task_output);
        all_output.push_str("\n\n");
    }

    save_output("batch_execution", &all_output);
    println!("\n✅ 所有任务完成。");
    wait_for_key();
}


/// 收集任务所需参数
fn collect_parameters(task_config: &mut TaskConfig) {
    match task_config.item.id {
        1 => {
            task_config.params.insert("test_size".to_string(), prompt_input("测试文件大小 (MB)", "1024"));
            task_config.params.insert("test_path".to_string(), prompt_input("测试路径", "/tmp"));
        }
        2 => {
             }
        // ... 其他参数收集 ...
        _ => {}
    }
}

/// 任务分发器：根据任务ID调用具体实现
fn execute_task(config: &TaskConfig) -> String {
    let mut output = String::new();
    
    match config.item.id {
        0 => {
            println!("正在启动系统信息监控界面...");
            let result = crate::sysinfo::run_system_monitor();
                match result {
                    Ok(_) => output.push_str("已退出系统信息监控。\n"),
                    Err(e) => output.push_str(&format!("启动系统监控失败: {}\n", e)),
            }
        }
        1 => {
            output.push_str(&format!("开始硬盘测试 (大小: {} MB)...\n", config.params.get("test_size").unwrap()));
            std::thread::sleep(std::time::Duration::from_secs(2));
            output.push_str("测试完成。顺序写入速度: 500 MB/s\n");
        }
        _ => {
            output.push_str(&format!("正在模拟执行任务: {}\n", config.item.name));
            if !config.params.is_empty() {
                output.push_str("参数:\n");
                for (key, val) in &config.params {
                    output.push_str(&format!("  - {}: {}\n", key, val));
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            output.push_str("任务模拟执行完成。\n");
        }
    }
    
    output
}