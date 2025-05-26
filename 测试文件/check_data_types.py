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
        key_columns = ['序号', '模块名称', '模块类型', '通道位号', '位号', '场站名', '变量名称（HMI）', '变量描述', '数据类型', 'PLC绝对地址']
        print(df[key_columns].head())
        
        # 检查PLC地址列
        print("\nPLC绝对地址列前5个值:")
        print(df['PLC绝对地址'].head())
        
    except Exception as e:
        print(f"错误: {e}")

if __name__ == "__main__":
    check_excel_data() 