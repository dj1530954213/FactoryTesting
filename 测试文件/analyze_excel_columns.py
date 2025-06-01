#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
分析Excel文件的列结构，找出为什么只解析了80个点位而不是88个
"""

import pandas as pd
import sys
import os

def analyze_excel_file(file_path):
    """分析Excel文件的结构"""
    print(f"分析Excel文件: {file_path}")
    
    if not os.path.exists(file_path):
        print(f"文件不存在: {file_path}")
        return
    
    try:
        # 读取Excel文件
        df = pd.read_excel(file_path, sheet_name=0)
        
        print(f"\n基本信息:")
        print(f"  总行数: {len(df)}")
        print(f"  总列数: {len(df.columns)}")
        print(f"  列名: {list(df.columns)}")
        
        # 检查每行的非空列数
        print(f"\n每行的非空列数分析:")
        for i, row in df.iterrows():
            non_null_count = row.notna().sum()
            total_cols = len(row)
            print(f"  第{i+2}行: {non_null_count}/{total_cols} 列有数据")
            
            # 如果列数少于53，显示详细信息
            if total_cols < 53:
                print(f"    ⚠️  该行列数不足53列，可能被跳过")
            
            # 检查关键列是否有数据
            key_columns = [6, 8, 51]  # 位号、变量名、PLC地址
            missing_key_data = []
            for col_idx in key_columns:
                if col_idx < len(row):
                    if pd.isna(row.iloc[col_idx]) or str(row.iloc[col_idx]).strip() == '':
                        missing_key_data.append(col_idx)
                else:
                    missing_key_data.append(col_idx)
            
            if missing_key_data:
                print(f"    ⚠️  关键列缺失数据: {missing_key_data}")
        
        # 检查第51列（PLC地址）的数据
        print(f"\n第51列（PLC地址）数据检查:")
        if len(df.columns) > 51:
            plc_col = df.iloc[:, 51]
            valid_plc_count = plc_col.notna().sum()
            print(f"  有效PLC地址数量: {valid_plc_count}/{len(df)}")
            
            # 显示前几个PLC地址
            for i, addr in enumerate(plc_col.head(10)):
                if pd.notna(addr):
                    print(f"    第{i+2}行: {addr}")
        else:
            print(f"  ⚠️  Excel文件只有{len(df.columns)}列，没有第51列")
        
        # 检查模块类型分布
        if len(df.columns) > 2:
            module_type_col = df.iloc[:, 2]
            print(f"\n模块类型分布:")
            type_counts = module_type_col.value_counts()
            for module_type, count in type_counts.items():
                print(f"  {module_type}: {count}个")
        
        # 检查哪些行可能被跳过
        print(f"\n可能被跳过的行分析:")
        skipped_rows = []
        for i, row in df.iterrows():
            # 检查是否满足解析条件
            row_issues = []
            
            # 1. 列数检查
            if len(row) < 53:
                row_issues.append(f"列数不足({len(row)}<53)")
            
            # 2. 关键字段检查
            key_fields = [
                (6, "位号"),
                (8, "变量名称"),
                (51, "PLC地址")
            ]
            
            for col_idx, field_name in key_fields:
                if col_idx >= len(row) or pd.isna(row.iloc[col_idx]) or str(row.iloc[col_idx]).strip() == '':
                    row_issues.append(f"{field_name}为空")
            
            if row_issues:
                skipped_rows.append((i+2, row_issues))
        
        if skipped_rows:
            print(f"  发现{len(skipped_rows)}行可能被跳过:")
            for row_num, issues in skipped_rows:
                print(f"    第{row_num}行: {', '.join(issues)}")
        else:
            print(f"  所有行都应该能正常解析")
        
        print(f"\n预期解析结果:")
        expected_parsed = len(df) - len(skipped_rows)
        print(f"  应该解析: {expected_parsed}行")
        print(f"  实际解析: 80行")
        print(f"  差异: {expected_parsed - 80}行")
        
    except Exception as e:
        print(f"分析失败: {e}")

if __name__ == "__main__":
    # 分析测试Excel文件
    excel_file = "测试IO.xlsx"
    if os.path.exists(excel_file):
        analyze_excel_file(excel_file)
    else:
        print(f"Excel文件不存在: {excel_file}")
        print("当前目录文件:")
        for f in os.listdir("."):
            print(f"  {f}")
