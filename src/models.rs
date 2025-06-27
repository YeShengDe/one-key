// src/models.rs
use std::collections::HashMap;

/// 定义了主菜单返回的动作
#[derive(Debug, Clone)]
pub enum MenuAction {
    Continue, // 在某些场景下可能需要，暂时保留
    Exit,
    ExecuteSingle(usize),
    ExecuteMultiple(Vec<usize>),
}

/// 定义了菜单中的一个项目
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: usize,
    pub name: &'static str,
    pub description: &'static str,
    pub requires_params: bool,
}

impl MenuItem {
    pub fn new(id: usize, name: &'static str, description: &'static str, requires_params: bool) -> Self {
        Self { id, name, description, requires_params }
    }
}

/// 定义了执行一个任务所需的完整配置
#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub item: MenuItem,
    pub params: HashMap<String, String>,
}