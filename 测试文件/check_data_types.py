#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
检查Excel文件中的数据类型和模块类型
"""

import pandas as pd

def check_excel_data():
    """检查Excel文件中的数据类型"""
    try:
        # 读取Excel文件
        df = pd.read_excel('测试IO.xlsx')
        
        print("=== Excel文件数据分析 ===")
        print(f"总行数: {len(df)}")
        print(f"总列数: {len(df.columns)}")
        
        # 显示所有列名
        print("\n所有列名:")
        for i, col in enumerate(df.columns):
            print(f"  {i}: {col}")
        
        # 检查供电类型列
        power_supply_col = '供电类型（有源/无源）'
        if power_supply_col in df.columns:
            print(f"\n{power_supply_col}列的唯一值:")
            power_types = df[power_supply_col].unique()
            for pt in power_types:
                count = len(df[df[power_supply_col] == pt])
                print(f"  '{pt}': {count}个")
            
            # 显示前10行的供电类型
            print(f"\n前10行的{power_supply_col}:")
            for i in range(min(10, len(df))):
                value = df.iloc[i][power_supply_col]
                module_type = df.iloc[i]['模块类型']
                tag = df.iloc[i]['位号']
                print(f"  行{i+1}: {tag} ({module_type}) -> '{value}'")
        else:
            print(f"\n❌ 未找到'{power_supply_col}'列")
        
        # 检查模块类型列的唯一值
        print("\n模块类型列的唯一值:")
        module_types = df['模块类型'].unique()
        for mt in module_types:
            count = len(df[df['模块类型'] == mt])
            print(f"  {mt}: {count}个")
        
        # 检查数据类型列的唯一值
        print("\n数据类型列的唯一值:")
        data_types = df['数据类型'].unique()
        for dt in data_types:
            count = len(df[df['数据类型'] == dt])
            print(f"  {dt}: {count}个")
        
        # 显示前几行的关键列
        print("\n前5行关键列数据:")
        key_columns = ['序号', '模块名称', '模块类型', '供电类型（有源/无源）', '通道位号', '位号', '场站名', '变量名称（HMI）', '变量描述', '数据类型', 'PLC绝对地址']
        available_columns = [col for col in key_columns if col in df.columns]
        print(f"可用的关键列: {available_columns}")
        print(df[available_columns].head())
        
    except Exception as e:
        print(f"错误: {e}")

if __name__ == "__main__":
    check_excel_data() 