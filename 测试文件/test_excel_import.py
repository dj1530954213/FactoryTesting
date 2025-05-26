#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试Excel导入功能
"""

import pandas as pd
import json
import os

def test_excel_import():
    """测试Excel导入功能"""
    excel_file = "测试IO.xlsx"
    
    if not os.path.exists(excel_file):
        print(f"错误: Excel文件不存在: {excel_file}")
        return
    
    print(f"正在测试Excel文件: {excel_file}")
    
    try:
        # 读取Excel文件
        df = pd.read_excel(excel_file)
        
        print(f"成功读取Excel文件！")
        print(f"总行数: {len(df)}")
        print(f"总列数: {len(df.columns)}")
        print(f"列名: {list(df.columns)}")
        
        # 显示前几行数据
        print("\n前5行数据:")
        for i, row in df.head().iterrows():
            print(f"第{i+1}行:")
            for j, col in enumerate(df.columns):
                if j < 12:  # 只显示前12列
                    print(f"  {col}: {row[col]}")
            print()
        
        # 统计有效数据行数
        valid_rows = 0
        for i, row in df.iterrows():
            if i == 0:  # 跳过标题行
                continue
            if pd.notna(row.iloc[6]) and str(row.iloc[6]).strip():  # 位号不为空
                valid_rows += 1
        
        print(f"有效数据行数: {valid_rows}")
        
        # 检查关键列的数据
        print("\n关键列数据检查:")
        print(f"位号列(第7列)非空行数: {df.iloc[:, 6].notna().sum()}")
        print(f"变量名列(第9列)非空行数: {df.iloc[:, 8].notna().sum()}")
        print(f"模块类型列(第3列)非空行数: {df.iloc[:, 2].notna().sum()}")
        
        # 显示模块类型分布
        if len(df) > 1:
            module_types = df.iloc[1:, 2].value_counts()
            print(f"\n模块类型分布:")
            for module_type, count in module_types.items():
                print(f"  {module_type}: {count}个")
        
        return True
        
    except Exception as e:
        print(f"读取Excel文件失败: {e}")
        return False

if __name__ == "__main__":
    test_excel_import() 