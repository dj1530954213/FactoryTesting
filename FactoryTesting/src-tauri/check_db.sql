-- 检查数据库内容的SQL脚本

-- 检查 channel_point_definitions 表
.print "=== 检查 channel_point_definitions 表 ==="
SELECT COUNT(*) as '总记录数' FROM channel_point_definitions;

.print ""
.print "前5条记录:"
SELECT 
    substr(id, 1, 8) as 'ID前8位',
    tag as '标签',
    variable_name as '变量名',
    module_type as '模块类型',
    power_supply_type as '供电类型',
    channel_tag_in_module as '通道标签'
FROM channel_point_definitions 
LIMIT 5;

-- 检查 test_batch_info 表
.print ""
.print "=== 检查 test_batch_info 表 ==="
SELECT COUNT(*) as '总记录数' FROM test_batch_info;

.print ""
.print "所有批次记录:"
SELECT 
    substr(batch_id, 1, 20) as '批次ID前20位',
    batch_name as '批次名称',
    total_points as '总点位',
    created_at as '创建时间'
FROM test_batch_info;

-- 检查 channel_test_instances 表
.print ""
.print "=== 检查 channel_test_instances 表 ==="
SELECT COUNT(*) as '总记录数' FROM channel_test_instances;

.print ""
.print "前5条测试实例记录:"
SELECT 
    substr(instance_id, 1, 8) as '实例ID前8位',
    substr(definition_id, 1, 8) as '定义ID前8位',
    substr(test_batch_id, 1, 20) as '批次ID前20位',
    overall_status as '状态',
    assigned_test_plc_channel as 'PLC通道'
FROM channel_test_instances 
LIMIT 5;
