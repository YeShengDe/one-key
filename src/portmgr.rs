use std::io;
use std::process::Command;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
    execute,
    cursor::Hide,
    terminal::{Clear, ClearType},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Layout, Direction, Constraint},
    widgets::{Block, Paragraph, Borders, List, ListItem},
    style::{Style, Color, Modifier},
    Frame,
};

pub struct PortManager {
    selected: usize,
    message: String,
    input_mode: InputMode,
    input_buffer: String,
    show_input: bool,
    input_prompt: String,
    pending_action: Option<PendingAction>,
    should_exit: bool,
}

#[derive(Clone)]
enum PendingAction {
    OpenPort,
    ClosePort,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Input,
}

impl Default for PortManager {
    fn default() -> Self {
        Self {
            selected: 0,
            message: "使用 ↑↓ 选择，Enter 确认，q 退出".to_string(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            show_input: false,
            input_prompt: String::new(),
            pending_action: None,
            should_exit: false,
        }
    }
}

impl PortManager {
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, Hide, Clear(ClearType::All))?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        disable_raw_mode()?;
        terminal.show_cursor()?;

        result
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if self.should_exit {
                return Ok(());
            }

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.input_mode {
                        InputMode::Normal => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    self.should_exit = true;
                                }
                                KeyCode::Up => {
                                    if self.selected > 0 {
                                        self.selected -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if self.selected < 3 {
                                        self.selected += 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    self.handle_selection();
                                }
                                KeyCode::Char('1') => {
                                    self.selected = 0;
                                    self.handle_selection();
                                }
                                KeyCode::Char('2') => {
                                    self.selected = 1;
                                    self.handle_selection();
                                }
                                KeyCode::Char('3') => {
                                    self.selected = 2;
                                    self.handle_selection();
                                }
                                KeyCode::Char('4') => {
                                    self.selected = 3;
                                    self.handle_selection();
                                }
                                _ => {}
                            }
                        }
                        InputMode::Input => {
                            match key.code {
                                KeyCode::Enter => {
                                    self.handle_input();
                                }
                                KeyCode::Char(c) => {
                                    self.input_buffer.push(c);
                                }
                                KeyCode::Backspace => {
                                    self.input_buffer.pop();
                                }
                                KeyCode::Esc => {
                                    self.cancel_input();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(4),
                Constraint::Length(if self.show_input { 3 } else { 0 }),
            ])
            .split(f.area());

        // 标题
        let title = Paragraph::new("端口管理工具")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        f.render_widget(title, chunks[0]);

        // 菜单
        let items = vec![
            "1. 开放端口",
            "2. 关闭端口", 
            "3. 查看端口",
            "4. 退出程序"
        ];
        
        let menu_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(*item).style(style)
            })
            .collect();

        let menu = List::new(menu_items)
            .block(Block::default().borders(Borders::ALL));
        
        f.render_widget(menu, chunks[1]);

        // 状态信息
        let message_color = if self.message.contains("成功") {
            Color::Green
        } else if self.message.contains("错误") || self.message.contains("失败") {
            Color::Red
        } else {
            Color::Yellow
        };

        let message = Paragraph::new(self.message.as_str())
            .block(Block::default().borders(Borders::ALL).title("状态"))
            .style(Style::default().fg(message_color))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(message, chunks[2]);

        // 输入框
        if self.show_input {
            let input = Paragraph::new(format!("输入: {}", self.input_buffer))
                .block(Block::default().borders(Borders::ALL).title(self.input_prompt.as_str()))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(input, chunks[3]);
        }
    }

    fn handle_selection(&mut self) {
        match self.selected {
            0 => self.start_port_action(PendingAction::OpenPort),
            1 => self.start_port_action(PendingAction::ClosePort),
            2 => self.list_ports(),
            3 => self.should_exit = true,
            _ => {}
        }
    }

    fn start_port_action(&mut self, action: PendingAction) {
        self.pending_action = Some(action.clone());
        self.input_mode = InputMode::Input;
        self.show_input = true;
        self.input_buffer.clear();
        
        self.input_prompt = match action {
            PendingAction::OpenPort => "端口号 (如: 8080 或 8080/tcp)".to_string(),
            PendingAction::ClosePort => "端口号 (如: 8080 或 8080/tcp)".to_string(),
        };
    }

    fn handle_input(&mut self) {
        if let Some(action) = &self.pending_action {
            let port_info = self.input_buffer.trim();
            if !port_info.is_empty() {
                match action {
                    PendingAction::OpenPort => {
                        self.message = self.open_port(port_info);
                    }
                    PendingAction::ClosePort => {
                        self.message = self.close_port(port_info);
                    }
                }
            } else {
                self.message = "错误: 请输入端口号".to_string();
            }
        }
        self.cancel_input();
    }

    fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.show_input = false;
        self.input_buffer.clear();
        self.pending_action = None;
    }

    fn open_port(&self, port_info: &str) -> String {
        let (port, protocol) = self.parse_port_info(port_info);
        
        // 验证端口号
        if let Err(e) = port.parse::<u16>() {
            return format!("错误: 无效端口号 - {}", e);
        }
        
        if self.command_exists("ufw") {
            let cmd = format!("ufw allow {}/{}", port, protocol);
            self.execute_sudo_command(&cmd)
        } else if self.command_exists("firewall-cmd") {
            let cmd = format!("firewall-cmd --permanent --add-port={}/{} && firewall-cmd --reload", port, protocol);
            self.execute_sudo_command(&cmd)
        } else if self.command_exists("iptables") {
            // 修复 iptables 命令
            let cmd = if protocol == "tcp" {
                format!("iptables -I INPUT -p tcp -m tcp --dport {} -j ACCEPT", port)
            } else {
                format!("iptables -I INPUT -p udp -m udp --dport {} -j ACCEPT", port)
            };
            self.execute_sudo_command(&cmd)
        } else {
            "错误: 未找到防火墙工具 (需要 ufw, firewall-cmd 或 iptables)".to_string()
        }
    }

    fn close_port(&self, port_info: &str) -> String {
        let (port, protocol) = self.parse_port_info(port_info);
        
        if let Err(e) = port.parse::<u16>() {
            return format!("错误: 无效端口号 - {}", e);
        }
        
        if self.command_exists("ufw") {
            let cmd = format!("ufw delete allow {}/{}", port, protocol);
            self.execute_sudo_command(&cmd)
        } else if self.command_exists("firewall-cmd") {
            let cmd = format!("firewall-cmd --permanent --remove-port={}/{} && firewall-cmd --reload", port, protocol);
            self.execute_sudo_command(&cmd)
        } else if self.command_exists("iptables") {
            // 修复 iptables 命令
            let cmd = if protocol == "tcp" {
                format!("iptables -D INPUT -p tcp -m tcp --dport {} -j ACCEPT", port)
            } else {
                format!("iptables -D INPUT -p udp -m udp --dport {} -j ACCEPT", port)
            };
            self.execute_sudo_command(&cmd)
        } else {
            "错误: 未找到防火墙工具".to_string()
        }
    }

    fn list_ports(&mut self) {
        let mut result = String::new();
        
        // 检查监听端口
        if let Ok(output) = Command::new("ss")
            .args(&["-tuln"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            result.push_str("监听端口:\n");
            
            let mut ports = Vec::new();
            for line in output_str.lines().skip(1) {
                if let Some(port_info) = self.extract_port_from_ss(line) {
                    ports.push(port_info);
                }
            }
            
            if ports.is_empty() {
                result.push_str("无监听端口\n");
            } else {
                for port in ports.iter().take(10) { // 只显示前10个
                    result.push_str(&format!("  {}\n", port));
                }
                if ports.len() > 10 {
                    result.push_str(&format!("  ... 还有 {} 个端口\n", ports.len() - 10));
                }
            }
        } else if let Ok(output) = Command::new("netstat")
            .args(&["-tuln"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            result.push_str("监听端口:\n");
            
            for line in output_str.lines() {
                if line.contains("LISTEN") || (line.contains("udp") && !line.contains("LISTEN")) {
                    if let Some(addr) = self.extract_port_from_netstat(line) {
                        result.push_str(&format!("  {}\n", addr));
                    }
                }
            }
        }

        self.message = if result.trim().is_empty() {
            "无法获取端口信息".to_string()
        } else {
            result.trim().to_string()
        };
    }

    fn parse_port_info(&self, port_info: &str) -> (String, String) {
        if port_info.contains('/') {
            let parts: Vec<&str> = port_info.split('/').collect();
            (parts[0].to_string(), parts[1].to_lowercase())
        } else {
            (port_info.to_string(), "tcp".to_string())
        }
    }

    fn command_exists(&self, cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn execute_sudo_command(&self, cmd: &str) -> String {
        match Command::new("sudo")
            .arg("-n") // 非交互模式
            .arg("sh")
            .arg("-c")
            .arg(cmd)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.trim().is_empty() {
                        "成功: 操作已完成".to_string()
                    } else {
                        format!("成功: {}", stdout.trim())
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("password") || stderr.contains("sudo") {
                        "错误: 需要 sudo 权限，请在终端中运行".to_string()
                    } else {
                        format!("错误: {}", stderr.trim())
                    }
                }
            }
            Err(e) => format!("执行失败: {}", e),
        }
    }

    fn extract_port_from_netstat(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let addr = parts[3];
            if let Some(port_start) = addr.rfind(':') {
                let port = &addr[port_start + 1..];
                let protocol = if line.contains("tcp") { "tcp" } else { "udp" };
                return Some(format!("{}/{}", port, protocol));
            }
        }
        None
    }

    fn extract_port_from_ss(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let proto = parts[0].to_lowercase();
            let addr = parts[4];
            if let Some(port_start) = addr.rfind(':') {
                let port = &addr[port_start + 1..];
                if port != "*" && !port.is_empty() {
                    return Some(format!("{}/{}", port, proto));
                }
            }
        }
        None
    }
}

pub fn port_manager_menu() {
    let mut port_manager = PortManager::default();
    
    if let Err(e) = port_manager.run() {
        eprintln!("程序错误: {}", e);
    }
}