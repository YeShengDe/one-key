// src/ui.rs
use crate::models::{MenuAction, MenuItem};
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io::stdout;
use std::time::{Duration, Instant};

/// 显示主菜单并处理用户交互，返回用户的选择
pub fn main_menu() -> MenuAction {
    let items = get_menu_items();
    let mut selected_index = 0;
    let mut selected_items: Vec<bool> = vec![false; items.len()];

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    let mut num_input = String::new();
    let mut last_num_input_time: Option<Instant> = None;

    let action = loop {
        terminal
            .draw(|f| {
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ])
                    .split(size);

                let title = Paragraph::new("OneKey VPS 助手")
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                let list_items: Vec<ListItem> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let prefix = if selected_items[i] { "[✓]" } else { "[ ]" };
                        let main = format!("{} {}. {}", prefix, i + 1, item.name);
                        let desc = format!("({})", item.description);
                        let main_span = Span::styled(main, Style::default().add_modifier(Modifier::BOLD));
                        let desc_span = Span::styled(format!(" {}", desc), Style::default().fg(Color::Gray));
                        ListItem::new(Line::from(vec![main_span, desc_span]))
                    })
                    .collect();

                let list = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL).title("选择要执行的任务"))
                    .highlight_symbol("▶ ")
                    .highlight_style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    );
                
                let mut list_state = ListState::default();
                list_state.select(Some(selected_index));
                f.render_stateful_widget(list, chunks[1], &mut list_state);

                let selected_count = selected_items.iter().filter(|&&x| x).count();
                let help_text = "↑/↓:选择 | Space:标记/取消 | Enter:执行 | q:退出";
                let mut status_text = if selected_count > 0 {
                    format!("{}\n已标记 {} 项任务", help_text, selected_count)
                } else {
                    help_text.to_string()
                };
                if !num_input.is_empty() {
                    status_text.push_str(&format!(" | 数字输入: {}", num_input));
                }

                let help = Paragraph::new(status_text)
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(help, chunks[2]);
            })
            .unwrap();

        if let Some(last) = last_num_input_time {
            if last.elapsed() >= Duration::from_millis(800) {
                if let Ok(idx) = num_input.parse::<usize>() {
                    if idx > 0 && idx <= items.len() {
                        break MenuAction::ExecuteSingle(idx - 1);
                    }
                }
                num_input.clear();
                last_num_input_time = None;
            }
        }

        if event::poll(Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    if !matches!(key.code, KeyCode::Char(c) if c.is_ascii_digit()) {
                        num_input.clear();
                        last_num_input_time = None;
                    }

                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break MenuAction::Exit;
                    }
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if selected_index > 0 { selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if selected_index < items.len() - 1 { selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            selected_items[selected_index] = !selected_items[selected_index];
                        }
                        KeyCode::Enter => {
                            let indices: Vec<usize> = selected_items
                                .iter().enumerate()
                                .filter_map(|(i, &s)| if s { Some(i) } else { None })
                                .collect();
                            
                            if !indices.is_empty() {
                                break MenuAction::ExecuteMultiple(indices);
                            } else {
                                break MenuAction::ExecuteSingle(selected_index);
                            }
                        }
                        KeyCode::Char('q') => break MenuAction::Exit,
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            num_input.push(c);
                            last_num_input_time = Some(Instant::now());
                        }
                        _ => {}
                    }
                }
            }
        }
    };
    action
}

/// 定义所有菜单项
pub fn get_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new(0, "系统信息", "显示系统详细信息", false),
        MenuItem::new(1, "硬盘测试", "测试硬盘读写性能", true),
        MenuItem::new(2, "CPU测试", "测试CPU性能和稳定性", true),
        MenuItem::new(3, "解锁测试", "测试流媒体解锁情况", false),
        MenuItem::new(4, "网速测试", "测试网络上/下行速度", true),
        MenuItem::new(5, "三网ping测试", "测试到三大运营商的延迟", false),
        MenuItem::new(6, "三网回程测试", "测试到三大运营商的回程路由", false),
        MenuItem::new(7, "sing-box一键脚本", "部署sing-box代理服务", true),
        MenuItem::new(8, "xray一键脚本", "部署xray代理服务", true),
        MenuItem::new(9, "内存占用排行", "显示内存使用排行", false),
        MenuItem::new(10, "开放端口", "使用防火墙开放指定端口", true),
        MenuItem::new(11, "关闭端口", "使用防火墙关闭指定端口", true),
        MenuItem::new(12, "k3s", "安装/管理轻量化K8s", true),
        MenuItem::new(13, "k8s", "安装/管理标准K8s", true),
        MenuItem::new(14, "tcp调优", "应用BBR等TCP网络优化", true),
    ]
}