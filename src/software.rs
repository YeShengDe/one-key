use ratatui::{prelude::*, widgets::{Block, Borders, List, ListItem}};
use crossterm::{event, execute, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::io::{stdout};

pub fn common_software_menu() {
    let items = vec![
        "安装 Docker",
        "安装 Node.js",
        "安装 Python",
        "安装 Rust",
        "安装 Go",
        "返回主菜单"
    ];
    let mut selected = 0;
    enable_raw_mode().unwrap();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    let res = loop {
        terminal.draw(|f| {
            let size = f.area();
            let items_widget: Vec<ListItem> = items.iter().map(|i| ListItem::new(*i)).collect();
            let list = List::new(items_widget)
                .block(Block::default().borders(Borders::ALL).title("常用软件安装"))
                .highlight_symbol("▶ ")
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            let mut state = ratatui::widgets::ListState::default();
            state.select(Some(selected));
            f.render_stateful_widget(list, size, &mut state);
        }).unwrap();
        if event::poll(std::time::Duration::from_millis(200)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                use crossterm::event::{KeyCode, KeyEventKind};
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            if selected > 0 { selected -= 1; }
                        }
                        KeyCode::Down => {
                            if selected < items.len() - 1 { selected += 1; }
                        }
                        KeyCode::Enter => break selected,
                        KeyCode::Char('q') => break items.len() - 1,
                        _ => {}
                    }
                }
            }
        }
    };
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen).unwrap();
    match res {
        0 => install_docker(),
        1 => install_nodejs(),
        2 => install_python(),
        3 => install_rust(),
        4 => install_go(),
        5 | _ => return,
    }
}

fn install_docker() {
    println!("正在安装 Docker...");
    // 这里可以添加实际的安装逻辑
    println!("Docker 安装完成（模拟）");
}

fn install_nodejs() {
    println!("正在安装 Node.js...");
    // 这里可以添加实际的安装逻辑
    println!("Node.js 安装完成（模拟）");
}

fn install_python() {
    println!("正在安装 Python...");
    // 这里可以添加实际的安装逻辑
    println!("Python 安装完成（模拟）");
}

fn install_rust() {
    println!("正在安装 Rust...");
    // 这里可以添加实际的安装逻辑
    println!("Rust 安装完成（模拟）");
}

fn install_go() {
    println!("正在安装 Go...");
    // 这里可以添加实际的安装逻辑
    println!("Go 安装完成（模拟）");
}