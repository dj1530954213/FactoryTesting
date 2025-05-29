#!/usr/bin/env python3
"""
测试批次分配功能的脚本

这个脚本模拟前端的Excel导入和批次分配过程，
用于验证后端的通道分配服务是否正确生成多个批次。
"""

import json
import time
import requests
from typing import List, Dict, Any

def create_test_channel_definitions(count: int = 20) -> List[Dict[str, Any]]:
    """创建测试用的通道定义"""
    definitions = []
    
    for i in range(count):
        # 交替创建不同类型的通道
        if i % 4 == 0:
            module_type = "AI"
            power_type = "有源" if i % 8 < 4 else "无源"
        elif i % 4 == 1:
            module_type = "AO"
            power_type = "有源" if i % 8 < 4 else "无源"
        elif i % 4 == 2:
            module_type = "DI"
            power_type = "有源" if i % 8 < 4 else "无源"
        else:
            module_type = "DO"
            power_type = "有源" if i % 8 < 4 else "无源"
        
        definition = {
            "id": f"CH_{i+1:03d}",
            "tag": f"Channel_{i+1:03d}",
            "description": f"测试通道{i+1}",
            "station": "Station1",
            "module": f"Module{(i//8)+1}",
            "module_type": module_type,
            "channel": f"CH{(i%8)+1:02d}",
            "data_type": "Float" if module_type in ["AI", "AO"] else "Bool",
            "plc_communication_address": f"DB1.DBD{i*4}",
            "power_supply_type": power_type,
            "range_lower_limit": 0.0 if module_type in ["AI", "AO"] else None,
            "range_upper_limit": 100.0 if module_type in ["AI", "AO"] else None,
            "test_rig_plc_address": f"DB2.DBD{i*4}" if module_type in ["AI", "AO"] else None,
        }
        definitions.append(definition)
    
    return definitions

def create_test_batch_info() -> Dict[str, Any]:
    """创建测试批次信息"""
    return {
        "batch_id": f"test_batch_{int(time.time())}",
        "product_model": "TestProduct_V1.0",
        "serial_number": f"SN{int(time.time())}",
        "operator_name": "测试操作员",
        "total_points": 0,  # 将由后端计算
        "passed_points": 0,
        "failed_points": 0,
        "overall_status": "NotTested",
        "created_at": time.strftime("%Y-%m-%dT%H:%M:%S.000Z"),
        "updated_at": time.strftime("%Y-%m-%dT%H:%M:%S.000Z"),
    }

def test_batch_allocation():
    """测试批次分配功能"""
    print("=== 开始测试批次分配功能 ===")
    
    # 创建测试数据
    channel_definitions = create_test_channel_definitions(20)  # 创建20个通道
    batch_info = create_test_batch_info()
    
    print(f"创建了 {len(channel_definitions)} 个通道定义")
    print(f"批次ID: {batch_info['batch_id']}")
    
    # 显示通道分组统计
    type_stats = {}
    for definition in channel_definitions:
        key = f"{definition['module_type']}_{definition['power_supply_type']}"
        type_stats[key] = type_stats.get(key, 0) + 1
    
    print("\n=== 通道分组统计 ===")
    for key, count in type_stats.items():
        print(f"{key}: {count} 个")
    
    # 创建测试执行请求
    request_data = {
        "batch_info": batch_info,
        "channel_definitions": channel_definitions,
        "max_concurrent_tests": 3,
        "auto_start": False
    }
    
    print("\n=== 模拟提交测试执行请求 ===")
    print(f"请求数据大小: {len(json.dumps(request_data))} 字节")
    
    # 这里我们只是打印请求数据，实际的HTTP请求需要Tauri应用运行
    print("\n=== 预期结果 ===")
    print("根据我们的修复，应该生成多个批次：")
    print("- 每8个通道一个批次（默认配置）")
    print(f"- 总共应该生成 {(len(channel_definitions) + 7) // 8} 个批次")
    print("- 每个批次包含最多8个通道实例")
    
    # 保存测试数据到文件，方便调试
    with open("test_batch_allocation_data.json", "w", encoding="utf-8") as f:
        json.dump(request_data, f, ensure_ascii=False, indent=2)
    
    print(f"\n测试数据已保存到: test_batch_allocation_data.json")
    print("请在Tauri应用中导入此数据进行测试")

if __name__ == "__main__":
    test_batch_allocation() 