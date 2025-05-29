#!/usr/bin/env python3
"""
测试后端批次分配功能的脚本

这个脚本通过单元测试的方式验证 ChannelAllocationService 的功能
"""

import subprocess
import sys
import os

def run_rust_test():
    """运行 Rust 的单元测试"""
    print("=== 运行 Rust 后端单元测试 ===")
    
    # 切换到 src-tauri 目录
    original_dir = os.getcwd()
    tauri_dir = os.path.join(original_dir, "src-tauri")
    
    try:
        os.chdir(tauri_dir)
        
        # 运行特定的测试
        cmd = ["cargo", "test", "test_multiple_batch_allocation", "--", "--nocapture"]
        print(f"执行命令: {' '.join(cmd)}")
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        print("=== 测试输出 ===")
        print(result.stdout)
        
        if result.stderr:
            print("=== 错误输出 ===")
            print(result.stderr)
            
        if result.returncode == 0:
            print("✅ 测试通过!")
            return True
        else:
            print(f"❌ 测试失败，退出码: {result.returncode}")
            return False
            
    except Exception as e:
        print(f"❌ 运行测试时出错: {e}")
        return False
        
    finally:
        os.chdir(original_dir)

def main():
    print("=== 开始测试后端批次分配功能 ===")
    
    # 运行 Rust 测试
    success = run_rust_test()
    
    if success:
        print("\n🎉 所有测试都通过了！批次分配功能正常工作。")
        print("\n现在可以测试前端功能：")
        print("1. 启动应用: npm run tauri dev")
        print("2. 导入测试数据文件: test_batch_allocation_data.json")
        print("3. 查看是否正确生成了多个批次")
    else:
        print("\n❌ 测试失败，请检查后端代码")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 