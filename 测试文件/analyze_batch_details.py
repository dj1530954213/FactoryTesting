#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
分析批次分配的详细规律
"""

import json

def analyze_batch_details():
    """分析批次分配详细规律"""
    with open('correct_allocation_data.json', 'r', encoding='utf-8') as f:
        data = json.load(f)

    print('=== 批次分配详细分析 ===')
    batch1 = [item for item in data if item.get('测试批次') == '批次1']
    batch2 = [item for item in data if item.get('测试批次') == '批次2']

    print(f'批次1: {len(batch1)}个通道')
    print(f'批次2: {len(batch2)}个通道')

    print('\n=== 批次1模块类型分布 ===')
    batch1_modules = {}
    for item in batch1:
        module_type = item.get('模块类型', 'Unknown')
        batch1_modules[module_type] = batch1_modules.get(module_type, 0) + 1
    for module, count in sorted(batch1_modules.items()):
        print(f'{module}: {count}个')

    print('\n=== 批次2模块类型分布 ===')
    batch2_modules = {}
    for item in batch2:
        module_type = item.get('模块类型', 'Unknown')
        batch2_modules[module_type] = batch2_modules.get(module_type, 0) + 1
    for module, count in sorted(batch2_modules.items()):
        print(f'{module}: {count}个')

    print('\n=== 测试PLC通道分配规律 ===')
    print('批次1前10个通道的测试PLC通道:')
    for i, item in enumerate(batch1[:10]):
        print(f'  {item.get("变量名称", "N/A")} ({item.get("模块类型", "N/A")}) -> {item.get("测试PLC通道位号", "N/A")}')

    print('\n批次2前10个通道的测试PLC通道:')
    for i, item in enumerate(batch2[:10]):
        print(f'  {item.get("变量名称", "N/A")} ({item.get("模块类型", "N/A")}) -> {item.get("测试PLC通道位号", "N/A")}')

    # 分析测试PLC通道的使用情况
    print('\n=== 测试PLC通道使用分析 ===')
    test_plc_channels = {}
    for item in data:
        test_channel = item.get('测试PLC通道位号', 'N/A')
        batch = item.get('测试批次', 'N/A')
        module_type = item.get('模块类型', 'N/A')
        
        if test_channel not in test_plc_channels:
            test_plc_channels[test_channel] = []
        test_plc_channels[test_channel].append({
            'batch': batch,
            'module_type': module_type,
            'variable': item.get('变量名称', 'N/A')
        })

    # 显示测试PLC通道的重用情况
    print('测试PLC通道重用情况:')
    for channel, usages in sorted(test_plc_channels.items()):
        if len(usages) > 1:
            print(f'  {channel}: 被{len(usages)}个通道使用')
            for usage in usages:
                print(f'    - {usage["variable"]} ({usage["module_type"]}, {usage["batch"]})')

if __name__ == "__main__":
    analyze_batch_details() 