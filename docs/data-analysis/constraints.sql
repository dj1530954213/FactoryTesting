-- 数据库约束条件和业务规则
-- 基于原C#项目的业务逻辑分析

-- ============================================================================
-- 主键约束 (PRIMARY KEY)
-- ============================================================================

-- 通道点位定义表主键
ALTER TABLE channel_point_definitions 
ADD CONSTRAINT pk_channel_point_definitions PRIMARY KEY (id);

-- 测试批次信息表主键
ALTER TABLE test_batch_info 
ADD CONSTRAINT pk_test_batch_info PRIMARY KEY (batch_id);

-- 通道测试实例表主键
ALTER TABLE channel_test_instances 
ADD CONSTRAINT pk_channel_test_instances PRIMARY KEY (instance_id);

-- 原始测试结果表主键
ALTER TABLE raw_test_outcomes 
ADD CONSTRAINT pk_raw_test_outcomes PRIMARY KEY (id);

-- 测试PLC通道配置表主键
ALTER TABLE test_plc_channel_configs 
ADD CONSTRAINT pk_test_plc_channel_configs PRIMARY KEY (id);

-- PLC连接配置表主键
ALTER TABLE plc_connection_configs 
ADD CONSTRAINT pk_plc_connection_configs PRIMARY KEY (id);

-- 通道映射配置表主键
ALTER TABLE channel_mapping_configs 
ADD CONSTRAINT pk_channel_mapping_configs PRIMARY KEY (id);

-- ============================================================================
-- 外键约束 (FOREIGN KEY)
-- ============================================================================

-- 测试实例表外键约束
ALTER TABLE channel_test_instances 
ADD CONSTRAINT fk_instances_definition 
FOREIGN KEY (definition_id) REFERENCES channel_point_definitions(id)
ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE channel_test_instances 
ADD CONSTRAINT fk_instances_batch 
FOREIGN KEY (test_batch_id) REFERENCES test_batch_info(batch_id)
ON DELETE CASCADE ON UPDATE CASCADE;

-- 测试结果表外键约束
ALTER TABLE raw_test_outcomes 
ADD CONSTRAINT fk_outcomes_instance 
FOREIGN KEY (channel_instance_id) REFERENCES channel_test_instances(instance_id)
ON DELETE CASCADE ON UPDATE CASCADE;

-- 通道映射配置表外键约束
ALTER TABLE channel_mapping_configs 
ADD CONSTRAINT fk_mapping_source_channel 
FOREIGN KEY (source_channel_id) REFERENCES test_plc_channel_configs(id)
ON DELETE CASCADE ON UPDATE CASCADE;

-- ============================================================================
-- 唯一性约束 (UNIQUE)
-- ============================================================================

-- 通道定义表唯一约束
ALTER TABLE channel_point_definitions 
ADD CONSTRAINT uk_channel_definitions_tag UNIQUE (tag);

-- 批次信息表唯一约束
ALTER TABLE test_batch_info 
ADD CONSTRAINT uk_test_batch_info_name UNIQUE (batch_name);

-- PLC连接配置表唯一约束
ALTER TABLE plc_connection_configs 
ADD CONSTRAINT uk_plc_connections_name UNIQUE (name);

-- 测试PLC通道配置表唯一约束
ALTER TABLE test_plc_channel_configs 
ADD CONSTRAINT uk_test_plc_channels_address UNIQUE (channel_address);

-- ============================================================================
-- 非空约束 (NOT NULL)
-- ============================================================================

-- 通道点位定义表必填字段
ALTER TABLE channel_point_definitions 
ALTER COLUMN tag SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN variable_name SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN station_name SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN module_name SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN module_type SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN data_type SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN plc_communication_address SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN created_time SET NOT NULL;
ALTER TABLE channel_point_definitions 
ALTER COLUMN updated_time SET NOT NULL;

-- 测试批次信息表必填字段
ALTER TABLE test_batch_info 
ALTER COLUMN batch_name SET NOT NULL;
ALTER TABLE test_batch_info 
ALTER COLUMN overall_status SET NOT NULL;
ALTER TABLE test_batch_info 
ALTER COLUMN created_time SET NOT NULL;
ALTER TABLE test_batch_info 
ALTER COLUMN updated_time SET NOT NULL;

-- 测试实例表必填字段
ALTER TABLE channel_test_instances 
ALTER COLUMN definition_id SET NOT NULL;
ALTER TABLE channel_test_instances 
ALTER COLUMN test_batch_id SET NOT NULL;
ALTER TABLE channel_test_instances 
ALTER COLUMN overall_status SET NOT NULL;
ALTER TABLE channel_test_instances 
ALTER COLUMN created_time SET NOT NULL;
ALTER TABLE channel_test_instances 
ALTER COLUMN updated_time SET NOT NULL;

-- ============================================================================
-- 检查约束 (CHECK)
-- ============================================================================

-- 通道定义表检查约束
ALTER TABLE channel_point_definitions 
ADD CONSTRAINT chk_definition_tag_format 
CHECK (LENGTH(tag) > 0 AND tag NOT LIKE '% %'); -- 标签不能为空且不能包含空格

ALTER TABLE channel_point_definitions 
ADD CONSTRAINT chk_definition_module_type 
CHECK (module_type IN ('AI', 'AO', 'DI', 'DO', 'AINone', 'AONone', 'DINone', 'DONone', 'Communication'));

ALTER TABLE channel_point_definitions 
ADD CONSTRAINT chk_definition_data_type 
CHECK (data_type IN ('Bool', 'Float', 'Double', 'Int', 'Int16', 'Int32', 'UInt16', 'UInt32', 'String'));

ALTER TABLE channel_point_definitions 
ADD CONSTRAINT chk_definition_range_valid 
CHECK (range_lower_limit IS NULL OR range_upper_limit IS NULL OR range_lower_limit < range_upper_limit);

-- 测试批次信息表检查约束
ALTER TABLE test_batch_info 
ADD CONSTRAINT chk_batch_points_non_negative 
CHECK (total_points >= 0 AND tested_points >= 0 AND passed_points >= 0 AND failed_points >= 0 AND skipped_points >= 0 AND not_tested_points >= 0);

ALTER TABLE test_batch_info 
ADD CONSTRAINT chk_batch_points_consistency 
CHECK (tested_points + not_tested_points = total_points);

ALTER TABLE test_batch_info 
ADD CONSTRAINT chk_batch_tested_breakdown 
CHECK (passed_points + failed_points + skipped_points = tested_points);

ALTER TABLE test_batch_info 
ADD CONSTRAINT chk_batch_progress_range 
CHECK (progress_percentage >= 0.0 AND progress_percentage <= 100.0);

ALTER TABLE test_batch_info 
ADD CONSTRAINT chk_batch_status_valid 
CHECK (overall_status IN ('NotTested', 'InProgress', 'Completed', 'Failed', 'Cancelled'));

ALTER TABLE test_batch_info 
ADD CONSTRAINT chk_batch_time_sequence 
CHECK (start_time IS NULL OR end_time IS NULL OR start_time <= end_time);

-- 测试实例表检查约束
ALTER TABLE channel_test_instances 
ADD CONSTRAINT chk_instance_status_valid 
CHECK (overall_status IN ('NotTested', 'Skipped', 'WiringConfirmationRequired', 'WiringConfirmed', 
                         'HardPointTestInProgress', 'HardPointTesting', 'HardPointTestCompleted',
                         'ManualTestInProgress', 'ManualTesting', 'TestCompletedPassed', 
                         'TestCompletedFailed', 'Retesting'));

ALTER TABLE channel_test_instances 
ADD CONSTRAINT chk_instance_retries_non_negative 
CHECK (retries_count >= 0);

ALTER TABLE channel_test_instances 
ADD CONSTRAINT chk_instance_duration_positive 
CHECK (total_test_duration_ms IS NULL OR total_test_duration_ms >= 0);

ALTER TABLE channel_test_instances 
ADD CONSTRAINT chk_instance_time_sequence 
CHECK (start_time IS NULL OR final_test_time IS NULL OR start_time <= final_test_time);

-- 测试结果表检查约束
ALTER TABLE raw_test_outcomes 
ADD CONSTRAINT chk_outcome_sub_test_item_valid 
CHECK (sub_test_item IN ('HardPoint', 'TrendCheck', 'Trend', 'ReportCheck', 'Report', 'Maintenance',
                        'LowLowAlarm', 'LowAlarm', 'HighAlarm', 'HighHighAlarm', 'AlarmValueSetting',
                        'MaintenanceFunction', 'StateDisplay', 'Output0Percent', 'Output25Percent',
                        'Output50Percent', 'Output75Percent', 'Output100Percent', 'CommunicationTest'));

-- PLC连接配置表检查约束
ALTER TABLE plc_connection_configs 
ADD CONSTRAINT chk_plc_connection_port_range 
CHECK (port > 0 AND port <= 65535);

ALTER TABLE plc_connection_configs 
ADD CONSTRAINT chk_plc_connection_timeout_positive 
CHECK (timeout_ms > 0);

ALTER TABLE plc_connection_configs 
ADD CONSTRAINT chk_plc_connection_max_connections_positive 
CHECK (max_connections > 0);

ALTER TABLE plc_connection_configs 
ADD CONSTRAINT chk_plc_connection_protocol_valid 
CHECK (protocol_type IN ('ModbusTCP', 'S7', 'OPCUA', 'EthernetIP'));

-- ============================================================================
-- 默认值约束 (DEFAULT)
-- ============================================================================

-- 通用默认值
ALTER TABLE channel_point_definitions 
ALTER COLUMN created_time SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE channel_point_definitions 
ALTER COLUMN updated_time SET DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE test_batch_info 
ALTER COLUMN created_time SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE test_batch_info 
ALTER COLUMN updated_time SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE test_batch_info 
ALTER COLUMN overall_status SET DEFAULT 'NotTested';
ALTER TABLE test_batch_info 
ALTER COLUMN total_points SET DEFAULT 0;
ALTER TABLE test_batch_info 
ALTER COLUMN tested_points SET DEFAULT 0;
ALTER TABLE test_batch_info 
ALTER COLUMN passed_points SET DEFAULT 0;
ALTER TABLE test_batch_info 
ALTER COLUMN failed_points SET DEFAULT 0;
ALTER TABLE test_batch_info 
ALTER COLUMN skipped_points SET DEFAULT 0;
ALTER TABLE test_batch_info 
ALTER COLUMN not_tested_points SET DEFAULT 0;
ALTER TABLE test_batch_info 
ALTER COLUMN progress_percentage SET DEFAULT 0.0;

ALTER TABLE channel_test_instances 
ALTER COLUMN created_time SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE channel_test_instances 
ALTER COLUMN updated_time SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE channel_test_instances 
ALTER COLUMN overall_status SET DEFAULT 'NotTested';
ALTER TABLE channel_test_instances 
ALTER COLUMN retries_count SET DEFAULT 0;

ALTER TABLE raw_test_outcomes 
ALTER COLUMN execution_time SET DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE test_plc_channel_configs 
ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE test_plc_channel_configs 
ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE test_plc_channel_configs 
ALTER COLUMN is_enabled SET DEFAULT TRUE;

ALTER TABLE plc_connection_configs 
ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE plc_connection_configs 
ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE plc_connection_configs 
ALTER COLUMN timeout_ms SET DEFAULT 5000;
ALTER TABLE plc_connection_configs 
ALTER COLUMN max_connections SET DEFAULT 1;
ALTER TABLE plc_connection_configs 
ALTER COLUMN is_test_plc SET DEFAULT FALSE;
ALTER TABLE plc_connection_configs 
ALTER COLUMN is_enabled SET DEFAULT TRUE;

ALTER TABLE channel_mapping_configs 
ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE channel_mapping_configs 
ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE channel_mapping_configs 
ALTER COLUMN is_enabled SET DEFAULT TRUE;

-- ============================================================================
-- 触发器约束 (TRIGGER)
-- ============================================================================

-- 自动更新时间戳触发器
CREATE TRIGGER trg_channel_point_definitions_updated_time
    BEFORE UPDATE ON channel_point_definitions
    FOR EACH ROW
    SET NEW.updated_time = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_test_batch_info_updated_time
    BEFORE UPDATE ON test_batch_info
    FOR EACH ROW
    SET NEW.updated_time = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_channel_test_instances_updated_time
    BEFORE UPDATE ON channel_test_instances
    FOR EACH ROW
    SET NEW.updated_time = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_test_plc_channel_configs_updated_at
    BEFORE UPDATE ON test_plc_channel_configs
    FOR EACH ROW
    SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_plc_connection_configs_updated_at
    BEFORE UPDATE ON plc_connection_configs
    FOR EACH ROW
    SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_channel_mapping_configs_updated_at
    BEFORE UPDATE ON channel_mapping_configs
    FOR EACH ROW
    SET NEW.updated_at = CURRENT_TIMESTAMP;

-- 业务逻辑触发器
CREATE TRIGGER trg_test_batch_progress_update
    AFTER UPDATE ON channel_test_instances
    FOR EACH ROW
    WHEN NEW.overall_status != OLD.overall_status
    BEGIN
        UPDATE test_batch_info 
        SET 
            tested_points = (
                SELECT COUNT(*) FROM channel_test_instances 
                WHERE test_batch_id = NEW.test_batch_id 
                AND overall_status IN ('TestCompletedPassed', 'TestCompletedFailed')
            ),
            passed_points = (
                SELECT COUNT(*) FROM channel_test_instances 
                WHERE test_batch_id = NEW.test_batch_id 
                AND overall_status = 'TestCompletedPassed'
            ),
            failed_points = (
                SELECT COUNT(*) FROM channel_test_instances 
                WHERE test_batch_id = NEW.test_batch_id 
                AND overall_status = 'TestCompletedFailed'
            ),
            progress_percentage = (
                SELECT 
                    CASE 
                        WHEN total_points = 0 THEN 0.0
                        ELSE (COUNT(*) * 100.0 / total_points)
                    END
                FROM channel_test_instances 
                WHERE test_batch_id = NEW.test_batch_id 
                AND overall_status IN ('TestCompletedPassed', 'TestCompletedFailed')
            )
        WHERE batch_id = NEW.test_batch_id;
    END;
