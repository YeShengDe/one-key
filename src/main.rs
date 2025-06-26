
mod software;
mod sysinfo;
mod portmgr;

use software::common_software_menu;
use sysinfo::run_system_monitor;
// use dialoguer::{theme::ColorfulTheme, Select};
use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem},
};
use std::io::stdout;

fn main() {
    loop {
        match main_menu() {
            MenuAction::Continue => continue,
            MenuAction::Exit => break,
        }
    }
}

enum MenuAction {
    Continue,
    Exit,
}

fn main_menu() -> MenuAction {
    let items = vec![
        "系统信息",
        "常用软件",
        "一键脚本",
        "性能测试",
        "端口管理",
        "退出",
    ];
    let mut selected = 0;
    enable_raw_mode().unwrap();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    use ratatui::widgets::ListState;
    let mut list_state = ListState::default();
    list_state.select(Some(selected));
    let res = loop {
        terminal
            .draw(|f| {
                let size = f.area();
                let items_widget: Vec<ListItem> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| ListItem::new(format!("{}. {}", i + 1, item)))
                    .collect();
                let list = List::new(items_widget)
                    .block(Block::default().title("主菜单"))
                    .highlight_symbol("▶ ")
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD));
                f.render_stateful_widget(list, size, &mut list_state.clone());
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(200)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                use crossterm::event::{KeyCode, KeyEventKind};
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if selected < items.len() - 1 {
                                selected += 1;
                            }
                        }
                        KeyCode::Enter => break selected,
                        KeyCode::Char('q') => break items.len() - 1,
                        KeyCode::Char(c) if c >= '1' && c <= char::from_digit(items.len() as u32, 10).unwrap() => {
                            let idx = c.to_digit(10).unwrap() as usize - 1;
                            if idx < items.len() {
                                break idx;
                            }
                        }
                        _ => {}
                    }
                    list_state.select(Some(selected));
                }
            }
        }
    };
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen).unwrap();
    match res {
        0 => {
            let _ = run_system_monitor();
            MenuAction::Continue
        }
        1 => {
            common_software_menu();
            MenuAction::Continue
        }
        2 => {
            // 一键脚本
            MenuAction::Continue
        }
        3 => {
            performance_test_menu();
            MenuAction::Continue
        }
        4 => {
            portmgr::port_manager_menu();
            MenuAction::Continue
        }
        _ => MenuAction::Exit,
    }
}

mod performance;

use performance::performance_test_menu;

