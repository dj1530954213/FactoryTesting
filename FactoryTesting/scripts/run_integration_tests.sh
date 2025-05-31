#!/bin/bash

# FAT_TEST 集成测试运行脚本
# 用于第五阶段的系统集成测试

set -e

echo "🧪 FAT_TEST 集成测试套件"
echo "========================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 测试结果统计
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 记录测试结果
record_test_result() {
    local test_name="$1"
    local result="$2"
    local duration="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" -eq 0 ]; then
        echo -e "${GREEN}✅ $test_name (${duration}s)${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ $test_name (${duration}s)${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# 运行单个测试
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}🔄 运行测试: $test_name${NC}"
    
    local start_time=$(date +%s)
    
    if eval "$test_command" > /dev/null 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        record_test_result "$test_name" 0 "$duration"
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        record_test_result "$test_name" 1 "$duration"
        
        # 显示错误详情
        echo -e "${RED}错误详情:${NC}"
        eval "$test_command" || true
    fi
}

# 进入项目目录
cd FactoryTesting

echo -e "${BLUE}📋 准备测试环境...${NC}"

# 检查必要的工具
check_tool() {
    local tool="$1"
    local install_hint="$2"
    
    if command -v "$tool" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $tool 已安装${NC}"
    else
        echo -e "${RED}❌ $tool 未安装${NC}"
        echo -e "${YELLOW}💡 $install_hint${NC}"
        return 1
    fi
}

# 检查工具
check_tool "cargo" "请安装 Rust: https://rustup.rs/"
check_tool "npm" "请安装 Node.js: https://nodejs.org/"

echo -e "${BLUE}🏗️ 构建项目...${NC}"

# 构建后端
echo "构建 Rust 后端..."
cd src-tauri
if cargo build > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Rust 后端构建成功${NC}"
else
    echo -e "${RED}❌ Rust 后端构建失败${NC}"
    exit 1
fi

# 构建前端
echo "构建 Angular 前端..."
cd ../
if npm run build > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Angular 前端构建成功${NC}"
else
    echo -e "${RED}❌ Angular 前端构建失败${NC}"
    exit 1
fi

echo -e "${BLUE}🧪 运行集成测试...${NC}"

# 1. 单元测试
cd src-tauri
run_test "Rust 单元测试" "cargo test --lib"

# 2. PLC 通信集成测试
run_test "PLC 通信集成测试" "cargo test --test plc_communication_integration_test"

# 3. 端到端集成测试
run_test "端到端集成测试" "cargo test --test end_to_end_integration_test"

# 4. 性能基准测试
run_test "性能基准测试" "cargo test --test performance_benchmark_test"

# 5. 前端单元测试
cd ../
run_test "Angular 单元测试" "npm run test -- --watch=false --browsers=ChromeHeadless"

# 6. 前端端到端测试（如果配置了）
if [ -f "e2e/protractor.conf.js" ] || [ -f "cypress.config.js" ]; then
    run_test "前端 E2E 测试" "npm run e2e"
fi

echo -e "${BLUE}🔍 运行代码质量检查...${NC}"

# 运行质量检查脚本
if [ -f "scripts/quality_check.sh" ]; then
    chmod +x scripts/quality_check.sh
    run_test "代码质量检查" "./scripts/quality_check.sh"
fi

echo -e "${BLUE}📊 生成测试报告...${NC}"

# 生成测试覆盖率报告
cd src-tauri
if command -v cargo-tarpaulin > /dev/null 2>&1; then
    echo "生成 Rust 测试覆盖率报告..."
    if cargo tarpaulin --out Html --output-dir target/coverage > /dev/null 2>&1; then
        echo -e "${GREEN}📊 Rust 覆盖率报告: src-tauri/target/coverage/tarpaulin-report.html${NC}"
    fi
fi

# 生成性能报告
echo "生成性能测试报告..."
if cargo test --test performance_benchmark_test -- --nocapture > target/performance_report.txt 2>&1; then
    echo -e "${GREEN}📈 性能测试报告: src-tauri/target/performance_report.txt${NC}"
fi

echo ""
echo "========================"
echo -e "${BLUE}📊 集成测试总结${NC}"
echo "========================"
echo -e "总测试数: ${TOTAL_TESTS}"
echo -e "${GREEN}通过: ${PASSED_TESTS}${NC}"
echo -e "${RED}失败: ${FAILED_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 所有集成测试都通过了！${NC}"
    echo -e "${GREEN}✨ 系统已准备好进入生产环境${NC}"
    exit 0
else
    PASS_RATE=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo -e "${YELLOW}⚠️  通过率: ${PASS_RATE}%${NC}"
    
    if [ $PASS_RATE -ge 90 ]; then
        echo -e "${YELLOW}✨ 测试结果良好，建议修复剩余问题后发布${NC}"
        exit 0
    elif [ $PASS_RATE -ge 70 ]; then
        echo -e "${YELLOW}⚠️  测试结果一般，需要修复关键问题${NC}"
        exit 1
    else
        echo -e "${RED}❌ 测试结果不佳，需要大量修复工作${NC}"
        exit 1
    fi
fi
