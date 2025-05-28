#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
分析正确的通道分配Excel文件
用于理解真实的批次分配逻辑
"""

import pandas as pd
import json
import os
from pathlib import Path

def analyze_correct_allocation():
    """分析正确的通道分配Excel文件"""
    excel_file = "正确的通道分配.xlsx"
    
    if not os.path.exists(excel_file):
        print(f"错误: Excel文件不存在: {excel_file}")
        return
    
    print(f"正在分析正确的通道分配文件: {excel_file}")
    
    try:
        # 读取Excel文件的所有工作表
        excel_data = pd.ExcelFile(excel_file)
        print(f"工作表列表: {excel_data.sheet_names}")
        
        # 分析每个工作表
        for sheet_name in excel_data.sheet_names:
            print(f"\n=== 分析工作表: {sheet_name} ===")
            df = pd.read_excel(excel_file, sheet_name=sheet_name)
            
            print(f"行数: {len(df)}")
            print(f"列数: {len(df.columns)}")
            print(f"列名: {list(df.columns)}")
            
            # 显示前几行数据
            print("\n前5行数据:")
            print(df.head())
            
            # 如果是批次相关的工作表，进行详细分析
            if '批次' in sheet_name or 'batch' in sheet_name.lower():
                analyze_batch_allocation(df, sheet_name)
            
            # 如果是通道映射相关的工作表
            if '通道' in sheet_name or 'channel' in sheet_name.lower() or '映射' in sheet_name:
                analyze_channel_mapping(df, sheet_name)
    
    except Exception as e:
        print(f"分析Excel文件失败: {e}")

def analyze_batch_allocation(df, sheet_name):
    """分析批次分配逻辑"""
    print(f"\n--- 批次分配分析 ({sheet_name}) ---")
    
    # 检查是否有批次相关的列
    batch_columns = [col for col in df.columns if '批次' in str(col) or 'batch' in str(col).lower()]
    if batch_columns:
        print(f"批次相关列: {batch_columns}")
        
        for col in batch_columns:
            unique_values = df[col].unique()
            print(f"列 '{col}' 的唯一值: {unique_values}")
            
            # 统计每个批次的数量
            value_counts = df[col].value_counts()
            print(f"批次分布:")
            for value, count in value_counts.items():
                print(f"  {value}: {count}个")
    
    # 检查模块类型分布
    module_type_columns = [col for col in df.columns if '模块' in str(col) or 'module' in str(col).lower() or '类型' in str(col)]
    if module_type_columns:
        print(f"\n模块类型相关列: {module_type_columns}")
        
        for col in module_type_columns:
            if df[col].notna().any():
                module_counts = df[col].value_counts()
                print(f"模块类型分布 ({col}):")
                for module_type, count in module_counts.items():
                    print(f"  {module_type}: {count}个")

def analyze_channel_mapping(df, sheet_name):
    """分析通道映射逻辑"""
    print(f"\n--- 通道映射分析 ({sheet_name}) ---")
    
    # 检查测试PLC通道相关的列
    test_plc_columns = [col for col in df.columns if '测试' in str(col) and 'PLC' in str(col)]
    if test_plc_columns:
        print(f"测试PLC相关列: {test_plc_columns}")
        
        for col in test_plc_columns:
            if df[col].notna().any():
                unique_values = df[col].unique()
                print(f"列 '{col}' 的唯一值 (前10个): {unique_values[:10]}")
    
    # 检查被测PLC通道相关的列
    target_plc_columns = [col for col in df.columns if '被测' in str(col) or 'PLC' in str(col)]
    if target_plc_columns:
        print(f"被测PLC相关列: {target_plc_columns}")
        
        for col in target_plc_columns:
            if df[col].notna().any():
                unique_values = df[col].unique()
                print(f"列 '{col}' 的唯一值 (前10个): {unique_values[:10]}")
    
    # 检查点位名称相关的列
    tag_columns = [col for col in df.columns if '点位' in str(col) or '位号' in str(col) or 'tag' in str(col).lower()]
    if tag_columns:
        print(f"点位相关列: {tag_columns}")
        
        for col in tag_columns:
            if df[col].notna().any():
                sample_values = df[col].dropna().head(10).tolist()
                print(f"列 '{col}' 示例值: {sample_values}")

def generate_correct_allocation_data():
    """生成正确的分配数据用于Rust后端"""
    excel_file = "正确的通道分配.xlsx"
    
    if not os.path.exists(excel_file):
        print("无法生成数据：Excel文件不存在")
        return
    
    try:
        # 读取主要的分配数据
        df = pd.read_excel(excel_file, sheet_name=0)  # 读取第一个工作表
        
        allocation_data = []
        batch_summary = {}
        
        for index, row in df.iterrows():
            # 跳过空行或标题行
            if pd.isna(row.iloc[0]) or index == 0:
                continue
            
            # 提取分配信息
            allocation_item = {}
            
            # 根据实际列名提取数据
            for col_idx, col_name in enumerate(df.columns):
                if col_idx < len(row):
                    value = row.iloc[col_idx]
                    if pd.notna(value):
                        allocation_item[col_name] = str(value).strip()
            
            allocation_data.append(allocation_item)
        
        # 保存分配数据
        with open('correct_allocation_data.json', 'w', encoding='utf-8') as f:
            json.dump(allocation_data, f, ensure_ascii=False, indent=2)
        
        print(f"已生成正确的分配数据: {len(allocation_data)} 条记录")
        
        # 分析批次分布
        if allocation_data:
            print("\n批次分布分析:")
            batch_counts = {}
            module_type_counts = {}
            
            for item in allocation_data:
                # 查找批次信息
                batch_key = None
                for key in item.keys():
                    if '批次' in key or 'batch' in key.lower():
                        batch_key = key
                        break
                
                if batch_key and item[batch_key]:
                    batch_id = item[batch_key]
                    batch_counts[batch_id] = batch_counts.get(batch_id, 0) + 1
                
                # 查找模块类型信息
                module_key = None
                for key in item.keys():
                    if '模块' in key and '类型' in key:
                        module_key = key
                        break
                
                if module_key and item[module_key]:
                    module_type = item[module_key]
                    module_type_counts[module_type] = module_type_counts.get(module_type, 0) + 1
            
            print("批次分布:")
            for batch_id, count in sorted(batch_counts.items()):
                print(f"  {batch_id}: {count}个通道")
            
            print("\n模块类型分布:")
            for module_type, count in sorted(module_type_counts.items()):
                print(f"  {module_type}: {count}个通道")
        
    except Exception as e:
        print(f"生成分配数据失败: {e}")

def main():
    """主函数"""
    print("=== 正确的通道分配分析 ===")
    
    # 分析Excel文件
    analyze_correct_allocation()
    
    print("\n" + "="*50)
    
    # 生成正确的分配数据
    generate_correct_allocation_data()

if __name__ == "__main__":
    main() 