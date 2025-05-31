-- 基于原C#项目的数据库表结构分析
-- 提取自 Notes/ChannelMappings_202505301041.sql

-- 通道点位定义表
CREATE TABLE channel_point_definitions (
    id TEXT PRIMARY KEY NOT NULL,
    tag TEXT NOT NULL,                              -- 通道标识
    variable_name TEXT NOT NULL,                    -- 变量名称
    variable_description TEXT,                      -- 变量描述
    station_name TEXT NOT NULL,                     -- 站点名称
    module_name TEXT NOT NULL,                      -- 模块名称
    module_type TEXT NOT NULL,                      -- 模块类型 (AI/AO/DI/DO)
    channel_tag_in_module TEXT NOT NULL,            -- 模块内通道标识
    data_type TEXT NOT NULL,                        -- 数据类型
    power_supply_type TEXT NOT NULL,                -- 供电类型
    wire_system TEXT NOT NULL,                      -- 线制
    plc_absolute_address TEXT,                      -- PLC绝对地址
    plc_communication_address TEXT NOT NULL,        -- PLC通信地址
    range_lower_limit REAL,                         -- 量程下限
    range_upper_limit REAL,                         -- 量程上限
    engineering_unit TEXT,                          -- 工程单位
    
    -- 报警设定值相关字段
    sll_set_value REAL,                            -- 低低报警设定值
    sll_set_point_address TEXT,                    -- 低低报警设定点地址
    sll_set_point_plc_address TEXT,               -- 低低报警设定点PLC地址
    sll_feedback_address TEXT,                     -- 低低报警反馈地址
    sll_feedback_plc_address TEXT,                 -- 低低报警反馈PLC地址
    
    sl_set_value REAL,                             -- 低报警设定值
    sl_set_point_address TEXT,                     -- 低报警设定点地址
    sl_set_point_plc_address TEXT,                 -- 低报警设定点PLC地址
    sl_feedback_address TEXT,                      -- 低报警反馈地址
    sl_feedback_plc_address TEXT,                  -- 低报警反馈PLC地址
    
    sh_set_value REAL,                             -- 高报警设定值
    sh_set_point_address TEXT,                     -- 高报警设定点地址
    sh_set_point_plc_address TEXT,                 -- 高报警设定点PLC地址
    sh_feedback_address TEXT,                      -- 高报警反馈地址
    sh_feedback_plc_address TEXT,                  -- 高报警反馈PLC地址
    
    shh_set_value REAL,                            -- 高高报警设定值
    shh_set_point_address TEXT,                    -- 高高报警设定点地址
    shh_set_point_plc_address TEXT,                -- 高高报警设定点PLC地址
    shh_feedback_address TEXT,                     -- 高高报警反馈地址
    shh_feedback_plc_address TEXT,                 -- 高高报警反馈PLC地址
    
    -- 维护功能相关字段
    maintenance_value_set_point_address TEXT,      -- 维护值设定点地址
    maintenance_value_set_point_plc_address TEXT,  -- 维护值设定点PLC地址
    maintenance_enable_switch_point_address TEXT,  -- 维护使能开关点地址
    maintenance_enable_switch_point_plc_address TEXT, -- 维护使能开关点PLC地址
    
    -- 其他属性
    access_property TEXT,                          -- 访问属性
    save_history BOOLEAN,                          -- 是否保存历史
    power_failure_protection BOOLEAN,             -- 断电保护
    test_rig_plc_address TEXT,                     -- 测试台PLC地址
    
    -- 时间戳
    created_time TEXT NOT NULL,
    updated_time TEXT NOT NULL
);

-- 测试批次信息表
CREATE TABLE test_batch_info (
    batch_id TEXT PRIMARY KEY NOT NULL,
    batch_name TEXT NOT NULL,                      -- 批次名称
    product_model TEXT,                            -- 产品型号
    serial_number TEXT,                            -- 序列号
    customer_name TEXT,                            -- 客户名称
    station_name TEXT,                             -- 站点名称
    created_time TEXT NOT NULL,
    updated_time TEXT NOT NULL,
    start_time TEXT,                               -- 开始时间
    end_time TEXT,                                 -- 结束时间
    total_duration_ms INTEGER,                     -- 总耗时(毫秒)
    operator_name TEXT,                            -- 操作员
    created_by TEXT,                               -- 创建者
    overall_status TEXT NOT NULL,                  -- 整体状态
    status_summary TEXT,                           -- 状态摘要
    error_message TEXT,                            -- 错误信息
    total_points INTEGER DEFAULT 0,               -- 总点数
    tested_points INTEGER DEFAULT 0,              -- 已测试点数
    passed_points INTEGER DEFAULT 0,              -- 通过点数
    failed_points INTEGER DEFAULT 0,              -- 失败点数
    skipped_points INTEGER DEFAULT 0,             -- 跳过点数
    not_tested_points INTEGER DEFAULT 0,          -- 未测试点数
    progress_percentage REAL DEFAULT 0.0,         -- 进度百分比
    current_testing_channel TEXT,                 -- 当前测试通道
    test_configuration TEXT,                      -- 测试配置
    import_source TEXT,                           -- 导入源
    custom_data_json TEXT                         -- 自定义数据(JSON)
);

-- 通道测试实例表
CREATE TABLE channel_test_instances (
    instance_id TEXT PRIMARY KEY NOT NULL,
    definition_id TEXT NOT NULL,                  -- 关联的定义ID
    test_batch_id TEXT NOT NULL,                  -- 关联的批次ID
    test_batch_name TEXT NOT NULL,                -- 批次名称
    channel_tag TEXT NOT NULL,                    -- 通道标识
    variable_name TEXT NOT NULL,                  -- 变量名称
    variable_description TEXT NOT NULL,           -- 变量描述
    module_type TEXT NOT NULL,                    -- 模块类型
    data_type TEXT NOT NULL,                      -- 数据类型
    plc_communication_address TEXT NOT NULL,      -- PLC通信地址
    overall_status TEXT NOT NULL,                 -- 整体状态
    current_step_details TEXT,                    -- 当前步骤详情
    error_message TEXT,                           -- 错误信息
    created_time TEXT NOT NULL,
    start_time TEXT,                              -- 开始时间
    updated_time TEXT NOT NULL,
    final_test_time TEXT,                         -- 最终测试时间
    total_test_duration_ms INTEGER,               -- 总测试耗时
    
    -- 硬点测试相关
    hard_point_status INTEGER,                    -- 硬点状态
    hard_point_test_result TEXT,                  -- 硬点测试结果
    hard_point_error_detail TEXT,                 -- 硬点错误详情
    actual_value TEXT,                            -- 实际值
    expected_value TEXT,                          -- 期望值
    current_value TEXT,                           -- 当前值
    
    -- 报警测试状态
    low_low_alarm_status INTEGER,                 -- 低低报警状态
    low_alarm_status INTEGER,                     -- 低报警状态
    high_alarm_status INTEGER,                    -- 高报警状态
    high_high_alarm_status INTEGER,               -- 高高报警状态
    
    -- 其他测试项状态
    maintenance_function INTEGER,                 -- 维护功能状态
    trend_check INTEGER,                          -- 趋势检查状态
    report_check INTEGER,                         -- 报表检查状态
    show_value_status INTEGER,                    -- 显示值状态
    test_result_status INTEGER,                   -- 测试结果状态
    
    -- 测试PLC相关
    test_plc_channel_tag TEXT,                    -- 测试PLC通道标识
    test_plc_communication_address TEXT,          -- 测试PLC通信地址
    
    -- 操作和重试
    current_operator TEXT,                        -- 当前操作员
    retries_count INTEGER DEFAULT 0,             -- 重试次数
    
    -- JSON数据字段
    sub_test_results_json TEXT,                   -- 子测试结果(JSON)
    hardpoint_readings_json TEXT,                 -- 硬点读数(JSON)
    transient_data_json TEXT,                     -- 瞬态数据(JSON)
    
    -- 外键约束
    FOREIGN KEY (definition_id) REFERENCES channel_point_definitions(id),
    FOREIGN KEY (test_batch_id) REFERENCES test_batch_info(batch_id)
);

-- 原始测试结果表
CREATE TABLE raw_test_outcomes (
    id TEXT PRIMARY KEY NOT NULL,
    channel_instance_id TEXT NOT NULL,           -- 关联的测试实例ID
    sub_test_item TEXT NOT NULL,                 -- 子测试项
    success BOOLEAN NOT NULL,                    -- 是否成功
    message TEXT,                                -- 消息
    execution_time TEXT NOT NULL,                -- 执行时间
    readings_json TEXT,                          -- 读数数据(JSON)
    
    FOREIGN KEY (channel_instance_id) REFERENCES channel_test_instances(instance_id)
);

-- 测试PLC通道配置表
CREATE TABLE test_plc_channel_configs (
    id TEXT PRIMARY KEY NOT NULL,
    channel_address TEXT NOT NULL,               -- 通道地址
    channel_type TEXT NOT NULL,                  -- 通道类型
    data_type TEXT NOT NULL,                     -- 数据类型
    description TEXT,                            -- 描述
    is_enabled BOOLEAN DEFAULT TRUE,             -- 是否启用
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- PLC连接配置表
CREATE TABLE plc_connection_configs (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,                          -- 连接名称
    ip_address TEXT NOT NULL,                    -- IP地址
    port INTEGER NOT NULL,                       -- 端口
    protocol_type TEXT NOT NULL,                 -- 协议类型
    timeout_ms INTEGER DEFAULT 5000,             -- 超时时间
    max_connections INTEGER DEFAULT 1,           -- 最大连接数
    is_test_plc BOOLEAN DEFAULT FALSE,           -- 是否为测试PLC
    is_enabled BOOLEAN DEFAULT TRUE,             -- 是否启用
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 通道映射配置表
CREATE TABLE channel_mapping_configs (
    id TEXT PRIMARY KEY NOT NULL,
    source_channel_id TEXT NOT NULL,             -- 源通道ID
    target_plc_address TEXT NOT NULL,            -- 目标PLC地址
    mapping_type TEXT NOT NULL,                  -- 映射类型
    is_enabled BOOLEAN DEFAULT TRUE,             -- 是否启用
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 创建索引
CREATE INDEX idx_channel_test_instances_batch_id ON channel_test_instances(test_batch_id);
CREATE INDEX idx_channel_test_instances_definition_id ON channel_test_instances(definition_id);
CREATE INDEX idx_raw_test_outcomes_instance_id ON raw_test_outcomes(channel_instance_id);
CREATE INDEX idx_channel_point_definitions_tag ON channel_point_definitions(tag);
CREATE INDEX idx_channel_point_definitions_module_type ON channel_point_definitions(module_type);
