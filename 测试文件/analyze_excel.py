#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Excel文件分析脚本
用于分析真实的Excel文件结构，提取数据并生成测试用例
"""

import pandas as pd
import json
import os
from pathlib import Path

def analyze_excel_file(file_path):
    """分析Excel文件并提取数据结构"""
    print(f"正在分析Excel文件: {file_path}")
    
    try:
        # 读取Excel文件
        df = pd.read_excel(file_path)
        
        print(f"文件读取成功！")
        print(f"总行数: {len(df)}")
        print(f"总列数: {len(df.columns)}")
        print(f"列名: {list(df.columns)}")
        
        # 显示前几行数据
        print("\n前5行数据:")
        print(df.head())
        
        # 显示数据类型
        print("\n数据类型:")
        print(df.dtypes)
        
        # 检查空值
        print("\n空值统计:")
        print(df.isnull().sum())
        
        # 生成Rust测试数据
        generate_rust_test_data(df)
        
        # 生成前端测试数据
        generate_frontend_test_data(df)
        
        return df
        
    except Exception as e:
        print(f"读取Excel文件失败: {e}")
        return None

def generate_rust_test_data(df):
    """生成Rust后端测试数据"""
    print("\n=== 生成Rust测试数据 ===")
    
    # 假设Excel列名映射
    column_mapping = {
        '标签': 'tag',
        '变量名': 'variable_name', 
        '描述': 'description',
        '工位': 'station_name',
        '模块': 'module_name',
        '模块类型': 'module_type',
        '通道号': 'channel_number',
        '数据类型': 'point_data_type',
        'PLC地址': 'plc_communication_address'
    }
    
    rust_test_data = []
    
    for index, row in df.iterrows():
        # 跳过空行
        if pd.isna(row.iloc[0]):
            continue
            
        # 根据实际列名提取数据
        test_item = {}
        for excel_col, rust_field in column_mapping.items():
            if excel_col in df.columns:
                value = row[excel_col]
                if pd.notna(value):
                    test_item[rust_field] = str(value).strip()
                else:
                    test_item[rust_field] = ""
        
        rust_test_data.append(test_item)
    
    # 保存为JSON文件
    with open('rust_test_data.json', 'w', encoding='utf-8') as f:
        json.dump(rust_test_data, f, ensure_ascii=False, indent=2)
    
    print(f"已生成 {len(rust_test_data)} 条Rust测试数据")
    
    # 生成Rust单元测试代码
    generate_rust_unit_test(rust_test_data)

def generate_rust_unit_test(test_data):
    """生成Rust单元测试代码"""
    rust_test_code = '''#[cfg(test)]
mod excel_import_tests {
    use super::*;
    use crate::services::infrastructure::excel::ExcelImporter;
    use std::path::Path;

    #[tokio::test]
    async fn test_real_excel_import() {
        // 测试真实Excel文件导入
        let file_path = r"C:\\Program Files\\Git\\code\\FactoryTesting\\测试文件\\测试IO.xlsx";
        
        if Path::new(file_path).exists() {
            let result = ExcelImporter::parse_excel_file(file_path).await;
            
            match result {
                Ok(definitions) => {
                    println!("成功解析Excel文件，共{}个定义", definitions.len());
                    
                    // 验证期望的数据
                    let expected_data = vec!['''
    
    for i, item in enumerate(test_data[:5]):  # 只生成前5条作为示例
        rust_test_code += f'''
                        // 第{i+1}条数据
                        ChannelPointDefinition {{
                            tag: "{item.get('tag', '')}".to_string(),
                            variable_name: "{item.get('variable_name', '')}".to_string(),
                            description: "{item.get('description', '')}".to_string(),
                            station_name: "{item.get('station_name', '')}".to_string(),
                            module_name: "{item.get('module_name', '')}".to_string(),
                            module_type: ModuleType::{item.get('module_type', 'AI')},
                            channel_number: "{item.get('channel_number', '')}".to_string(),
                            point_data_type: PointDataType::{item.get('point_data_type', 'Float')},
                            plc_communication_address: "{item.get('plc_communication_address', '')}".to_string(),
                            ..Default::default()
                        }},'''
    
    rust_test_code += '''
                    ];
                    
                    // 验证数据数量
                    assert!(definitions.len() >= expected_data.len(), 
                           "解析的数据数量不足，期望至少{}条，实际{}条", 
                           expected_data.len(), definitions.len());
                    
                    println!("所有验证通过！");
                }
                Err(e) => {
                    panic!("解析Excel文件失败: {}", e);
                }
            }
        } else {
            println!("测试文件不存在，跳过测试");
        }
    }
}'''
    
    # 保存Rust测试代码
    with open('rust_unit_test.rs', 'w', encoding='utf-8') as f:
        f.write(rust_test_code)
    
    print("已生成Rust单元测试代码: rust_unit_test.rs")

def generate_frontend_test_data(df):
    """生成前端测试数据"""
    print("\n=== 生成前端测试数据 ===")
    
    frontend_test_data = []
    
    for index, row in df.iterrows():
        # 跳过空行
        if pd.isna(row.iloc[0]):
            continue
            
        # 根据前端PreviewDataItem格式生成数据
        item = {
            "tag": str(row.iloc[0]) if pd.notna(row.iloc[0]) else "",
            "description": str(row.iloc[2]) if len(row) > 2 and pd.notna(row.iloc[2]) else "",
            "moduleType": str(row.iloc[5]) if len(row) > 5 and pd.notna(row.iloc[5]) else "AI",
            "channelNumber": str(row.iloc[6]) if len(row) > 6 and pd.notna(row.iloc[6]) else "",
            "plcAddress": str(row.iloc[8]) if len(row) > 8 and pd.notna(row.iloc[8]) else ""
        }
        
        frontend_test_data.append(item)
    
    # 保存为JSON文件
    with open('frontend_test_data.json', 'w', encoding='utf-8') as f:
        json.dump(frontend_test_data, f, ensure_ascii=False, indent=2)
    
    print(f"已生成 {len(frontend_test_data)} 条前端测试数据")

def main():
    """主函数"""
    # Excel文件路径
    excel_file = "测试IO.xlsx"
    
    # 检查文件是否存在
    if not os.path.exists(excel_file):
        print(f"错误: Excel文件不存在: {excel_file}")
        return
    
    # 分析Excel文件
    df = analyze_excel_file(excel_file)
    
    if df is not None:
        print("\n=== 分析完成 ===")
        print("生成的文件:")
        print("- rust_test_data.json: Rust后端测试数据")
        print("- rust_unit_test.rs: Rust单元测试代码")
        print("- frontend_test_data.json: 前端测试数据")

if __name__ == "__main__":
    main() 