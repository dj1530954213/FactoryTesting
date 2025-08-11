/// 全局日志适配器模块 - 实现log::Log trait
use super::{StructuredLog, AsyncLogProcessor, CoreLogCategory};
use std::sync::{Arc, Weak};
use log;
use once_cell;

/// 全局日志适配器 - 实现log::Log trait
/// 使用弱引用避免循环引用问题
pub struct GlobalLoggerAdapter {
    processor: Arc<AsyncLogProcessor>,
}

impl GlobalLoggerAdapter {
    pub fn new(processor: &AsyncLogProcessor) -> Self {
        Self {
            processor: Arc::new(processor.clone()),
        }
    }
    
    /// 从Arc创建新实例
    pub fn from_arc(processor: Arc<AsyncLogProcessor>) -> Self {
        Self { processor }
    }
}

impl log::Log for GlobalLoggerAdapter {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // 根据日志级别过滤
        metadata.level() <= log::max_level()
    }
    
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        // 构建结构化日志条目
        let entry = StructuredLog {
            timestamp: chrono::Utc::now(),
            level: record.level().to_string(),
            target: record.target().to_string(),
            message: record.args().to_string(),
            module: record.module_path().map(|s| s.to_string()),
            file: record.file().map(|s| s.to_string()),
            line: record.line(),
            fields: serde_json::Value::Null,
            trace_id: None,
            span_id: None,
            category: Self::extract_category_from_message(&record.args().to_string()),
            context: None,
        };
        
        // 发送日志到异步处理器
        self.processor.log(entry);
    }
    
    fn flush(&self) {
        self.processor.flush();
    }
}

impl GlobalLoggerAdapter {
    /// 从消息中提取核心问题分类
    fn extract_category_from_message(message: &str) -> Option<CoreLogCategory> {
        if message.contains("[通讯失败]") {
            Some(CoreLogCategory::CommunicationFailure)
        } else if message.contains("[文件解析失败]") {
            Some(CoreLogCategory::FileParsingFailure)
        } else if message.contains("[测试执行失败]") {
            Some(CoreLogCategory::TestExecutionFailure)
        } else if message.contains("[用户操作]") {
            Some(CoreLogCategory::UserOperations)
        } else {
            None
        }
    }
}