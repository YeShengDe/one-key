use std::io;
use std::process::Command;
use crossterm::{event, terminal::{disable_raw_mode, enable_raw_mode}};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Layout, Direction, Constraint, Rect},
    widgets::{Block, Paragraph},
    text::{Span, Line, Text},
    style::{Style, Color},
};

pub fn port_manager_menu() {
    let items = vec!["开放端口", "关闭端口", "查看已开放端口", "返回主菜单"];
    let mut selected = 0;
    let mut message = String::new();

    loop {
        print!("\x1b[2J\x1b[H");
        println!("端口管理 (TCP/UDP)");
        for (i, item) in items.iter().enumerate() {
            if i == selected {
                println!("▶ {}. {}", i + 1, item);
            } else {
                println!("  {}. {}", i + 1, item);
            }
        }
        if !message.is_empty() {
            println!("\n提示: {}", message);
        }

        if event::poll(std::time::Duration::from_millis(200)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                use crossterm::event::{KeyCode, KeyEventKind};
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => { if selected > 0 { selected -= 1; } }
                        KeyCode::Down => { if selected < items.len() - 1 { selected += 1; } }
                        KeyCode::Enter => {
                            message.clear();
                            message = match selected {
                                0 => tui_port_action(true),
                                1 => tui_port_action(false),
                                2 => tui_list_ports(),
                                3 => break,
                                _ => String::new(),
                            };
                        }
                        KeyCode::Char('q') => break,
                        KeyCode::Char(c) if c >= '1' && c <= char::from_digit(items.len() as u32, 10).unwrap() => {
                            let idx = c.to_digit(10).unwrap() as usize - 1;
                            if idx < items.len() { selected = idx; }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r)[1];

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical)[1]
}

fn tui_port_action(is_open: bool) -> String {
    let mut port = String::new();
    let mut proto_idx = 0;
    let protos = ["tcp", "udp"];
    let mut msg = String::new();
    let mut done = false;

    enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    while !done {
        terminal.draw(|f| {
            let area = centered_rect(50, 10, f.area());
            let mut lines: Vec<Line> = vec![
                Line::from(vec![Span::raw(format!("请输入端口号: {}", port))]),
                Line::from(vec![
                    Span::raw("协议 (← → 切换): "),
                    Span::styled(protos[proto_idx], Style::default().fg(Color::Yellow)),
                ]),
            ];
            if !msg.is_empty() {
                lines.push(Line::from(Span::styled(&msg, Style::default().fg(Color::Red))));
            }

            let para = Paragraph::new(Text::from(lines))
                .block(Block::default().title(if is_open { "开放端口" } else { "关闭端口" }));
            f.render_widget(para, area);
        }).unwrap();

        if event::poll(std::time::Duration::from_millis(200)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                use crossterm::event::KeyCode::*;
                match key.code {
                    Esc => return String::new(),
                    Enter => {
                        if let Ok(port_num) = port.trim().parse::<u16>() {
                            let proto = protos[proto_idx];
                            let ok = if is_open {
                                run_firewall_cmd("add", port_num, proto)
                            } else {
                                run_firewall_cmd("remove", port_num, proto)
                            };
                            msg = if ok {
                                format!("{} {} 端口 {} 成功", if is_open { "开放" } else { "关闭" }, proto, port_num)
                            } else {
                                "操作失败，请检查权限或防火墙状态。".to_string()
                            };
                            done = true;
                        } else {
                            msg = "无效的端口号".to_string();
                        }
                    }
                    Left => proto_idx = (proto_idx + protos.len() - 1) % protos.len(),
                    Right => proto_idx = (proto_idx + 1) % protos.len(),
                    Backspace => { port.pop(); }
                    Char(c) if c.is_ascii_digit() && port.len() < 5 => port.push(c),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    msg
}

fn run_firewall_cmd(action: &str, port: u16, proto: &str) -> bool {
    let ufw = Command::new("which").arg("ufw").output().map(|o| o.status.success()).unwrap_or(false);
    if ufw {
        let cmd = match action {
            "add" => vec!["allow"],
            "remove" => vec!["delete", "allow"],
            _ => vec![],
        };
        let status = Command::new("sudo")
            .arg("ufw")
            .args(&cmd)
            .arg(format!("{}/{}", port, proto))
            .status();
        return status.map(|s| s.success()).unwrap_or(false);
    } else {
        let ipt = Command::new("which").arg("iptables").output().map(|o| o.status.success()).unwrap_or(false);
        if !ipt {
            let _ = Command::new("sudo").arg("apt").arg("update").status();
            let _ = Command::new("sudo").arg("apt").arg("install").arg("-y").arg("iptables").status();
            let _ = Command::new("sudo").arg("apt").arg("install").arg("-y").arg("iptables-persistent").status();
        }
        let base = match action {
            "add" => "-A",
            "remove" => "-D",
            _ => "",
        };
        let status = Command::new("sudo")
            .arg("iptables")
            .arg(base)
            .arg("INPUT")
            .arg("-p").arg(proto)
            .arg("--dport").arg(port.to_string())
            .arg("-j").arg("ACCEPT")
            .status();
        let _ = Command::new("sudo").arg("netfilter-persistent").arg("save").status();
        return status.map(|s| s.success()).unwrap_or(false);
    }
}

fn tui_list_ports() -> String {
    let output = Command::new("ss").arg("-tuln").output();
    if let Ok(out) = output {
        let s = String::from_utf8_lossy(&out.stdout).to_string();
        let mut lines: Vec<&str> = s.lines().collect();
        if lines.len() > 20 { lines = lines[..20].to_vec(); }
        lines.join("\n")
    } else {
        "无法获取端口信息".to_string()
    }
}
