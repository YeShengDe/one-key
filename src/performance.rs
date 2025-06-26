pub mod perf_bandwidth;
pub mod perf_cntraceroute;
pub mod perf_io;
pub mod perf_netunlock;
pub mod perf_speedtest;

use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem},
};
use std::io::stdout;

pub fn performance_test_menu() {
    let items = vec![
        "IO测试",
        "网络解锁测试",
        "网速测试",
        "带宽测试",
        "中国一线城市各大网络商来回程线路测试",
        "返回主菜单",
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
                    .block(Block::default().title("性能测试"))
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
                        KeyCode::Char(c)
                            if c >= '1'
                                && c <= char::from_digit(items.len() as u32, 10).unwrap() =>
                        {
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
        0 => perf_io::run_io_test(),
        1 => perf_netunlock::run_netunlock_test(),
        2 => perf_speedtest::run_speedtest(),
        3 => perf_bandwidth::run_bandwidth_test(),
        4 => perf_cntraceroute::run_cntraceroute_test(),
        _ => {}
    }
}
