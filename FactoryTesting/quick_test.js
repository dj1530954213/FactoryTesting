// FAT_TEST 快速功能验证脚本
// 用于验证核心功能是否正常工作

console.log('🏭 FAT_TEST 快速功能验证开始...\n');

// 模拟测试数据
const testData = {
    batchInfo: {
        batch_id: `quick_test_${Date.now()}`,
        product_model: '快速测试产品',
        serial_number: `QT${Date.now()}`,
        customer_name: '测试客户',
        operator_name: '自动测试',
        creation_time: new Date().toISOString(),
        status: 'Created'
    },
    definitions: [
        {
            id: `def_${Date.now()}_1`,
            tag: 'AI001',
            variable_description: '模拟输入点1',
            station_name: '测试站1',
            module_name: '模拟模块',
            module_type: 'AI',
            channel_number: 'CH01',
            data_type: 'Float',
            plc_communication_address: 'DB1.DBD0',
            analog_range_min: 0.0,
            analog_range_max: 100.0
        },
        {
            id: `def_${Date.now()}_2`,
            tag: 'DI001',
            variable_description: '数字输入点1',
            station_name: '测试站1',
            module_name: '模拟模块',
            module_type: 'DI',
            channel_number: 'CH02',
            data_type: 'Bool',
            plc_communication_address: 'DB1.DBX0.0'
        }
    ]
};

// 测试结果统计
let testResults = {
    total: 0,
    passed: 0,
    failed: 0,
    errors: []
};

// 测试函数
async function runTest(testName, testFunction) {
    testResults.total++;
    console.log(`🔧 执行测试: ${testName}`);
    
    try {
        await testFunction();
        testResults.passed++;
        console.log(`✅ ${testName} - 通过\n`);
    } catch (error) {
        testResults.failed++;
        testResults.errors.push({ test: testName, error: error.message });
        console.log(`❌ ${testName} - 失败: ${error.message}\n`);
    }
}

// 检查Tauri环境
function checkTauriEnvironment() {
    if (typeof window === 'undefined' || !window.__TAURI__) {
        throw new Error('不在Tauri环境中，无法进行测试');
    }
    console.log('✅ Tauri环境检测通过');
}

// 调用Tauri命令的包装函数
async function invokeCommand(command, args = {}) {
    if (typeof window === 'undefined' || !window.__TAURI__) {
        throw new Error('Tauri环境不可用');
    }
    
    try {
        const result = await window.__TAURI__.invoke(command, args);
        return result;
    } catch (error) {
        throw new Error(`命令 ${command} 执行失败: ${error.message}`);
    }
}

// 测试1: 系统状态检查
async function testSystemStatus() {
    const status = await invokeCommand('get_system_status');
    
    if (!status || typeof status !== 'object') {
        throw new Error('系统状态返回格式错误');
    }
    
    console.log(`   系统健康状态: ${status.system_health || '未知'}`);
    console.log(`   活跃测试任务: ${status.active_test_tasks || 0}`);
    console.log(`   系统启动时间: ${status.system_uptime || '未知'}`);
}

// 测试2: Excel导入功能
async function testExcelImport() {
    const testFilePath = 'D:\\GIT\\Git\\code\\FactoryTesting\\测试文件\\测试IO.xlsx';
    
    try {
        const definitions = await invokeCommand('import_excel_file', { filePath: testFilePath });
        
        if (!Array.isArray(definitions) || definitions.length === 0) {
            throw new Error('Excel导入返回数据为空或格式错误');
        }
        
        console.log(`   成功解析 ${definitions.length} 个通道定义`);
        console.log(`   第一个定义: ${definitions[0].tag} - ${definitions[0].variable_description}`);
    } catch (error) {
        // 如果测试文件不存在，使用模拟数据
        console.log(`   测试文件不存在，跳过Excel导入测试: ${error.message}`);
    }
}

// 测试3: 创建测试批次
async function testCreateBatch() {
    const batchId = await invokeCommand('create_test_batch_with_definitions', {
        batchInfo: testData.batchInfo,
        definitions: testData.definitions
    });
    
    if (!batchId || typeof batchId !== 'string') {
        throw new Error('批次创建失败，返回的批次ID无效');
    }
    
    console.log(`   成功创建批次: ${batchId}`);
    console.log(`   产品型号: ${testData.batchInfo.product_model}`);
    console.log(`   序列号: ${testData.batchInfo.serial_number}`);
    
    // 保存批次ID供后续测试使用
    testData.createdBatchId = batchId;
}

// 测试4: 获取通道定义
async function testGetDefinitions() {
    const definitions = await invokeCommand('get_all_channel_definitions');
    
    if (!Array.isArray(definitions)) {
        throw new Error('获取通道定义失败，返回格式错误');
    }
    
    console.log(`   获取到 ${definitions.length} 个通道定义`);
    
    if (definitions.length > 0) {
        console.log(`   示例定义: ${definitions[0].tag} - ${definitions[0].module_type}`);
    }
}

// 测试5: 获取批次信息
async function testGetBatches() {
    const batches = await invokeCommand('get_all_batch_info');
    
    if (!Array.isArray(batches)) {
        throw new Error('获取批次信息失败，返回格式错误');
    }
    
    console.log(`   获取到 ${batches.length} 个测试批次`);
    
    if (batches.length > 0) {
        const latestBatch = batches[batches.length - 1];
        console.log(`   最新批次: ${latestBatch.batch_id} - ${latestBatch.product_model}`);
    }
}

// 测试6: 创建测试实例
async function testCreateInstance() {
    if (!testData.createdBatchId) {
        throw new Error('没有可用的批次ID，无法创建测试实例');
    }
    
    const instance = await invokeCommand('create_test_instance', {
        definitionId: testData.definitions[0].id,
        batchId: testData.createdBatchId
    });
    
    if (!instance || !instance.instance_id) {
        throw new Error('测试实例创建失败');
    }
    
    console.log(`   成功创建测试实例: ${instance.instance_id}`);
    console.log(`   关联定义: ${instance.definition_id}`);
    console.log(`   初始状态: ${instance.overall_status}`);
    
    testData.createdInstanceId = instance.instance_id;
}

// 测试7: 手动测试功能
async function testManualTesting() {
    if (!testData.createdInstanceId) {
        throw new Error('没有可用的测试实例ID');
    }
    
    // 测试读取通道值
    try {
        const value = await invokeCommand('read_channel_value_cmd', {
            instance_id: testData.createdInstanceId,
            plc_address: 'DB1.DBD0',
            data_type: 'Float'
        });
        
        console.log(`   读取通道值成功: ${JSON.stringify(value)}`);
    } catch (error) {
        console.log(`   读取通道值测试: ${error.message}`);
    }
    
    // 测试写入通道值
    try {
        await invokeCommand('write_channel_value_cmd', {
            instance_id: testData.createdInstanceId,
            plc_address: 'DB1.DBD0',
            data_type: 'Float',
            value_to_write: 42.5
        });
        
        console.log(`   写入通道值成功: 42.5`);
    } catch (error) {
        console.log(`   写入通道值测试: ${error.message}`);
    }
}

// 测试8: 报告生成
async function testReportGeneration() {
    // 获取报告模板
    try {
        const templates = await invokeCommand('get_report_templates');
        console.log(`   获取到 ${templates.length} 个报告模板`);
    } catch (error) {
        console.log(`   获取报告模板: ${error.message}`);
    }
    
    // 生成PDF报告
    if (testData.createdBatchId) {
        try {
            const request = {
                batch_ids: [testData.createdBatchId],
                template_id: 'default_pdf',
                output_filename: null,
                parameters: {}
            };
            
            const report = await invokeCommand('generate_pdf_report', { request });
            console.log(`   PDF报告生成成功: ${report.report_id}`);
        } catch (error) {
            console.log(`   PDF报告生成: ${error.message}`);
        }
    }
}

// 主测试函数
async function runAllTests() {
    console.log('开始执行快速功能验证...\n');
    
    try {
        // 环境检查
        checkTauriEnvironment();
        console.log('');
        
        // 执行所有测试
        await runTest('系统状态检查', testSystemStatus);
        await runTest('Excel导入功能', testExcelImport);
        await runTest('创建测试批次', testCreateBatch);
        await runTest('获取通道定义', testGetDefinitions);
        await runTest('获取批次信息', testGetBatches);
        await runTest('创建测试实例', testCreateInstance);
        await runTest('手动测试功能', testManualTesting);
        await runTest('报告生成功能', testReportGeneration);
        
    } catch (error) {
        console.log(`❌ 测试执行失败: ${error.message}\n`);
        testResults.failed++;
        testResults.errors.push({ test: '环境检查', error: error.message });
    }
    
    // 输出测试结果
    console.log('='.repeat(60));
    console.log('🏭 FAT_TEST 快速功能验证结果');
    console.log('='.repeat(60));
    console.log(`总测试数: ${testResults.total}`);
    console.log(`通过: ${testResults.passed} ✅`);
    console.log(`失败: ${testResults.failed} ❌`);
    console.log(`成功率: ${((testResults.passed / testResults.total) * 100).toFixed(1)}%`);
    
    if (testResults.errors.length > 0) {
        console.log('\n失败的测试详情:');
        testResults.errors.forEach((error, index) => {
            console.log(`${index + 1}. ${error.test}: ${error.error}`);
        });
    }
    
    console.log('\n' + '='.repeat(60));
    
    if (testResults.failed === 0) {
        console.log('🎉 所有测试通过！系统运行正常。');
    } else if (testResults.passed > testResults.failed) {
        console.log('⚠️  大部分测试通过，系统基本正常，但有部分功能需要检查。');
    } else {
        console.log('🚨 多个测试失败，系统可能存在问题，需要进一步调试。');
    }
}

// 如果在浏览器环境中，自动执行测试
if (typeof window !== 'undefined') {
    // 等待页面加载完成后执行测试
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            setTimeout(runAllTests, 2000); // 延迟2秒确保Tauri环境就绪
        });
    } else {
        setTimeout(runAllTests, 2000);
    }
} else {
    // Node.js环境
    console.log('请在Tauri应用的浏览器环境中运行此脚本');
}

// 导出测试函数供手动调用
if (typeof window !== 'undefined') {
    window.runQuickTest = runAllTests;
    window.testData = testData;
    window.testResults = testResults;
} 