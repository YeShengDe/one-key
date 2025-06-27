// src/main.rs
mod models;
mod ui;
mod tasks;
mod utils;

// 功能模块
mod sysinfo;
mod performance;
mod portmgr;
mod software;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use std::io::stdout;
use crate::models::MenuAction;

fn main() -> std::io::Result<()> {
    loop {
        // 进入TUI模式
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        // 显示菜单并获取用户操作
        let action = ui::main_menu();
        
        // 退出TUI模式，以便执行任务和打印输出
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;

        let should_exit = match action {
            MenuAction::Exit => true,
            MenuAction::ExecuteSingle(index) => {
                let items = ui::get_menu_items();
                tasks::execute_single_task(&items[index]);
                false // 不退出，继续循环
            }
            MenuAction::ExecuteMultiple(indices) => {
                let items = ui::get_menu_items();
                let selected_tasks = indices.into_iter().map(|i| items[i].clone()).collect();
                tasks::execute_multiple_tasks(selected_tasks);
                false // 不退出，继续循环
            }
            _ => false,
        };

        if should_exit {
            break;
        }
    }
    Ok(())
}