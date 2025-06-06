# 重复数据插入问题修复验证

## 问题描述
用户报告在点表导入和测试过程中，数据库中的点位数量从88个增加到680个，说明存在重复数据插入的问题。

## 问题根源分析
通过分析发现，问题在于**前端有多个流程都在创建批次**：

1. **数据导入页面** (`data-import.component.ts`)：调用 `autoAllocateBatch` 
2. **测试执行页面** (`test-execution.component.ts`)：调用 `submitTestExecution`

当用户从数据导入页面导航到测试执行页面时，两个页面都会创建批次，导致重复数据插入。

## 修复方案

### 1. 前端修复
- **修改测试执行组件**：使其能够接收URL参数中的批次ID
- **添加批次检查逻辑**：如果已有批次，不再创建新批次
- **改进导航流程**：数据导入页面传递批次ID给测试执行页面

### 2. 具体修改

#### 测试执行组件 (`test-execution.component.ts`)
```typescript
// 添加路由参数检查
constructor(
  private tauriApi: TauriApiService,
  private route: ActivatedRoute,
  private router: Router
) {}

ngOnInit() {
  // 检查URL参数中是否有批次ID
  this.route.queryParams.subscribe(params => {
    if (params['batchId']) {
      this.currentBatchId = params['batchId'];
      this.addLog('info', `从URL参数接收到批次ID: ${this.currentBatchId}`);
      // 加载并选择指定批次
      this.loadSessionBatches().then(() => {
        if (this.currentBatchId) {
          this.selectBatch(this.currentBatchId);
        }
      });
    } else {
      this.loadSessionBatches();
    }
  });
}

// 修改批次创建逻辑
createAndSubmitBatch() {
  // 检查是否已有批次，如果有就不再创建
  if (this.sessionBatches.length > 0) {
    this.addLog('warning', '检测到已有批次，请使用现有批次进行测试');
    // 自动选择第一个批次
    if (!this.currentBatchId && this.sessionBatches.length > 0) {
      this.selectBatch(this.sessionBatches[0].batch_id);
    }
    return;
  }
  // 原有的创建逻辑...
}
```

#### 数据导入组件 (`data-import.component.ts`)
```typescript
// 导航时传递批次ID
setTimeout(() => {
  this.router.navigate(['/test-execution'], {
    queryParams: { batchId: response.batch_info.batch_id }
  });
}, 1000);
```

## 修复效果

### 修复前
- 88个点位 → 680个点位（重复插入约7-8次）
- 每次页面切换都创建新批次
- 数据库中存在大量重复数据

### 修复后
- ✅ 只在数据导入时创建一次批次
- ✅ 测试执行页面使用现有批次
- ✅ 避免重复数据插入
- ✅ 正确的数据流：导入 → 分配 → 测试

## 测试验证步骤

1. **清空数据库**
2. **导入点表**（88个点位）
3. **检查数据库**：应该只有88个点位
4. **导航到测试执行页面**
5. **开始测试**
6. **再次检查数据库**：仍然只有88个点位

## 预期结果
- 数据库中的点位数量保持稳定（88个）
- 不再出现重复数据插入
- 测试流程正常工作
- 前端正确显示批次信息

## 注意事项
- 确保前端路由参数正确传递
- 测试执行页面要正确处理批次选择
- 保持数据一致性和完整性
