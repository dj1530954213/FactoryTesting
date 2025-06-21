using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using FatFullVersion.IServices;
using FatFullVersion.Models;
using Microsoft.Win32;
using NPOI.SS.UserModel;
using NPOI.SS.Util;
using NPOI.XSSF.UserModel;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 测试结果导出服务实现类 - Excel导出
    /// </summary>
    public class TestResultExportService : ITestResultExportService
    {
        private readonly IMessageService _messageService;

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="messageService">消息服务</param>
        public TestResultExportService(IMessageService messageService)
        {
            _messageService = messageService ?? throw new ArgumentNullException(nameof(messageService));
        }

        /// <summary>
        /// 检查是否所有测试点位都已通过
        /// </summary>
        /// <param name="testResults">测试结果数据</param>
        /// <returns>是否所有点位都已通过测试</returns>
        public bool AreAllTestsPassed(IEnumerable<ChannelMapping> testResults)
        {
            if (testResults == null || !testResults.Any())
            {
                return false;
            }

            // 检查是否所有通道的硬点测试结果都是"通过"
            bool allPassed = testResults.All(c => 
                !string.IsNullOrEmpty(c.HardPointTestResult) && 
                (c.HardPointTestResult == "通过" || c.HardPointTestResult == "已通过"));

            return allPassed;
        }

        /// <summary>
        /// 导出测试结果到Excel文件
        /// </summary>
        /// <param name="testResults">测试结果数据</param>
        /// <param name="filePath">导出文件路径，如果为null则通过文件对话框选择</param>
        /// <returns>导出是否成功</returns>
        public async Task<bool> ExportToExcelAsync(IEnumerable<ChannelMapping> testResults, string filePath = null)
        {
            try
            {
                if (testResults == null || !testResults.Any())
                {
                    await _messageService.ShowAsync("导出失败", "没有可导出的测试结果数据", MessageBoxButton.OK);
                    return false;
                }

                // 如果未指定文件路径，则打开保存文件对话框
                if (string.IsNullOrEmpty(filePath))
                {
                    var saveFileDialog = new SaveFileDialog
                    {
                        Filter = "Excel文件 (*.xlsx)|*.xlsx",
                        Title = "保存测试结果",
                        DefaultExt = "xlsx",
                        FileName = $"测试结果_{DateTime.Now:yyyyMMdd_HHmmss}"
                    };

                    if (saveFileDialog.ShowDialog() != true)
                    {
                        return false; // 用户取消操作
                    }

                    filePath = saveFileDialog.FileName;
                }

                // 确保有效的文件路径
                if (string.IsNullOrEmpty(filePath))
                {
                    await _messageService.ShowAsync("导出失败", "无效的文件路径", MessageBoxButton.OK);
                    return false;
                }

                // 使用Task.Run在后台线程中执行Excel导出操作
                return await Task.Run(() =>
                {
                    try
                    {
                        // 创建工作簿
                        var workbook = new XSSFWorkbook();
                        
                        // 创建工作表
                        var sheet = workbook.CreateSheet("测试结果");
                        
                        // 设置列宽 - 按照DataEditView.xaml中的DataGrid列进行设置
                        sheet.SetColumnWidth(0, 8 * 256);  // 测试ID
                        sheet.SetColumnWidth(1, 10 * 256); // 测试批次
                        sheet.SetColumnWidth(2, 25 * 256); // 变量名称
                        sheet.SetColumnWidth(3, 10 * 256); // 点表类型
                        sheet.SetColumnWidth(4, 10 * 256); // 数据类型
                        sheet.SetColumnWidth(5, 25 * 256); // 测试PLC通道位号
                        sheet.SetColumnWidth(6, 25 * 256); // 被测PLC通道位号
                        sheet.SetColumnWidth(7, 12 * 256); // 行程最小值
                        sheet.SetColumnWidth(8, 12 * 256); // 行程最大值
                        sheet.SetColumnWidth(9, 12 * 256); // 0%对比值
                        sheet.SetColumnWidth(10, 12 * 256); // 25%对比值
                        sheet.SetColumnWidth(11, 12 * 256); // 50%对比值
                        sheet.SetColumnWidth(12, 12 * 256); // 75%对比值
                        sheet.SetColumnWidth(13, 12 * 256); // 100%对比值
                        sheet.SetColumnWidth(14, 15 * 256); // 低低报反馈状态
                        sheet.SetColumnWidth(15, 15 * 256); // 低报反馈状态
                        sheet.SetColumnWidth(16, 15 * 256); // 高报反馈状态
                        sheet.SetColumnWidth(17, 15 * 256); // 高高报反馈状态
                        sheet.SetColumnWidth(18, 15 * 256); // 维护功能检测
                        sheet.SetColumnWidth(19, 15 * 256); // 趋势检查 (新)
                        sheet.SetColumnWidth(20, 15 * 256); // 报表检查 (新)
                        sheet.SetColumnWidth(21, 20 * 256); // 开始测试时间 (原19)
                        sheet.SetColumnWidth(22, 20 * 256); // 最终测试时间 (原20)
                        sheet.SetColumnWidth(23, 15 * 256); // 测试时长(秒) (原21)
                        sheet.SetColumnWidth(24, 25 * 256); // 通道硬点测试结果 (原22)
                        sheet.SetColumnWidth(25, 25 * 256); // 测试结果 (原23)
                        
                        // 创建标题行样式
                        var headerStyle = workbook.CreateCellStyle();
                        var headerFont = workbook.CreateFont();
                        headerFont.IsBold = true;
                        headerFont.FontHeightInPoints = 12;
                        headerStyle.SetFont(headerFont);
                        headerStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        headerStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        headerStyle.BorderTop = BorderStyle.Thin;
                        headerStyle.BorderBottom = BorderStyle.Thin;
                        headerStyle.BorderLeft = BorderStyle.Thin;
                        headerStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建内容行样式
                        var contentStyle = workbook.CreateCellStyle();
                        contentStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        contentStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        contentStyle.BorderTop = BorderStyle.Thin;
                        contentStyle.BorderBottom = BorderStyle.Thin;
                        contentStyle.BorderLeft = BorderStyle.Thin;
                        contentStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建通过状态样式
                        var passedStyle = workbook.CreateCellStyle();
                        passedStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        passedStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        passedStyle.FillForegroundColor = IndexedColors.LightGreen.Index;
                        passedStyle.FillPattern = FillPattern.SolidForeground;
                        passedStyle.BorderTop = BorderStyle.Thin;
                        passedStyle.BorderBottom = BorderStyle.Thin;
                        passedStyle.BorderLeft = BorderStyle.Thin;
                        passedStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建失败状态样式
                        var failedStyle = workbook.CreateCellStyle();
                        failedStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        failedStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        failedStyle.FillForegroundColor = IndexedColors.Rose.Index;
                        failedStyle.FillPattern = FillPattern.SolidForeground;
                        failedStyle.BorderTop = BorderStyle.Thin;
                        failedStyle.BorderBottom = BorderStyle.Thin;
                        failedStyle.BorderLeft = BorderStyle.Thin;
                        failedStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建标题行 - 按照DataEditView.xaml中的DataGrid列顺序
                        var headerRow = sheet.CreateRow(0);
                        var headers = new[] { 
                            "测试ID", "测试批次", "变量名称", "点表类型", "数据类型", 
                            "测试PLC通道位号", "被测PLC通道位号", "行程最小值", "行程最大值", 
                            "0%对比值", "25%对比值", "50%对比值", "75%对比值", "100%对比值", 
                            "低低报反馈状态", "低报反馈状态", "高报反馈状态", "高高报反馈状态", "维护功能检测", 
                            "上位机趋势检查", "上位机报表检查", "开始测试时间", "最终测试时间", "测试时长", "通道硬点测试结果", "测试结果" 
                        };
                        
                        for (int i = 0; i < headers.Length; i++)
                        {
                            var cell = headerRow.CreateCell(i);
                            cell.SetCellValue(headers[i]);
                            cell.CellStyle = headerStyle;
                        }
                        
                        // 填充数据行 - 按照DataEditView.xaml中的DataGrid列顺序和绑定
                        int rowIndex = 1;
                        foreach (var result in testResults)
                        {
                            var dataRow = sheet.CreateRow(rowIndex++);
                            //对于是否是跳过测试的结果进行判断
                            if (result.ResultText.Contains("跳过"))
                            {
                                // 设置单元格值
                                // 1. 测试ID
                                SetCellValue(dataRow, 0, result.TestId.ToString(), contentStyle);
                            
                                // 2. 测试批次
                                SetCellValue(dataRow, 1, result.TestBatch, contentStyle);
                            
                                // 3. 变量名称
                                SetCellValue(dataRow, 2, result.VariableName, contentStyle);
                            
                                // 4. 点表类型
                                SetCellValue(dataRow, 3, result.ModuleType, contentStyle);
                            
                                // 5. 数据类型
                                SetCellValue(dataRow, 4, result.DataType, contentStyle);
                            
                                // 6. 测试PLC通道位号
                                SetCellValue(dataRow, 5, result.TestPLCChannelTag, contentStyle);
                            
                                // 7. 被测PLC通道位号
                                SetCellValue(dataRow, 6, result.ChannelTag, contentStyle);
                            
                                // 8. 行程最小值
                                SetDoubleValue(dataRow, 7, Math.Round(result.RangeLowerLimitValue, 3), contentStyle);

                                // 9. 行程最大值
                                SetDoubleValue(dataRow, 8, Math.Round(result.RangeUpperLimitValue, 3), contentStyle);

                                // 10. 0%对比值
                                SetCellValue(dataRow, 9, "NaN", contentStyle);

                                // 11. 25%对比值
                                SetCellValue(dataRow, 10, "NaN", contentStyle);

                                // 12. 50%对比值
                                SetCellValue(dataRow, 11, "NaN", contentStyle);

                                // 13. 75%对比值
                                SetCellValue(dataRow, 12, "NaN", contentStyle);

                                // 14. 100%对比值
                                SetCellValue(dataRow, 13, "NaN", contentStyle);
                            
                                // 15. 低低报反馈状态
                                SetCellValue(dataRow, 14, "未测试", contentStyle);
                            
                                // 16. 低报反馈状态
                                SetCellValue(dataRow, 15, "未测试", contentStyle);
                            
                                // 17. 高报反馈状态
                                SetCellValue(dataRow, 16, "未测试", contentStyle);
                            
                                // 18. 高高报反馈状态
                                SetCellValue(dataRow, 17, "未测试", contentStyle);
                            
                                // 19. 维护功能检测
                                SetCellValue(dataRow, 18, "未测试", contentStyle);

                                // 20. 趋势检查 (新)
                                SetCellValue(dataRow, 19, "未测试", contentStyle);

                                // 21. 报表检查 (新)
                                SetCellValue(dataRow, 20, "未测试", contentStyle);
                            
                                // 22. 开始测试时间 (原20)
                                var testTimeStrSkipped = result.TestTime.HasValue 
                                    ? result.TestTime.Value.ToString("yyyy-MM-dd HH:mm:ss") 
                                    : "-";
                                SetCellValue(dataRow, 21, "未测试", contentStyle);
                            
                                // 23. 最终测试时间 (原21)
                                var finalTestTimeStrSkipped = result.FinalTestTime.HasValue 
                                    ? result.FinalTestTime.Value.ToString("yyyy-MM-dd HH:mm:ss") 
                                    : "-";
                                SetCellValue(dataRow, 22, "未测试", contentStyle);

                                // 24. 测试时长(秒) (原22)
                                SetCellValue(dataRow, 23, "未测试", contentStyle);
                                // 25. 通道硬点测试结果 (原23)
                                SetCellValue(dataRow, 24, "未测试", contentStyle);

                                // 26. 测试结果 (原24)
                                SetCellValue(dataRow, 25, result.ResultText, contentStyle);
                            }
                            else
                            {
                                // 设置单元格值
                                // 1. 测试ID
                                SetCellValue(dataRow, 0, result.TestId.ToString(), contentStyle);

                                // 2. 测试批次
                                SetCellValue(dataRow, 1, result.TestBatch, contentStyle);

                                // 3. 变量名称
                                SetCellValue(dataRow, 2, result.VariableName, contentStyle);

                                // 4. 点表类型
                                SetCellValue(dataRow, 3, result.ModuleType, contentStyle);

                                // 5. 数据类型
                                SetCellValue(dataRow, 4, result.DataType, contentStyle);

                                // 6. 测试PLC通道位号
                                SetCellValue(dataRow, 5, result.TestPLCChannelTag, contentStyle);

                                // 7. 被测PLC通道位号
                                SetCellValue(dataRow, 6, result.ChannelTag, contentStyle);

                                // 8. 行程最小值
                                SetDoubleValue(dataRow, 7, Math.Round(result.RangeLowerLimitValue, 3), contentStyle);

                                // 9. 行程最大值
                                SetDoubleValue(dataRow, 8, Math.Round(result.RangeUpperLimitValue, 3), contentStyle);

                                // 10. 0%对比值
                                SetDoubleValue(dataRow, 9, Math.Round(result.Value0Percent, 3), contentStyle);

                                // 11. 25%对比值
                                SetDoubleValue(dataRow, 10, Math.Round(result.Value25Percent, 3), contentStyle);

                                // 12. 50%对比值
                                SetDoubleValue(dataRow, 11, Math.Round(result.Value50Percent, 3), contentStyle);

                                // 13. 75%对比值
                                SetDoubleValue(dataRow, 12, Math.Round(result.Value75Percent, 3), contentStyle);

                                // 14. 100%对比值
                                SetDoubleValue(dataRow, 13, Math.Round(result.Value100Percent, 3), contentStyle);

                                // 15. 低低报反馈状态
                                SetCellValue(dataRow, 14, result.LowLowAlarmStatus, contentStyle);

                                // 16. 低报反馈状态
                                SetCellValue(dataRow, 15, result.LowAlarmStatus, contentStyle);

                                // 17. 高报反馈状态
                                SetCellValue(dataRow, 16, result.HighAlarmStatus, contentStyle);

                                // 18. 高高报反馈状态
                                SetCellValue(dataRow, 17, result.HighHighAlarmStatus, contentStyle);

                                // 19. 维护功能检测
                                SetCellValue(dataRow, 18, result.MaintenanceFunction, contentStyle);

                                // 20. 趋势检查 (新)
                                SetCellValue(dataRow, 19, result.TrendCheck, contentStyle);

                                // 21. 报表检查 (新)
                                SetCellValue(dataRow, 20, result.ReportCheck, contentStyle);

                                // 22. 开始测试时间 (原20)
                                var testTimeStr = result.TestTime.HasValue
                                    ? result.TestTime.Value.ToString("yyyy-MM-dd HH:mm:ss")
                                    : "-";
                                SetCellValue(dataRow, 21, testTimeStr, contentStyle);

                                // 23. 最终测试时间 (原21)
                                var finalTestTimeStr = result.FinalTestTime.HasValue
                                    ? result.FinalTestTime.Value.ToString("yyyy-MM-dd HH:mm:ss")
                                    : "-";
                                SetCellValue(dataRow, 22, finalTestTimeStr, contentStyle);

                                // 24. 测试时长(秒) (原22)
                                var durationCell = dataRow.CreateCell(23);
                                TimeSpan usedTime = TimeSpan.FromSeconds(Math.Round(result.TotalTestDuration, 0));
                                durationCell.SetCellValue($"{(int)usedTime.Hours:D2}小时{usedTime.Minutes:D2}分{usedTime.Seconds:D2}秒");
                                durationCell.CellStyle = contentStyle;

                                // 25. 通道硬点测试结果 (原23)
                                var resultCell = dataRow.CreateCell(24);
                                resultCell.SetCellValue(result.HardPointTestResult ?? "未测试");

                                if (!string.IsNullOrEmpty(result.HardPointTestResult) &&
                                    (result.HardPointTestResult == "通过" || result.HardPointTestResult == "已通过"))
                                {
                                    resultCell.CellStyle = passedStyle;
                                }
                                else
                                {
                                    resultCell.CellStyle = failedStyle;
                                }

                                // 26. 测试结果 (原24)
                                SetCellValue(dataRow, 25, result.ResultText, contentStyle);
                            }
                        }
                        
                        // 保存工作簿到文件
                        using (var fs = new FileStream(filePath, FileMode.Create, FileAccess.Write))
                        {
                            workbook.Write(fs);
                        }
                        
                        return true;
                    }
                    catch (Exception ex)
                    {
                        Console.WriteLine($"导出Excel时出错: {ex.Message}");
                        Application.Current.Dispatcher.Invoke(async () =>
                        {
                            await _messageService.ShowAsync("导出失败", $"导出Excel时出错: {ex.Message}", MessageBoxButton.OK);
                        });
                        return false;
                    }
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"导出测试结果出错: {ex.Message}");
                await _messageService.ShowAsync("导出失败", $"导出测试结果出错: {ex.Message}", MessageBoxButton.OK);
                return false;
            }
        }

        /// <summary>
        /// 导出通道映射到Excel文件
        /// </summary>
        /// <param name="channelMappings">通道映射数据</param>
        /// <param name="filePath">导出文件路径，如果为null则通过文件对话框选择</param>
        /// <returns>导出是否成功</returns>
        public async Task<bool> ExportChannelMapToExcelAsync(IEnumerable<ChannelMapping> channelMappings, string filePath = null)
        {
            try
            {
                if (channelMappings == null || !channelMappings.Any())
                {
                    await _messageService.ShowAsync("导出失败", "没有可导出的通道映射数据", MessageBoxButton.OK);
                    return false;
                }

                // 如果未指定文件路径，则打开保存文件对话框
                if (string.IsNullOrEmpty(filePath))
                {
                    var saveFileDialog = new SaveFileDialog
                    {
                        Filter = "Excel文件 (*.xlsx)|*.xlsx",
                        Title = "保存通道映射表",
                        DefaultExt = "xlsx",
                        FileName = $"通道映射表_{DateTime.Now:yyyyMMdd_HHmmss}"
                    };

                    if (saveFileDialog.ShowDialog() != true)
                    {
                        return false; // 用户取消操作
                    }

                    filePath = saveFileDialog.FileName;
                }

                // 确保有效的文件路径
                if (string.IsNullOrEmpty(filePath))
                {
                    await _messageService.ShowAsync("导出失败", "无效的文件路径", MessageBoxButton.OK);
                    return false;
                }

                // 使用Task.Run在后台线程中执行Excel导出操作
                return await Task.Run(() =>
                {
                    try
                    {
                        // 创建工作簿
                        var workbook = new XSSFWorkbook();
                        
                        // 创建工作表
                        var sheet = workbook.CreateSheet("通道映射表");
                        
                        // 设置列宽
                        sheet.SetColumnWidth(0, 15 * 256);  // 站场名
                        sheet.SetColumnWidth(1, 10 * 256);  // 测试ID
                        sheet.SetColumnWidth(2, 15 * 256);  // 测试批次
                        sheet.SetColumnWidth(3, 25 * 256);  // 变量名称
                        sheet.SetColumnWidth(4, 30 * 256);  // 变量描述
                        sheet.SetColumnWidth(5, 10 * 256);  // 模块类型
                        sheet.SetColumnWidth(6, 25 * 256);  // 测试PLC通道位号
                        sheet.SetColumnWidth(7, 25 * 256);  // 被测PLC通道位号
                        sheet.SetColumnWidth(8, 25 * 256);  // 被测PLC模块型号
                        sheet.SetColumnWidth(9, 15 * 256);  // 供电类型
                        sheet.SetColumnWidth(10, 10 * 256); // 线制
                        
                        // 创建标题行样式
                        var headerStyle = workbook.CreateCellStyle();
                        var headerFont = workbook.CreateFont();
                        headerFont.IsBold = true;
                        headerFont.FontHeightInPoints = 12;
                        headerStyle.SetFont(headerFont);
                        headerStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        headerStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        headerStyle.BorderTop = BorderStyle.Thin;
                        headerStyle.BorderBottom = BorderStyle.Thin;
                        headerStyle.BorderLeft = BorderStyle.Thin;
                        headerStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建内容行样式（带边框）
                        var contentStyle = workbook.CreateCellStyle();
                        contentStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        contentStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        contentStyle.BorderTop = BorderStyle.Thin;
                        contentStyle.BorderBottom = BorderStyle.Thin;
                        contentStyle.BorderLeft = BorderStyle.Thin;
                        contentStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建通过状态样式
                        var passedStyle = workbook.CreateCellStyle();
                        passedStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        passedStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        passedStyle.FillForegroundColor = IndexedColors.LightGreen.Index;
                        passedStyle.FillPattern = FillPattern.SolidForeground;
                        passedStyle.BorderTop = BorderStyle.Thin;
                        passedStyle.BorderBottom = BorderStyle.Thin;
                        passedStyle.BorderLeft = BorderStyle.Thin;
                        passedStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建失败状态样式
                        var failedStyle = workbook.CreateCellStyle();
                        failedStyle.Alignment = NPOI.SS.UserModel.HorizontalAlignment.Center;
                        failedStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                        failedStyle.FillForegroundColor = IndexedColors.Rose.Index;
                        failedStyle.FillPattern = FillPattern.SolidForeground;
                        failedStyle.BorderTop = BorderStyle.Thin;
                        failedStyle.BorderBottom = BorderStyle.Thin;
                        failedStyle.BorderLeft = BorderStyle.Thin;
                        failedStyle.BorderRight = BorderStyle.Thin;
                        
                        // 创建标题行
                        var headerRow = sheet.CreateRow(0);
                        var headers = new[] { 
                            "站场名", "测试ID", "测试批次", "变量名称", "变量描述", 
                            "模块类型", "测试PLC通道位号", "被测PLC通道位号", "被测PLC模块型号", "供电类型", "线制"
                        };
                        
                        for (int i = 0; i < headers.Length; i++)
                        {
                            var cell = headerRow.CreateCell(i);
                            cell.SetCellValue(headers[i]);
                            cell.CellStyle = headerStyle;
                        }
                        
                        // 按测试批次排序通道映射
                        var sortedChannels = channelMappings.OrderBy(c => c.TestBatch).ToList();
                        
                        // 填充数据行
                        int rowIndex = 1;
                        string currentStationName = null;
                        string currentTestBatch = null;
                        int stationNameStartRow = 1;
                        int testBatchStartRow = 1;
                        string currentModuleModel = null;
                        int moduleModelStartRow = 1;
                        
                        // 用于批次颜色的随机数生成器
                        var random = new Random();
                        // 批次颜色字典
                        var batchColors = new Dictionary<string, ICellStyle>();
                        
                        // 为模块类型创建固定颜色样式
                        var moduleTypeStyles = new Dictionary<string, ICellStyle>();
                        
                        // AI模块类型样式 - 浅蓝色
                        var aiStyle = workbook.CreateCellStyle();
                        aiStyle.CloneStyleFrom(contentStyle);
                        var aiColor = new XSSFColor(new byte[] { 173, 216, 230 }); // 浅蓝色
                        ((XSSFCellStyle)aiStyle).SetFillForegroundColor(aiColor);
                        aiStyle.FillPattern = FillPattern.SolidForeground;
                        moduleTypeStyles["ai"] = aiStyle;
                        
                        // AO模块类型样式 - 浅绿色
                        var aoStyle = workbook.CreateCellStyle();
                        aoStyle.CloneStyleFrom(contentStyle);
                        var aoColor = new XSSFColor(new byte[] { 144, 238, 144 }); // 浅绿色
                        ((XSSFCellStyle)aoStyle).SetFillForegroundColor(aoColor);
                        aoStyle.FillPattern = FillPattern.SolidForeground;
                        moduleTypeStyles["ao"] = aoStyle;
                        
                        // DI模块类型样式 - 浅黄色
                        var diStyle = workbook.CreateCellStyle();
                        diStyle.CloneStyleFrom(contentStyle);
                        var diColor = new XSSFColor(new byte[] { 255, 255, 150 }); // 浅黄色
                        ((XSSFCellStyle)diStyle).SetFillForegroundColor(diColor);
                        diStyle.FillPattern = FillPattern.SolidForeground;
                        moduleTypeStyles["di"] = diStyle;
                        
                        // DO模块类型样式 - 浅紫色
                        var doStyle = workbook.CreateCellStyle();
                        doStyle.CloneStyleFrom(contentStyle);
                        var doColor = new XSSFColor(new byte[] { 221, 160, 221 }); // 浅紫色
                        ((XSSFCellStyle)doStyle).SetFillForegroundColor(doColor);
                        doStyle.FillPattern = FillPattern.SolidForeground;
                        moduleTypeStyles["do"] = doStyle;
                        
                        foreach (var channel in sortedChannels)
                        {
                            var dataRow = sheet.CreateRow(rowIndex);
                            
                            // 设置单元格值
                            // 1. 站场名
                            SetCellValue(dataRow, 0, channel.StationName, contentStyle);
                            
                            // 2. 测试ID
                            SetCellValue(dataRow, 1, channel.TestId.ToString(), contentStyle);
                            
                            // 3. 测试批次
                            // 获取或创建该批次的样式
                            ICellStyle batchStyle;
                            if (!batchColors.TryGetValue(channel.TestBatch ?? "", out batchStyle))
                            {
                                // 为新批次创建随机颜色
                                batchStyle = workbook.CreateCellStyle();
                                batchStyle.CloneStyleFrom(contentStyle);
                                // 确保垂直居中设置被保留
                                batchStyle.VerticalAlignment = NPOI.SS.UserModel.VerticalAlignment.Center;
                                // 生成随机颜色 - 使用浅色以便文字清晰可见
                                byte r = (byte)random.Next(200, 256);
                                byte g = (byte)random.Next(200, 256);
                                byte b = (byte)random.Next(200, 256);
                                
                                var color = new XSSFColor(new byte[] { r, g, b });
                                ((XSSFCellStyle)batchStyle).SetFillForegroundColor(color);
                                batchStyle.FillPattern = FillPattern.SolidForeground;
                                
                                batchColors.Add(channel.TestBatch ?? "", batchStyle);
                            }
                            
                            SetCellValue(dataRow, 2, channel.TestBatch, batchStyle);
                            
                            // 4. 变量名称
                            SetCellValue(dataRow, 3, channel.VariableName, contentStyle);
                            
                            // 5. 变量描述
                            SetCellValue(dataRow, 4, channel.VariableDescription, contentStyle);
                            
                            // 6. 模块类型
                            string moduleType = channel.ModuleType?.ToLower() ?? "";
                            if (moduleTypeStyles.TryGetValue(moduleType, out ICellStyle moduleStyle))
                            {
                                SetCellValue(dataRow, 5, channel.ModuleType, moduleStyle);
                            }
                            else
                            {
                                SetCellValue(dataRow, 5, channel.ModuleType, contentStyle);
                            }
                            
                            // 7. 测试PLC通道位号
                            SetCellValue(dataRow, 6, channel.TestPLCChannelTag, contentStyle);
                            
                            // 8. 被测PLC通道位号
                            SetCellValue(dataRow, 7, channel.ChannelTag, contentStyle);

                            // 9. 被测PLC模块型号
                            SetCellValue(dataRow, 8, channel.ModuleName, contentStyle);
                            
                            // 10. 供电类型
                            SetCellValue(dataRow, 9, channel.PowerSupplyType, contentStyle);
                            
                            // 11. 线制
                            SetCellValue(dataRow, 10, channel.WireSystem, contentStyle);
                            
                            // 检查是否需要合并单元格
                            if (channel.StationName != currentStationName)
                            {
                                // 如果有前一个站场名，则合并单元格
                                if (currentStationName != null && rowIndex > stationNameStartRow)
                                {
                                    sheet.AddMergedRegion(new CellRangeAddress(stationNameStartRow, rowIndex - 1, 0, 0));
                                }
                                
                                // 设置新的站场名和起始行
                                currentStationName = channel.StationName;
                                stationNameStartRow = rowIndex;
                            }
                            
                            // 检查是否需要合并测试批次单元格
                            if (channel.TestBatch != currentTestBatch)
                            {
                                // 如果有前一个测试批次，则合并单元格
                                if (currentTestBatch != null && rowIndex > testBatchStartRow)
                                {
                                    sheet.AddMergedRegion(new CellRangeAddress(testBatchStartRow, rowIndex - 1, 2, 2));
                                }
                                
                                // 设置新的测试批次和起始行
                                currentTestBatch = channel.TestBatch;
                                testBatchStartRow = rowIndex;
                            }

                            // 检查是否需要合并被测PLC模块型号单元格
                            if (channel.ModuleName != currentModuleModel)
                            {
                                if (currentModuleModel != null && rowIndex > moduleModelStartRow)
                                {
                                    sheet.AddMergedRegion(new CellRangeAddress(moduleModelStartRow, rowIndex - 1, 8, 8));
                                }

                                currentModuleModel = channel.ModuleName;
                                moduleModelStartRow = rowIndex;
                            }
                            
                            rowIndex++;
                        }
                        
                        // 处理最后一组站场名合并
                        if (currentStationName != null && rowIndex > stationNameStartRow)
                        {
                            sheet.AddMergedRegion(new CellRangeAddress(stationNameStartRow, rowIndex - 1, 0, 0));
                        }
                        
                        // 处理最后一组测试批次合并
                        if (currentTestBatch != null && rowIndex > testBatchStartRow)
                        {
                            sheet.AddMergedRegion(new CellRangeAddress(testBatchStartRow, rowIndex - 1, 2, 2));
                        }

                        // 处理最后一组被测PLC模块型号合并
                        if (currentModuleModel != null && rowIndex > moduleModelStartRow)
                        {
                            sheet.AddMergedRegion(new CellRangeAddress(moduleModelStartRow, rowIndex - 1, 8, 8));
                        }
                        
                        // 保存工作簿到文件
                        using (var fs = new FileStream(filePath, FileMode.Create, FileAccess.Write))
                        {
                            workbook.Write(fs);
                        }
                        
                        return true;
                    }
                    catch (Exception ex)
                    {
                        // 记录错误日志
                        System.Diagnostics.Debug.WriteLine($"导出通道映射表失败: {ex.Message}");
                        return false;
                    }
                });
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("导出失败", $"导出通道映射表时发生错误: {ex.Message}", MessageBoxButton.OK);
                return false;
            }
        }


        /// <summary>
        /// 设置单元格值并应用样式
        /// </summary>
        /// <param name="row">行</param>
        /// <param name="index">列索引</param>
        /// <param name="value">单元格值</param>
        /// <param name="style">单元格样式</param>
        private void SetCellValue(IRow row, int index, string value, ICellStyle style)
        {
            var cell = row.CreateCell(index);
            cell.SetCellValue(value ?? string.Empty);
            cell.CellStyle = style;
        }
        
        /// <summary>
        /// 设置浮点数单元格值并应用样式
        /// </summary>
        /// <param name="row">行</param>
        /// <param name="index">列索引</param>
        /// <param name="value">单元格值</param>
        /// <param name="style">单元格样式</param>
        private void SetDoubleValue(IRow row, int index, double? value, ICellStyle style)
        {
            var cell = row.CreateCell(index);
            if (value.HasValue)
            {
                if (value is Double.NaN)
                {
                    cell.SetCellValue("N/A");
                }
                else
                {
                    cell.SetCellValue(value.Value);
                }
            }
            else
            {
                cell.SetCellValue("N/A");
            }
            cell.CellStyle = style;
        }
    }
} 