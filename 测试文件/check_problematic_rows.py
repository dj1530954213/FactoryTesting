#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
检查Excel文件中可能导致解析失败的行
"""

import pandas as pd
import re

def check_problematic_rows():
    """检查可能有问题的行"""
    df = pd.read_excel("测试IO.xlsx")
    
    print(f"检查Excel文件中的问题行...")
    print(f"总行数: {len(df)}")
    
    problematic_rows = []
    
    for i, row in df.iterrows():
        row_num = i + 2  # Excel行号（从2开始，因为第1行是标题）
        issues = []
        
        # 检查关键字段
        tag = str(row.iloc[6]).strip() if pd.notna(row.iloc[6]) else ""
        var_name = str(row.iloc[8]).strip() if pd.notna(row.iloc[8]) else ""
        module_type = str(row.iloc[2]).strip() if pd.notna(row.iloc[2]) else ""
        data_type = str(row.iloc[10]).strip() if pd.notna(row.iloc[10]) else ""
        plc_addr = str(row.iloc[51]).strip() if pd.notna(row.iloc[51]) else ""
        
        # 1. 检查空字段
        if not tag:
            issues.append("位号为空")
        if not var_name:
            issues.append("变量名为空")
        if not plc_addr:
            issues.append("PLC地址为空")
        
        # 2. 检查模块类型
        if module_type.upper() not in ['AI', 'AO', 'DI', 'DO']:
            issues.append(f"模块类型无效: '{module_type}'")
        
        # 3. 检查数据类型
        valid_data_types = ['BOOL', 'BOOLEAN', 'INT', 'INTEGER', 'FLOAT', 'REAL', 'STRING']
        if data_type.upper() not in valid_data_types:
            issues.append(f"数据类型无效: '{data_type}'")
        
        # 4. 检查特殊字符
        if tag and not re.match(r'^[A-Za-z0-9_\-\.]+$', tag):
            issues.append(f"位号包含特殊字符: '{tag}'")
        
        if var_name and not re.match(r'^[A-Za-z0-9_\-\.\u4e00-\u9fff\(\)]+$', var_name):
            issues.append(f"变量名包含特殊字符: '{var_name}'")
        
        if issues:
            problematic_rows.append((row_num, issues, {
                '位号': tag,
                '变量名': var_name,
                '模块类型': module_type,
                '数据类型': data_type,
                'PLC地址': plc_addr
            }))
    
    if problematic_rows:
        print(f"\n发现 {len(problematic_rows)} 行可能有问题:")
        for row_num, issues, data in problematic_rows:
            print(f"\n第{row_num}行:")
            print(f"  问题: {', '.join(issues)}")
            print(f"  数据: {data}")
    else:
        print("\n没有发现明显的问题行")
    
    # 统计模块类型和数据类型
    print(f"\n模块类型统计:")
    module_counts = df.iloc[:, 2].value_counts()
    for module_type, count in module_counts.items():
        print(f"  {module_type}: {count}个")
    
    print(f"\n数据类型统计:")
    data_type_counts = df.iloc[:, 10].value_counts()
    for data_type, count in data_type_counts.items():
        print(f"  {data_type}: {count}个")
    
    return len(problematic_rows)

if __name__ == "__main__":
    check_problematic_rows()
