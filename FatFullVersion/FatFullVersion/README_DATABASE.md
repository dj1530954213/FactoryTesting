# 数据库使用说明

本应用使用SQLite数据库存储配置信息，包括PLC连接配置和通道比较表。在首次运行应用之前，需要初始化数据库。

## 初始化数据库方法

### 方法一：运行批处理文件（推荐）

1. 双击项目目录中的 `CreateDatabase.bat` 文件
2. 等待脚本执行完成，显示"数据库创建成功！"消息
3. 按任意键关闭窗口

### 方法二：手动执行PowerShell脚本

1. 打开PowerShell终端，确保当前目录是项目所在目录
2. 执行以下命令：
```powershell
.\CreateDatabase.ps1
```
3. 等待脚本执行完成

### 方法三：手动执行EF Core命令

如果上述两种方法无法正常工作，可以尝试手动执行以下步骤：

1. 确保已安装EF Core工具：
```
dotnet tool install --global dotnet-ef
```

2. 添加迁移：
```
dotnet ef migrations add InitialCreate --project FatFullVersion.csproj --context ApplicationDbContext
```

3. 更新数据库：
```
dotnet ef database update --project FatFullVersion.csproj --context ApplicationDbContext
```

## 数据库位置

数据库文件位于用户本地应用程序数据目录：
```
%LocalAppData%\FatFullVersion\fattest.db
```

## 常见问题

### 问题1：应用启动时提示"找不到数据库"或类似错误

解决方法：
- 确保已运行以上提到的数据库初始化命令之一
- 检查 `%LocalAppData%\FatFullVersion` 目录是否存在，如果不存在，请尝试手动创建
- 检查权限问题，确保应用程序有权限读写该目录

### 问题2：数据库初始化脚本失败

解决方法：
- 确保已安装 .NET SDK 和 EF Core工具
- 尝试以管理员身份运行脚本
- 检查错误信息，根据具体情况解决

### 问题3：使用过程中数据库损坏

解决方法：
- 删除 `%LocalAppData%\FatFullVersion\fattest.db` 文件
- 重新运行初始化脚本
- 重启应用程序

## 数据备份

建议定期备份数据库文件，以防数据丢失：
1. 关闭应用程序
2. 复制 `%LocalAppData%\FatFullVersion\fattest.db` 文件到安全位置
3. 重新启动应用程序

## 技术说明

本应用使用：
- Entity Framework Core 8.0 作为ORM框架
- SQLite 作为嵌入式数据库
- 代码优先（Code-First）方式定义数据库结构 