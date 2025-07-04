//! 系统监控模块
//!
//! 提供系统性能监控、业务指标监控和健康检查功能

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 系统性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: MemoryUsage,
    pub disk_usage: DiskUsage,
    pub network_stats: NetworkStats,
}

/// 内存使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub usage_percent: f64,
}

/// 磁盘使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub total_gb: u64,
    pub used_gb: u64,
    pub available_gb: u64,
    pub usage_percent: f64,
}

/// 网络统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// 业务指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub timestamp: u64,
    pub total_tests_run: u64,
    pub successful_tests: u64,
    pub failed_tests: u64,
    pub success_rate: f64,
    pub average_test_duration: Duration,
    pub active_batches: u32,
    pub plc_connections: u32,
}

/// 健康检查状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub response_time: Duration,
}

/// 系统监控器
pub struct SystemMonitor {
    metrics_history: Arc<RwLock<Vec<SystemMetrics>>>,
    business_metrics: Arc<RwLock<BusinessMetrics>>,
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    max_history_size: usize,
}

impl SystemMonitor {
    /// 创建新的系统监控器
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            business_metrics: Arc::new(RwLock::new(BusinessMetrics {
                timestamp: current_timestamp(),
                total_tests_run: 0,
                successful_tests: 0,
                failed_tests: 0,
                success_rate: 0.0,
                average_test_duration: Duration::from_secs(0),
                active_batches: 0,
                plc_connections: 0,
            })),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            max_history_size: 1000, // 保留最近1000个数据点
        }
    }
    
    /// 收集系统指标
    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics, Box<dyn std::error::Error>> {
        let timestamp = current_timestamp();
        
        // 获取CPU使用率
        let cpu_usage = self.get_cpu_usage().await?;
        
        // 获取内存使用情况
        let memory_usage = self.get_memory_usage().await?;
        
        // 获取磁盘使用情况
        let disk_usage = self.get_disk_usage().await?;
        
        // 获取网络统计
        let network_stats = self.get_network_stats().await?;
        
        let metrics = SystemMetrics {
            timestamp,
            cpu_usage,
            memory_usage,
            disk_usage,
            network_stats,
        };
        
        // 存储到历史记录
        let mut history = self.metrics_history.write().await;
        history.push(metrics.clone());
        
        // 限制历史记录大小
        if history.len() > self.max_history_size {
            history.remove(0);
        }
        
        Ok(metrics)
    }
    
    /// 更新业务指标
    pub async fn update_business_metrics(
        &self,
        test_completed: bool,
        test_success: bool,
        test_duration: Duration,
        active_batches: u32,
        plc_connections: u32,
    ) {
        let mut metrics = self.business_metrics.write().await;
        
        metrics.timestamp = current_timestamp();
        
        if test_completed {
            metrics.total_tests_run += 1;
            if test_success {
                metrics.successful_tests += 1;
            } else {
                metrics.failed_tests += 1;
            }
            
            // 更新成功率
            metrics.success_rate = if metrics.total_tests_run > 0 {
                metrics.successful_tests as f64 / metrics.total_tests_run as f64 * 100.0
            } else {
                0.0
            };
            
            // 更新平均测试时间（简单移动平均）
            let current_avg_ms = metrics.average_test_duration.as_millis() as f64;
            let new_duration_ms = test_duration.as_millis() as f64;
            let total_tests = metrics.total_tests_run as f64;
            
            let new_avg_ms = (current_avg_ms * (total_tests - 1.0) + new_duration_ms) / total_tests;
            metrics.average_test_duration = Duration::from_millis(new_avg_ms as u64);
        }
        
        metrics.active_batches = active_batches;
        metrics.plc_connections = plc_connections;
    }
    
    /// 执行健康检查
    pub async fn perform_health_check(&self, component: &str) -> HealthCheck {
        let start_time = Instant::now();
        
        let (status, message) = match component {
            "database" => self.check_database_health().await,
            "plc_communication" => self.check_plc_health().await,
            "memory" => self.check_memory_health().await,
            "disk" => self.check_disk_health().await,
            _ => (HealthStatus::Unknown, "未知组件".to_string()),
        };
        
        let response_time = start_time.elapsed();
        
        let health_check = HealthCheck {
            component: component.to_string(),
            status,
            message,
            timestamp: current_timestamp(),
            response_time,
        };
        
        // 存储健康检查结果
        let mut checks = self.health_checks.write().await;
        checks.insert(component.to_string(), health_check.clone());
        
        health_check
    }
    
    /// 获取所有健康检查结果
    pub async fn get_all_health_checks(&self) -> HashMap<String, HealthCheck> {
        self.health_checks.read().await.clone()
    }
    
    /// 获取系统指标历史
    pub async fn get_metrics_history(&self, limit: Option<usize>) -> Vec<SystemMetrics> {
        let history = self.metrics_history.read().await;
        
        if let Some(limit) = limit {
            let start = if history.len() > limit {
                history.len() - limit
            } else {
                0
            };
            history[start..].to_vec()
        } else {
            history.clone()
        }
    }
    
    /// 获取当前业务指标
    pub async fn get_business_metrics(&self) -> BusinessMetrics {
        self.business_metrics.read().await.clone()
    }
    
    // 私有方法：获取CPU使用率
    async fn get_cpu_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 这里应该使用系统API获取真实的CPU使用率
        // 为了演示，返回模拟数据
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(10.0..50.0))
    }
    
    // 私有方法：获取内存使用情况
    async fn get_memory_usage(&self) -> Result<MemoryUsage, Box<dyn std::error::Error>> {
        // 这里应该使用系统API获取真实的内存信息
        // 为了演示，返回模拟数据
        let total_mb = 8192; // 8GB
        let used_mb = 3072;  // 3GB
        let available_mb = total_mb - used_mb;
        let usage_percent = used_mb as f64 / total_mb as f64 * 100.0;
        
        Ok(MemoryUsage {
            total_mb,
            used_mb,
            available_mb,
            usage_percent,
        })
    }
    
    // 私有方法：获取磁盘使用情况
    async fn get_disk_usage(&self) -> Result<DiskUsage, Box<dyn std::error::Error>> {
        // 这里应该使用系统API获取真实的磁盘信息
        let total_gb = 500;
        let used_gb = 200;
        let available_gb = total_gb - used_gb;
        let usage_percent = used_gb as f64 / total_gb as f64 * 100.0;
        
        Ok(DiskUsage {
            total_gb,
            used_gb,
            available_gb,
            usage_percent,
        })
    }
    
    // 私有方法：获取网络统计
    async fn get_network_stats(&self) -> Result<NetworkStats, Box<dyn std::error::Error>> {
        // 这里应该使用系统API获取真实的网络统计
        Ok(NetworkStats {
            bytes_sent: 1024 * 1024 * 100,     // 100MB
            bytes_received: 1024 * 1024 * 50,  // 50MB
            packets_sent: 10000,
            packets_received: 8000,
        })
    }
    
    // 私有方法：检查数据库健康状态
    async fn check_database_health(&self) -> (HealthStatus, String) {
        // 这里应该实际检查数据库连接
        (HealthStatus::Healthy, "数据库连接正常".to_string())
    }
    
    // 私有方法：检查PLC通信健康状态
    async fn check_plc_health(&self) -> (HealthStatus, String) {
        // 这里应该实际检查PLC连接
        (HealthStatus::Healthy, "PLC通信正常".to_string())
    }
    
    // 私有方法：检查内存健康状态
    async fn check_memory_health(&self) -> (HealthStatus, String) {
        if let Ok(memory) = self.get_memory_usage().await {
            if memory.usage_percent > 90.0 {
                (HealthStatus::Critical, format!("内存使用率过高: {:.1}%", memory.usage_percent))
            } else if memory.usage_percent > 80.0 {
                (HealthStatus::Warning, format!("内存使用率较高: {:.1}%", memory.usage_percent))
            } else {
                (HealthStatus::Healthy, format!("内存使用正常: {:.1}%", memory.usage_percent))
            }
        } else {
            (HealthStatus::Unknown, "无法获取内存信息".to_string())
        }
    }
    
    // 私有方法：检查磁盘健康状态
    async fn check_disk_health(&self) -> (HealthStatus, String) {
        if let Ok(disk) = self.get_disk_usage().await {
            if disk.usage_percent > 95.0 {
                (HealthStatus::Critical, format!("磁盘空间不足: {:.1}%", disk.usage_percent))
            } else if disk.usage_percent > 85.0 {
                (HealthStatus::Warning, format!("磁盘空间较少: {:.1}%", disk.usage_percent))
            } else {
                (HealthStatus::Healthy, format!("磁盘空间充足: {:.1}%", disk.usage_percent))
            }
        } else {
            (HealthStatus::Unknown, "无法获取磁盘信息".to_string())
        }
    }
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

