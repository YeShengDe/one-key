// 获取 Rust 版本
fn get_rust_version() -> String {
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if let Ok(ver) = String::from_utf8(output.stdout) {
            return ver.trim().to_string();
        }
    }
    "未知".to_string()
}

// 获取 TCP 拥塞算法
fn get_tcp_congestion_algo() -> String {
    if let Ok(output) = Command::new("cat").arg("/proc/sys/net/ipv4/tcp_congestion_control").output() {
        if let Ok(algo) = String::from_utf8(output.stdout) {
            return algo.trim().to_string();
        }
    }
    "未知".to_string()
}

// 获取 DNS 服务器
fn get_dns() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
        for line in content.lines() {
            if line.starts_with("nameserver") {
                if let Some(dns) = line.split_whitespace().nth(1) {
                    return dns.to_string();
                }
            }
        }
    }
    "".to_string()
}
use chrono::{Local, DateTime};
use hostname::get;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{System, Networks, Disks};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Gauge, List, ListItem, Paragraph, Wrap},
    Terminal, Frame,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Debug)]
pub struct NetworkData {
    pub isp: String,
    pub ipv4: String,
    pub ipv6: String,
    pub ipv6_support: bool,
    pub dns: String,
    pub location: String,
    pub loading: bool,
}

impl Default for NetworkData {
    fn default() -> Self {
        Self {
            isp: "Loading...".to_string(),
            ipv4: "Loading...".to_string(),
            ipv6: "Loading...".to_string(),
            ipv6_support: false,
            dns: "Loading...".to_string(),
            location: "Loading...".to_string(),
            loading: true,
        }
    }
}

pub struct SystemInfo {
    sys: System,
    boot_time: DateTime<Local>,
    network_data: Arc<Mutex<NetworkData>>,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let boot_time = SystemTime::now()
            .checked_sub(Duration::from_secs(System::uptime()))
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
            .flatten()
            .map(|utc| utc.with_timezone(&Local::now().timezone()))
            .unwrap_or_else(|| Local::now());

        let network_data = Arc::new(Mutex::new(NetworkData::default()));
        let network_data_clone = Arc::clone(&network_data);
        // 启动后台线程获取网络信息，获取后自动刷新界面
        thread::spawn(move || {
            let data = fetch_network_info();
            if let Ok(mut network) = network_data_clone.lock() {
                *network = data;
            }
            // 通知主线程刷新（通过写入一个文件描述符或其它机制，简化为发送 SIGUSR1）
            #[cfg(target_os = "linux")]
            unsafe {
                libc::kill(libc::getpid(), libc::SIGUSR1);
            }
        });

        Self {
            sys,
            boot_time,
            network_data,
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
        
        // 重新获取网络信息
        let network_data_clone = Arc::clone(&self.network_data);
        thread::spawn(move || {
            // 设置为加载状态
            if let Ok(mut network) = network_data_clone.lock() {
                network.loading = true;
            }
            
            let data = fetch_network_info();
            if let Ok(mut network) = network_data_clone.lock() {
                *network = data;
            }
        });
    }

    fn get_system_info(&self) -> Vec<ListItem> {
        let hostname = get().ok().and_then(|h| h.into_string().ok()).unwrap_or_default();
        let os_version = System::long_os_version().unwrap_or_default();
        let kernel_version = System::kernel_version().unwrap_or_default();
        let arch = std::env::consts::ARCH;
        let current_user = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default();
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "Unknown".to_string());
        let shell_name = shell.split('/').last().map(|s| s.to_string()).unwrap_or(shell.clone());
        let rust_version = get_rust_version();
        let uptime = self.format_uptime();
        let aes_ni = get_aes_ni_support();
        let vm_support = get_vm_support();
        let vm_type = get_vm_type();

        vec![
            ListItem::new(Line::from(vec![
                Span::styled("🏠 主机名: ", Style::default().fg(Color::Cyan)),
                Span::raw(hostname),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("💽 操作系统: ", Style::default().fg(Color::Cyan)),
                Span::raw(os_version),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("⚙️ 内核版本: ", Style::default().fg(Color::Cyan)),
                Span::raw(kernel_version),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("🏗️ 系统架构: ", Style::default().fg(Color::Cyan)),
                Span::raw(arch),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("👤 当前用户: ", Style::default().fg(Color::Cyan)),
                Span::raw(current_user),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("🐚 Shell: ", Style::default().fg(Color::Cyan)),
                Span::raw(shell_name),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("🦀 Rust版本: ", Style::default().fg(Color::Cyan)),
                Span::raw(rust_version),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("🔐 AES-NI: ", Style::default().fg(Color::Cyan)),
                Span::raw(aes_ni),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("💻 VM-x/AMD-V: ", Style::default().fg(Color::Cyan)),
                Span::raw(vm_support),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("🖥️ VM类型: ", Style::default().fg(Color::Cyan)),
                Span::raw(vm_type),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("🚀 启动时间: ", Style::default().fg(Color::Cyan)),
                Span::raw(self.boot_time.format("%Y-%m-%d %H:%M:%S").to_string()),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("⏰ 运行时长: ", Style::default().fg(Color::Cyan)),
                Span::raw(uptime),
            ])),
        ]
    }

    fn get_hardware_info(&self) -> Vec<ListItem> {
        let cpu_brand = self.sys.cpus().first().map(|c| c.brand()).unwrap_or_default();
        let cpu_cores = self.sys.cpus().len();
        let cpu_freq = self.sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
        
        let total_mem = self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_mem = self.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_swap = self.sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_swap = self.sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;

        let mut items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("🔥 CPU型号: ", Style::default().fg(Color::Magenta)),
                Span::raw(cpu_brand),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("⚡ CPU核心: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{} 核心", cpu_cores)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("📊 CPU频率: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{} MHz", cpu_freq)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("💾 物理内存: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{:.1}/{:.1} GB", used_mem, total_mem)),
            ])),
        ];

        if total_swap > 0.0 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("💿 虚拟内存: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{:.1}/{:.1} GB", used_swap, total_swap)),
            ])));
        }

        // 添加磁盘信息
        let disks = Disks::new_with_refreshed_list();
        for (i, disk) in disks.iter().enumerate() {
            let mount_point = disk.mount_point().to_string_lossy();
            let total_gb = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let available_gb = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_gb = total_gb - available_gb;
            let usage_percent = if total_gb > 0.0 { used_gb / total_gb * 100.0 } else { 0.0 };
            
            let disk_label = if i == 0 { "💾 主硬盘" } else { &format!("💾 硬盘{}", i + 1) };
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{}: ", disk_label), Style::default().fg(Color::Magenta)),
                Span::raw(format!("{:.1}/{:.1} GB ({:.1}%) - {}", 
                         used_gb, total_gb, usage_percent, mount_point)),
            ])));
        }

        items
    }

    fn get_network_info(&self) -> Vec<ListItem> {
        let networks = Networks::new_with_refreshed_list();
        let net_algo = get_tcp_congestion_algo();
        
        let network_data = if let Ok(data) = self.network_data.lock() {
            data.clone()
        } else {
            NetworkData::default()
        };

        let mut items = vec![];

        // 显示主要网卡信息
        for (interface_name, data) in networks.iter() {
            if data.received() > 0 || data.transmitted() > 0 {
                let received_mb = data.received() as f64 / 1024.0 / 1024.0;
                let transmitted_mb = data.transmitted() as f64 / 1024.0 / 1024.0;
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("📡 网卡 {}: ", interface_name), Style::default().fg(Color::Green)),
                    Span::raw(format!("↓{:.1}MB ↑{:.1}MB", received_mb, transmitted_mb)),
                ])));
            }
        }

        // 只显示IPv4和运营商等，IPv6仅在有时显示
        items.push(ListItem::new(Line::from(vec![
            Span::styled("🌍 公网IPv4: ", Style::default().fg(Color::Green)),
            Span::raw(network_data.ipv4.clone()),
        ])));

        if network_data.ipv6_support && !network_data.ipv6.is_empty() && network_data.ipv6 != "Loading..." && network_data.ipv6 != "无公网IPv6" {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("🌍 公网IPv6: ", Style::default().fg(Color::Green)),
                Span::raw(network_data.ipv6.clone()),
            ])));
        }

        items.push(ListItem::new(Line::from(vec![
            Span::styled("📶 ISP运营商: ", Style::default().fg(Color::Green)),
            Span::raw(network_data.isp.clone()),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("📍 地理位置: ", Style::default().fg(Color::Green)),
            Span::raw(network_data.location.clone()),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("🔍 DNS服务器: ", Style::default().fg(Color::Green)),
            Span::raw(network_data.dns.clone()),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("⚡ 拥塞算法: ", Style::default().fg(Color::Green)),
            Span::raw(if net_algo.is_empty() { "未知".to_string() } else { net_algo }),
        ])));

        items
    }

    fn get_performance_info(&self) -> (f64, f64, f64, f64) {
        let cpu_cores = self.sys.cpus().len();
        let cpu_usage: f32 = self.sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_cores as f32;
        
        let used_mem = self.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_mem = self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let mem_percent = if total_mem > 0.0 { used_mem / total_mem * 100.0 } else { 0.0 };
        
        let used_swap = self.sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_swap = self.sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
        let swap_percent = if total_swap > 0.0 { used_swap / total_swap * 100.0 } else { 0.0 };

        (cpu_usage as f64, mem_percent, swap_percent, total_swap)
    }

    fn format_uptime(&self) -> String {
        let uptime_secs = System::uptime();
        let days = uptime_secs / 86400;
        let hours = (uptime_secs % 86400) / 3600;
        let minutes = (uptime_secs % 3600) / 60;
        
        if days > 0 {
            format!("{}天 {}时 {}分", days, hours, minutes)
        } else if hours > 0 {
            format!("{}时 {}分", hours, minutes)
        } else {
            format!("{}分钟", minutes)
        }
    }
}

fn ui(f: &mut Frame, app: &SystemInfo, refresh_count: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.area());

    // 顶部标题，带刷新次数
    let title = Paragraph::new(format!("🖥️  系统信息概览   刷新次数: {}", refresh_count))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default())
        .wrap(Wrap { trim: true });
    f.render_widget(title, chunks[0]);

    // 主体内容区域
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // 左侧区域
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);

    // 右侧区域
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    // 系统基础信息
    let system_info = List::new(app.get_system_info())
        .block(Block::default()
            .title("📋 系统基础信息"));
    f.render_widget(system_info, left_chunks[0]);

    // 硬件配置信息
    let hardware_info = List::new(app.get_hardware_info())
        .block(Block::default()
            .title("🖥️  硬件配置信息"));
    f.render_widget(hardware_info, left_chunks[1]);

    // 网络连接信息
    let network_info = List::new(app.get_network_info())
        .block(Block::default()
            .title("🌐 网络连接信息"));
    f.render_widget(network_info, right_chunks[0]);

    // 性能监控
    let perf_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3)])
        .split(right_chunks[1]);

    let (cpu_usage, mem_percent, swap_percent, total_swap) = app.get_performance_info();

    // CPU使用率
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("🔥 CPU使用率"))
        .gauge_style(Style::default().fg(Color::Red))
        .percent(cpu_usage as u16)
        .label(format!("{:.1}%", cpu_usage));
    f.render_widget(cpu_gauge, perf_chunks[0]);

    // 内存使用率
    let mem_gauge = Gauge::default()
        .block(Block::default().title("💾 内存使用率"))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(mem_percent as u16)
        .label(format!("{:.1}%", mem_percent));
    f.render_widget(mem_gauge, perf_chunks[1]);

    // 交换分区使用率（如果存在）
    if total_swap > 0.0 {
        let swap_gauge = Gauge::default()
            .block(Block::default().title("💿 交换分区使用率"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(swap_percent as u16)
            .label(format!("{:.1}%", swap_percent));
        f.render_widget(swap_gauge, perf_chunks[2]);
    } else {
        // 显示提示信息
        let info = Paragraph::new("💿 无交换分区")
            .block(Block::default().title("💿 交换分区"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(info, perf_chunks[2]);
    }

    // 底部帮助信息（去除 Q 操作，仅保留 R/Enter）
    let help_text = vec![
        Line::from(vec![
            Span::styled("按键: ", Style::default().fg(Color::Yellow)),
            Span::raw("R"),
            Span::styled(" - 刷新  ", Style::default().fg(Color::Gray)),
            Span::raw("Enter"),
            Span::styled(" - 返回主菜单/退出", Style::default().fg(Color::Gray)),
        ])
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    let help_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y + chunks[1].height - 1,
        width: chunks[1].width,
        height: 1,
    };
    f.render_widget(help, help_area);
}

pub fn run_system_monitor() -> Result<(), Box<dyn std::error::Error>> {
    // 启用 raw mode 确保按键响应迅速
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = SystemInfo::new();
    let mut refresh_count = 0;
    let mut needs_redraw = true;

    // 注册 SIGUSR1 信号处理器用于自动刷新
    #[cfg(target_os = "linux")]
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use signal_hook::consts::SIGUSR1;
        use signal_hook::flag;
        let redraw_flag = Arc::new(AtomicBool::new(false));
        flag::register(SIGUSR1, Arc::clone(&redraw_flag)).unwrap();
        loop {
            if needs_redraw || redraw_flag.swap(false, Ordering::SeqCst) {
                terminal.draw(|f| ui(f, &app, refresh_count))?;
                needs_redraw = false;
            }
            if crossterm::event::poll(std::time::Duration::from_millis(500))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Enter => {
                            break;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            app.refresh();
                            refresh_count += 1;
                            needs_redraw = true;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    loop {
        if needs_redraw {
            terminal.draw(|f| ui(f, &app, refresh_count))?;
            needs_redraw = false;
        }
        if crossterm::event::poll(std::time::Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Enter => {
                        break;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        app.refresh();
                        refresh_count += 1;
                        needs_redraw = true;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

// 新增的辅助函数
fn fetch_network_info() -> NetworkData {
    let mut data = NetworkData::default();
    data.loading = false;
    
    // 获取 IPv6 支持信息
    data.ipv6_support = check_ipv6_support();
    
    // 获取网络信息
    match ureq::get("https://ipinfo.io/json")
        .timeout(std::time::Duration::from_secs(10))
        .call() 
    {
        Ok(resp) => {
            match resp.into_json::<serde_json::Value>() {
                Ok(json) => {
                    data.isp = json.get("org")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知")
                        .to_string();
                    data.ipv4 = json.get("ip")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知")
                        .to_string();
                    let city = json.get("city")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let region = json.get("region")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let country = json.get("country")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    data.location = format!("{} {} {}", country, region, city).trim().to_string();
                    if data.location.is_empty() {
                        data.location = "未知".to_string();
                    }
                }
                Err(_) => {
                    data.isp = "获取失败".to_string();
                    data.ipv4 = "获取失败".to_string();
                    data.location = "获取失败".to_string();
                }
            }
        }
        Err(_) => {
            data.isp = "网络错误".to_string();
            data.ipv4 = "网络错误".to_string();
            data.location = "网络错误".to_string();
        }
    }
    
    // 如果支持 IPv6，获取 IPv6 地址
    if data.ipv6_support {
        data.ipv6 = get_ipv6_address();
    }
    
    data.dns = get_dns();
    if data.dns.is_empty() {
        data.dns = "未知".to_string();
    }
    
    data
}

fn check_ipv6_support() -> bool {
    // 检查 /proc/net/if_inet6 文件是否存在
    if std::path::Path::new("/proc/net/if_inet6").exists() {
        return true;
    }
    
    // 尝试通过 ip 命令检查
    if let Ok(output) = Command::new("ip")
        .args(["-6", "addr", "show"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            return !output_str.is_empty() && output_str.contains("inet6");
        }
    }
    
    // 尝试通过 ifconfig 检查
    if let Ok(output) = Command::new("ifconfig")
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            return output_str.contains("inet6");
        }
    }
    
    false
}

fn get_ipv6_address() -> String {
    // 尝试通过 curl 获取公网 IPv6 地址
    if let Ok(output) = Command::new("curl")
        .args(["-6", "-s", "--max-time", "5", "https://api6.ipify.org"])
        .output()
    {
        if let Ok(ipv6) = String::from_utf8(output.stdout) {
            let trimmed = ipv6.trim();
            if !trimmed.is_empty() && trimmed.contains(':') {
                return trimmed.to_string();
            }
        }
    }
    
    // 尝试从本地接口获取 IPv6 地址
    if let Ok(output) = Command::new("ip")
        .args(["-6", "addr", "show", "scope", "global"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                if line.contains("inet6") && !line.contains("fe80") {
                    if let Some(addr) = line.split_whitespace().nth(1) {
                        if let Some(ipv6) = addr.split('/').next() {
                            return ipv6.to_string();
                        }
                    }
                }
            }
        }
    }
    
    "无公网IPv6".to_string()
}

fn get_aes_ni_support() -> String {
    if let Ok(output) = Command::new("grep")
        .args(["-o", "aes", "/proc/cpuinfo"])
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            return "✔ Enabled".to_string();
        }
    }
    
    // 备用检查方法
    if let Ok(output) = Command::new("cat")
        .arg("/proc/cpuinfo")
        .output()
    {
        if let Ok(content) = String::from_utf8(output.stdout) {
            if content.contains("aes") {
                return "✔ Enabled".to_string();
            }
        }
    }
    
    "❌ Disabled".to_string()
}

fn get_vm_support() -> String {
    if let Ok(output) = Command::new("grep")
        .args(["-E", "(vmx|svm)", "/proc/cpuinfo"])
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            return "✔ Enabled".to_string();
        }
    }
    
    "❌ Disabled".to_string()
}

fn get_vm_type() -> String {
    // 检查是否在虚拟机中运行
    let vm_indicators = [
        ("/sys/class/dmi/id/product_name", vec!["VMware", "VirtualBox", "QEMU", "KVM", "Bochs", "Parallels", "BHYVE"]),
        ("/sys/class/dmi/id/sys_vendor", vec!["VMware", "Oracle Corporation", "QEMU", "Red Hat", "Microsoft", "Xen", "Parallels", "BHYVE"]),
        ("/proc/scsi/scsi", vec!["VMware", "Virtual", "QEMU", "KVM", "Xen"]),
    ];
    for (file, keywords) in &vm_indicators {
        if let Ok(content) = std::fs::read_to_string(file) {
            for keyword in keywords {
                if content.contains(keyword) {
                    return keyword.to_string();
                }
            }
        }
    }
    // 检查 systemd-detect-virt
    if let Ok(output) = Command::new("systemd-detect-virt").arg("-q").output() {
        if output.status.success() {
            if let Ok(virt_type) = String::from_utf8(output.stdout) {
                let vt = virt_type.trim();
                if !vt.is_empty() && vt != "none" {
                    return vt.to_string();
                }
            }
        }
    }
    "Physical/Unknown".to_string()
}