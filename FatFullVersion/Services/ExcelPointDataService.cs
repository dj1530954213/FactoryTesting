using System;
using System.Collections.Generic;
using System.Diagnostics.Eventing.Reader;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using FatFullVersion.Models;
using FatFullVersion.Services.Interfaces;
using System.Text;
using Microsoft.Win32;
using System.Windows;
using NPOI.SS.UserModel;
using NPOI.XSSF.UserModel;
using NPOI.HSSF.UserModel;
using FatFullVersion.IServices;

namespace FatFullVersion.Services
{
    /// <summary>
    /// Excel点表数据服务实现类
    /// </summary>
    public class ExcelPointDataService : IPointDataService
    {
        // 存储当前导入的点表数据
        private List<ExcelPointData> _currentPointDataList = new List<ExcelPointData>();

        /// <summary>
        /// 获取点表数据
        /// 参数必须包含：
        /// - filePath: Excel文件路径
        /// - sheetName: (可选) 工作表名称，如果未指定则使用第一个工作表
        /// 或者
        /// - fileContent: 文件内容（字节数组）
        /// - fileType: 文件类型，支持"xls"或"xlsx"
        /// </summary>
        /// <param name="parameters">获取参数</param>
        /// <returns>点表对象列表</returns>
        public async Task<IEnumerable<ExcelPointData>> GetPointDataAsync(Dictionary<string, object> parameters = null)
        {
            // 验证参数
            if (parameters == null)
            {
                throw new ArgumentException("必须提供参数");
            }

            // 从文件路径加载
            if (parameters.ContainsKey("filePath"))
            {
                string filePath = parameters["filePath"].ToString();
                string sheetName = parameters.ContainsKey("sheetName") ? parameters["sheetName"].ToString() : null;

                // 验证文件存在
                if (!File.Exists(filePath))
                {
                    throw new FileNotFoundException($"找不到指定的Excel文件: {filePath}");
                }

                // 根据文件扩展名判断文件类型
                string extension = Path.GetExtension(filePath).ToLower();
                if (extension == ".xlsx" || extension == ".xls")
                {
                    // 读取Excel文件
                    using (var fileStream = new FileStream(filePath, FileMode.Open, FileAccess.Read))
                    {
                        byte[] buffer = new byte[fileStream.Length];
                        await fileStream.ReadAsync(buffer, 0, (int)fileStream.Length);
                        
                        string fileType = extension == ".xlsx" ? "xlsx" : "xls";
                        return await ParseExcelContentAsync(buffer, fileType, sheetName);
                    }
                }
                else
                {
                    throw new ArgumentException("不支持的文件类型，仅支持Excel文件 (.xls, .xlsx)");
                }
            }
            // 从文件内容加载
            else if (parameters.ContainsKey("fileContent"))
            {
                string fileType = parameters.ContainsKey("fileType") ? parameters["fileType"].ToString().ToLower() : "xlsx";
                string sheetName = parameters.ContainsKey("sheetName") ? parameters["sheetName"].ToString() : null;
                
                if (fileType == "xlsx" || fileType == "xls")
                {
                    if (parameters["fileContent"] is byte[] bytes)
                    {
                        return await ParseExcelContentAsync(bytes, fileType, sheetName);
                    }
                    else
                    {
                        throw new ArgumentException("fileContent参数必须是字节数组");
                    }
                }
                else
                {
                    throw new ArgumentException("不支持的文件类型，仅支持Excel文件类型 (xlsx, xls)");
                }
            }
            else
            {
                throw new ArgumentException("必须提供filePath或fileContent参数");
            }
        }

        /// <summary>
        /// 验证点表数据的有效性
        /// </summary>
        /// <param name="pointDataList">需要验证的点表数据列表</param>
        /// <returns>验证结果，包含错误信息</returns>
        public Task<ValidationResult> ValidatePointDataAsync(IEnumerable<ExcelPointData> pointDataList)
        {
            var result = new ValidationResult { IsValid = true };

            // 检查点表数据是否为空
            if (pointDataList == null || !pointDataList.Any())
            {
                result.IsValid = false;
                result.ErrorMessages.Add("点表数据不能为空");
                return Task.FromResult(result);
            }

            // 检查必填字段
            foreach (var point in pointDataList)
            {
                if (string.IsNullOrWhiteSpace(point.ChannelTag))
                {
                    result.IsValid = false;
                    result.ErrorMessages.Add($"序号 {point.SerialNumber} 的通道位号不能为空");
                }

                if (string.IsNullOrWhiteSpace(point.VariableName))
                {
                    result.IsValid = false;
                    result.ErrorMessages.Add($"序号 {point.SerialNumber} 的变量名称不能为空");
                }
                
                // 检查量程设置合理性
                if (point.RangeLowerLimitValue >= point.RangeUpperLimitValue)
                {
                    result.IsValid = false;
                    result.ErrorMessages.Add($"序号 {point.SerialNumber} 的量程设置不合理，低限必须小于高限");
                }
                
                // 检查报警限值合理性
                if (point.LowLowLimit > point.LowLimit || 
                    point.LowLimit > point.HighLimit || 
                    point.HighLimit > point.HighHighLimit)
                {
                    result.IsValid = false;
                    result.ErrorMessages.Add($"序号 {point.SerialNumber} 的报警限值设置不合理，应满足低低限 ≤ 低限 ≤ 高限 ≤ 高高限");
                }
            }

            return Task.FromResult(result);
        }

        /// <summary>
        /// 保存点表数据
        /// </summary>
        /// <param name="pointDataList">点表数据列表</param>
        /// <returns>操作结果</returns>
        public Task<bool> SavePointDataAsync(IEnumerable<ExcelPointData> pointDataList)
        {
            // 在应用内存中保存点表数据，不依赖文件存储
            _currentPointDataList = pointDataList.ToList();
            
            // 为每条数据设置创建时间和更新时间
            foreach (var point in _currentPointDataList)
            {
                if (point.CreatedTime == default)
                {
                    point.CreatedTime = DateTime.Now;
                }
                
                point.UpdatedTime = DateTime.Now;
            }
            
            Console.WriteLine($"已保存 {_currentPointDataList.Count} 条点表数据到内存");
            
            return Task.FromResult(true);
        }

        /// <summary>
        /// 导入点表配置文件
        /// </summary>
        /// <param name="importAction">导入完成后的回调动作，参数为导入的点表数据列表</param>
        public async Task ImportPointConfigurationAsync(Action<IEnumerable<ExcelPointData>> importAction = null)
        {
            try
            {
                // 创建打开文件对话框
                var openFileDialog = new OpenFileDialog
                {
                    Filter = "Excel文件 (*.xlsx;*.xls)|*.xlsx;*.xls|所有文件 (*.*)|*.*",
                    Title = "选择点表配置文件"
                };

                // 显示对话框
                if (openFileDialog.ShowDialog() == true)
                {
                    string filePath = openFileDialog.FileName;
                    
                    // 读取文件内容
                    byte[] fileContent;
                    using (var fileStream = new FileStream(filePath, FileMode.Open, FileAccess.Read))
                    {
                        fileContent = new byte[fileStream.Length];
                        await fileStream.ReadAsync(fileContent, 0, (int)fileStream.Length);
                    }
                    
                    // 获取文件类型
                    string fileType = Path.GetExtension(filePath).ToLower() == ".xlsx" ? "xlsx" : "xls";
                    
                    try
                    {
                        // 解析点表数据
                        var parameters = new Dictionary<string, object>
                        {
                            { "fileContent", fileContent },
                            { "fileType", fileType }
                        };
                        
                        var pointDataList = await GetPointDataAsync(parameters);
                        
                        // 保存到内存中
                        await SavePointDataAsync(pointDataList);
                        
                        // 调用回调，通知导入完成
                        importAction?.Invoke(pointDataList);
                        
                        MessageBox.Show($"成功导入 {pointDataList.Count()} 条点表数据", "导入成功", MessageBoxButton.OK, MessageBoxImage.Information);
                    }
                    catch (Exception ex)
                    {
                        MessageBox.Show($"解析点表数据失败: {ex.Message}", "导入失败", MessageBoxButton.OK, MessageBoxImage.Error);
                    }
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"导入点表配置失败: {ex.Message}", "导入失败", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        #region 私有方法

        /// <summary>
        /// 解析Excel内容为点表数据列表
        /// </summary>
        /// <param name="excelContent">Excel文件内容</param>
        /// <param name="fileType">文件类型：xlsx或xls</param>
        /// <param name="sheetName">工作表名称，如果为空则使用第一个工作表</param>
        /// <returns>点表数据列表</returns>
        private async Task<IEnumerable<ExcelPointData>> ParseExcelContentAsync(byte[] excelContent, string fileType, string sheetName = null)
        {
            return await Task.Run(() => {
                var result = new List<ExcelPointData>();
                
                // 创建工作簿对象
                IWorkbook workbook;
                using (var stream = new MemoryStream(excelContent))
                {
                    // 根据文件类型创建对应的工作簿
                    if (fileType == "xlsx")
                    {
                        workbook = new XSSFWorkbook(stream);
                    }
                    else
                    {
                        workbook = new HSSFWorkbook(stream);
                    }
                    
                    // 确定要使用的工作表
                    ISheet sheet = null;
                    
                    if (!string.IsNullOrEmpty(sheetName))
                    {
                        // 按名称查找工作表
                        sheet = workbook.GetSheet(sheetName);
                        
                        if (sheet == null)
                        {
                            throw new ArgumentException($"找不到名为 '{sheetName}' 的工作表");
                        }
                    }
                    else
                    {
                        // 使用第一个工作表
                        sheet = workbook.GetSheetAt(0);
                        
                        if (sheet == null)
                        {
                            throw new ArgumentException("Excel文件不包含工作表");
                        }
                    }
                    
                    // 确保至少有一行数据（标题行）
                    if (sheet.LastRowNum < 1)
                    {
                        throw new ArgumentException("Excel工作表不包含数据行");
                    }
                    
                    // 第一行是标题行，从第二行开始解析数据
                    for (int i = 1; i <= sheet.LastRowNum; i++)
                    {
                        IRow row = sheet.GetRow(i);
                        
                        if (row == null)
                        {
                            continue;
                        }
                        
                        // 创建并填充ExcelPointData对象
                        var excelPoint = new ExcelPointData();
                        
                        // 解析所有字段
                        // 序号
                        excelPoint.SerialNumber = GetIntCellValue(row.GetCell(0));
                        
                        // 模块名称
                        excelPoint.ModuleName = GetStringCellValue(row.GetCell(1));
                        
                        // 模块类型
                        excelPoint.ModuleType = GetStringCellValue(row.GetCell(2));
                        
                        // 供电类型(需要区分关于DI点位如果是非安全型的就将其分配到有源中也就是测试PLC的DO1中)
                        if (GetStringCellValue(row.GetCell(2)).Contains("DI")&& !GetStringCellValue(row.GetCell(1)).Contains("S"))
                        {
                            excelPoint.PowerSupplyType = "有源";
                        }
                        else
                        {
                            excelPoint.PowerSupplyType = GetStringCellValue(row.GetCell(3));
                        }  
                        
                        // 线制
                        excelPoint.WireSystem = GetStringCellValue(row.GetCell(4));
                        
                        // 通道位号
                        excelPoint.ChannelTag = GetStringCellValue(row.GetCell(5));
                        
                        // 位号
                        excelPoint.Tag = GetStringCellValue(row.GetCell(6));
                        
                        // 场站名
                        excelPoint.StationName = GetStringCellValue(row.GetCell(6));
                        
                        // 变量名称
                        excelPoint.VariableName = GetStringCellValue(row.GetCell(8));
                        
                        // 变量描述
                        excelPoint.VariableDescription = GetStringCellValue(row.GetCell(9));
                        
                        // 数据类型
                        excelPoint.DataType = GetStringCellValue(row.GetCell(10));
                        
                        // 读写属性
                        excelPoint.AccessProperty = GetStringCellValue(row.GetCell(11));
                        
                        // 保存历史
                        excelPoint.SaveHistory = GetStringCellValue(row.GetCell(12));
                        
                        // 掉电保护
                        excelPoint.PowerFailureProtection = GetStringCellValue(row.GetCell(13));
                        //预留点位量程上下限默认值赋值
                        if (excelPoint.VariableName.Contains("YLDW"))
                        {
                            // 量程低限
                            excelPoint.RangeLowerLimit = "0";
                            // 量程高限
                            excelPoint.RangeUpperLimit = "100";
                        }
                        else
                        {
                            // 量程低限
                            excelPoint.RangeLowerLimit = GetStringCellValue(row.GetCell(14));
                            // 量程高限
                            excelPoint.RangeUpperLimit = GetStringCellValue(row.GetCell(15));
                        }
                        // SLL设定值
                        excelPoint.SLLSetValue = GetStringCellValue(row.GetCell(16));
                        
                        // SLL设定点位
                        excelPoint.SLLSetPoint = GetStringCellValue(row.GetCell(17));
                        
                        // SLL设定点位_PLC地址
                        excelPoint.SLLSetPointPLCAddress = GetStringCellValue(row.GetCell(18));
                        
                        // SLL设定点位_通讯地址
                        excelPoint.SLLSetPointCommAddress = GetStringCellValue(row.GetCell(19));
                        
                        // SL设定值
                        excelPoint.SLSetValue = GetStringCellValue(row.GetCell(20));
                        
                        // SL设定点位
                        excelPoint.SLSetPoint = GetStringCellValue(row.GetCell(21));
                        
                        // SL设定点位_PLC地址
                        excelPoint.SLSetPointPLCAddress = GetStringCellValue(row.GetCell(22));
                        
                        // SL设定点位_通讯地址
                        excelPoint.SLSetPointCommAddress = GetStringCellValue(row.GetCell(23));
                        
                        // SH设定值
                        excelPoint.SHSetValue = GetStringCellValue(row.GetCell(24));
                        
                        // SH设定点位
                        excelPoint.SHSetPoint = GetStringCellValue(row.GetCell(25));
                        
                        // SH设定点位_PLC地址
                        excelPoint.SHSetPointPLCAddress = GetStringCellValue(row.GetCell(26));
                        
                        // SH设定点位_通讯地址
                        excelPoint.SHSetPointCommAddress = GetStringCellValue(row.GetCell(27));
                        
                        // SHH设定值
                        excelPoint.SHHSetValue = GetStringCellValue(row.GetCell(28));
                        
                        // SHH设定点位
                        excelPoint.SHHSetPoint = GetStringCellValue(row.GetCell(29));
                        
                        // SHH设定点位_PLC地址
                        excelPoint.SHHSetPointPLCAddress = GetStringCellValue(row.GetCell(30));
                        
                        // SHH设定点位_通讯地址
                        excelPoint.SHHSetPointCommAddress = GetStringCellValue(row.GetCell(31));
                        
                        // LL报警
                        excelPoint.LLAlarm = GetStringCellValue(row.GetCell(32));
                        
                        // LL报警_PLC地址
                        excelPoint.LLAlarmPLCAddress = GetStringCellValue(row.GetCell(33));
                        
                        // LL报警_通讯地址
                        excelPoint.LLAlarmCommAddress = GetStringCellValue(row.GetCell(34));
                        
                        // L报警
                        excelPoint.LAlarm = GetStringCellValue(row.GetCell(35));
                        
                        // L报警_PLC地址
                        excelPoint.LAlarmPLCAddress = GetStringCellValue(row.GetCell(36));
                        
                        // L报警_通讯地址
                        excelPoint.LAlarmCommAddress = GetStringCellValue(row.GetCell(37));
                        
                        // H报警
                        excelPoint.HAlarm = GetStringCellValue(row.GetCell(38));
                        
                        // H报警_PLC地址
                        excelPoint.HAlarmPLCAddress = GetStringCellValue(row.GetCell(39));
                        
                        // H报警_通讯地址
                        excelPoint.HAlarmCommAddress = GetStringCellValue(row.GetCell(40));
                        
                        // HH报警
                        excelPoint.HHAlarm = GetStringCellValue(row.GetCell(41));
                        
                        // HH报警_PLC地址
                        excelPoint.HHAlarmPLCAddress = GetStringCellValue(row.GetCell(42));
                        
                        // HH报警_通讯地址
                        excelPoint.HHAlarmCommAddress = GetStringCellValue(row.GetCell(43));
                        
                        // 维护值设定
                        excelPoint.MaintenanceValueSetting = GetStringCellValue(row.GetCell(44));
                        
                        // 维护值设定点位
                        excelPoint.MaintenanceValueSetPoint = GetStringCellValue(row.GetCell(45));
                        
                        // 维护值设定点位_PLC地址
                        excelPoint.MaintenanceValueSetPointPLCAddress = GetStringCellValue(row.GetCell(46));
                        
                        // 维护值设定点位_通讯地址
                        excelPoint.MaintenanceValueSetPointCommAddress = GetStringCellValue(row.GetCell(47));
                        
                        // 维护使能开关点位
                        excelPoint.MaintenanceEnableSwitchPoint = GetStringCellValue(row.GetCell(48));
                        
                        // 维护使能开关点位_PLC地址
                        excelPoint.MaintenanceEnableSwitchPointPLCAddress = GetStringCellValue(row.GetCell(49));
                        
                        // 维护使能开关点位_通讯地址
                        excelPoint.MaintenanceEnableSwitchPointCommAddress = GetStringCellValue(row.GetCell(50));
                        
                        // PLC绝对地址
                        excelPoint.PLCAbsoluteAddress = GetStringCellValue(row.GetCell(51));
                        
                        // 上位机通讯地址
                        excelPoint.CommunicationAddress = GetStringCellValue(row.GetCell(52));
                        
                        // 设置创建时间
                        excelPoint.CreatedTime = DateTime.Now;
                        
                        // 添加到结果列表
                        result.Add(excelPoint);
                    }
                    
                    return result;
                }
            });
        }

        /// <summary>
        /// 获取单元格的字符串值，处理各种类型
        /// </summary>
        /// <param name="cell">单元格</param>
        /// <returns>字符串值</returns>
        private string GetStringCellValue(ICell cell)
        {
            if (cell == null)
                return string.Empty;
            
            switch (cell.CellType)
            {
                case CellType.String:
                    return cell.StringCellValue;
                
                case CellType.Numeric:
                    if (DateUtil.IsCellDateFormatted(cell))
                    {
                        return cell.DateCellValue.ToString("yyyy-MM-dd");
                    }
                    else
                    {
                        return cell.NumericCellValue.ToString();
                    }
                
                case CellType.Boolean:
                    return cell.BooleanCellValue.ToString();
                
                case CellType.Formula:
                    switch (cell.CachedFormulaResultType)
                    {
                        case CellType.String:
                            return cell.StringCellValue;
                        
                        case CellType.Numeric:
                            return cell.NumericCellValue.ToString();
                        
                        default:
                            return cell.ToString();
                    }
                
                default:
                    return string.Empty;
            }
        }
        
        /// <summary>
        /// 获取单元格的整数值
        /// </summary>
        /// <param name="cell">单元格</param>
        /// <returns>整数值，如果转换失败则返回0</returns>
        private int GetIntCellValue(ICell cell)
        {
            if (cell == null)
                return 0;
            
            try
            {
                switch (cell.CellType)
                {
                    case CellType.Numeric:
                        return (int)cell.NumericCellValue;
                    
                    case CellType.String:
                        if (int.TryParse(cell.StringCellValue, out int result))
                        {
                            return result;
                        }
                        return 0;
                    
                    case CellType.Formula:
                        if (cell.CachedFormulaResultType == CellType.Numeric)
                        {
                            return (int)cell.NumericCellValue;
                        }
                        else if (cell.CachedFormulaResultType == CellType.String &&
                                int.TryParse(cell.StringCellValue, out int formulaResult))
                        {
                            return formulaResult;
                        }
                        return 0;
                    
                    default:
                        return 0;
                }
            }
            catch
            {
                return 0;
            }
        }

        #endregion
    }
} 