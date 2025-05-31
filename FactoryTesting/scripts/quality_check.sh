#!/bin/bash

# FAT_TEST 代码质量检查脚本
# 用于第五阶段的代码质量提升

set -e  # 遇到错误立即退出

echo "🔍 FAT_TEST 代码质量检查开始"
echo "================================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查结果统计
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# 记录检查结果
check_result() {
    local name="$1"
    local result="$2"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [ "$result" -eq 0 ]; then
        echo -e "${GREEN}✅ $name${NC}"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        echo -e "${RED}❌ $name${NC}"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
}

# 进入Rust项目目录
cd FactoryTesting/src-tauri

echo -e "${BLUE}📦 检查项目结构...${NC}"

# 检查必要的文件是否存在
if [ -f "Cargo.toml" ]; then
    check_result "Cargo.toml 存在" 0
else
    check_result "Cargo.toml 存在" 1
fi

if [ -f "src/lib.rs" ]; then
    check_result "lib.rs 存在" 0
else
    check_result "lib.rs 存在" 1
fi

echo -e "${BLUE}🦀 Rust 代码质量检查...${NC}"

# 1. 代码格式检查
echo "检查代码格式..."
if cargo fmt --check > /dev/null 2>&1; then
    check_result "代码格式检查" 0
else
    check_result "代码格式检查" 1
    echo -e "${YELLOW}💡 运行 'cargo fmt' 来修复格式问题${NC}"
fi

# 2. Clippy 静态分析
echo "运行 Clippy 静态分析..."
if cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    check_result "Clippy 静态分析" 0
else
    check_result "Clippy 静态分析" 1
    echo -e "${YELLOW}💡 运行 'cargo clippy --fix' 来修复部分问题${NC}"
fi

# 3. 编译检查
echo "检查编译..."
if cargo check > /dev/null 2>&1; then
    check_result "编译检查" 0
else
    check_result "编译检查" 1
fi

# 4. 单元测试
echo "运行单元测试..."
if cargo test --lib > /dev/null 2>&1; then
    check_result "单元测试" 0
else
    check_result "单元测试" 1
fi

# 5. 集成测试
echo "运行集成测试..."
if cargo test --test "*" > /dev/null 2>&1; then
    check_result "集成测试" 0
else
    check_result "集成测试" 1
fi

# 6. 文档测试
echo "运行文档测试..."
if cargo test --doc > /dev/null 2>&1; then
    check_result "文档测试" 0
else
    check_result "文档测试" 1
fi

# 7. 安全审计
echo "运行安全审计..."
if command -v cargo-audit > /dev/null 2>&1; then
    if cargo audit > /dev/null 2>&1; then
        check_result "安全审计" 0
    else
        check_result "安全审计" 1
    fi
else
    echo -e "${YELLOW}⚠️  cargo-audit 未安装，跳过安全审计${NC}"
    echo -e "${YELLOW}💡 运行 'cargo install cargo-audit' 来安装${NC}"
fi

# 8. 依赖检查
echo "检查依赖..."
if cargo tree > /dev/null 2>&1; then
    check_result "依赖检查" 0
else
    check_result "依赖检查" 1
fi

# 9. 代码覆盖率（如果安装了 tarpaulin）
echo "检查测试覆盖率..."
if command -v cargo-tarpaulin > /dev/null 2>&1; then
    echo "生成测试覆盖率报告..."
    if cargo tarpaulin --out Html --output-dir target/coverage > /dev/null 2>&1; then
        check_result "测试覆盖率" 0
        echo -e "${GREEN}📊 覆盖率报告已生成: target/coverage/tarpaulin-report.html${NC}"
    else
        check_result "测试覆盖率" 1
    fi
else
    echo -e "${YELLOW}⚠️  cargo-tarpaulin 未安装，跳过覆盖率检查${NC}"
    echo -e "${YELLOW}💡 运行 'cargo install cargo-tarpaulin' 来安装${NC}"
fi

# 10. 性能基准测试
echo "运行性能基准测试..."
if cargo test --test performance_benchmark_test > /dev/null 2>&1; then
    check_result "性能基准测试" 0
else
    check_result "性能基准测试" 1
fi

echo -e "${BLUE}🌐 前端代码质量检查...${NC}"

# 切换到前端目录
cd ../

# 检查 Angular 项目
if [ -f "package.json" ]; then
    check_result "package.json 存在" 0
    
    # 检查依赖是否安装
    if [ -d "node_modules" ]; then
        check_result "依赖已安装" 0
        
        # TypeScript 编译检查
        echo "检查 TypeScript 编译..."
        if npm run build > /dev/null 2>&1; then
            check_result "TypeScript 编译" 0
        else
            check_result "TypeScript 编译" 1
        fi
        
        # Angular 测试
        echo "运行 Angular 测试..."
        if npm run test -- --watch=false --browsers=ChromeHeadless > /dev/null 2>&1; then
            check_result "Angular 测试" 0
        else
            check_result "Angular 测试" 1
        fi
        
        # ESLint 检查（如果配置了）
        if [ -f ".eslintrc.json" ] || [ -f ".eslintrc.js" ]; then
            echo "运行 ESLint 检查..."
            if npm run lint > /dev/null 2>&1; then
                check_result "ESLint 检查" 0
            else
                check_result "ESLint 检查" 1
            fi
        fi
        
    else
        check_result "依赖已安装" 1
        echo -e "${YELLOW}💡 运行 'npm install' 来安装依赖${NC}"
    fi
else
    check_result "package.json 存在" 1
fi

echo ""
echo "================================"
echo -e "${BLUE}📊 质量检查总结${NC}"
echo "================================"
echo -e "总检查项: ${TOTAL_CHECKS}"
echo -e "${GREEN}通过: ${PASSED_CHECKS}${NC}"
echo -e "${RED}失败: ${FAILED_CHECKS}${NC}"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}🎉 所有质量检查都通过了！${NC}"
    exit 0
else
    PASS_RATE=$((PASSED_CHECKS * 100 / TOTAL_CHECKS))
    echo -e "${YELLOW}⚠️  通过率: ${PASS_RATE}%${NC}"
    
    if [ $PASS_RATE -ge 80 ]; then
        echo -e "${YELLOW}✨ 质量良好，建议修复剩余问题${NC}"
        exit 0
    else
        echo -e "${RED}❌ 质量需要改进，请修复失败的检查项${NC}"
        exit 1
    fi
fi
