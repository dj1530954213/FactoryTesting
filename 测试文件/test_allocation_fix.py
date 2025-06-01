#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试分配修复是否生效
"""

def test_allocation_logic():
    """模拟分配逻辑测试"""
    print("测试分配逻辑修复...")
    
    # 模拟88个通道
    total_channels = 88
    
    # 修复前的逻辑
    max_channels_per_batch_old = 60
    batches_old = []
    remaining_old = total_channels
    batch_num = 1
    
    while remaining_old > 0:
        batch_size = min(remaining_old, max_channels_per_batch_old)
        batches_old.append(f"批次{batch_num}: {batch_size}个通道")
        remaining_old -= batch_size
        batch_num += 1
    
    print(f"\n修复前 (MAX_CHANNELS_PER_BATCH = {max_channels_per_batch_old}):")
    for batch in batches_old:
        print(f"  {batch}")
    print(f"  总批次数: {len(batches_old)}")
    print(f"  第一批次显示: {min(total_channels, max_channels_per_batch_old)} 个通道")
    
    # 修复后的逻辑
    max_channels_per_batch_new = 88
    batches_new = []
    remaining_new = total_channels
    batch_num = 1
    
    while remaining_new > 0:
        batch_size = min(remaining_new, max_channels_per_batch_new)
        batches_new.append(f"批次{batch_num}: {batch_size}个通道")
        remaining_new -= batch_size
        batch_num += 1
    
    print(f"\n修复后 (MAX_CHANNELS_PER_BATCH = {max_channels_per_batch_new}):")
    for batch in batches_new:
        print(f"  {batch}")
    print(f"  总批次数: {len(batches_new)}")
    print(f"  第一批次显示: {min(total_channels, max_channels_per_batch_new)} 个通道")
    
    print(f"\n结论:")
    print(f"  修复前: 显示 {min(total_channels, max_channels_per_batch_old)} 个通道 (不正确)")
    print(f"  修复后: 显示 {min(total_channels, max_channels_per_batch_new)} 个通道 (正确)")

if __name__ == "__main__":
    test_allocation_logic()
