using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using System.Net.Http;
using System.Text.Json;
using FatFullVersion.Models;
using FatFullVersion.Services.Interfaces;
using System.Windows;
using FatFullVersion.IServices;

namespace FatFullVersion.Services
{
    /// <summary>
    /// API点表数据服务实现类，通过API获取点表数据
    /// </summary>
    public class ApiPointDataService : IPointDataService
    {
        private readonly HttpClient _httpClient;
        private List<ExcelPointData> _currentPointDataList = new List<ExcelPointData>();
        private readonly string _baseApiUrl;

        public ApiPointDataService(string baseApiUrl = "https://api.example.com/")
        {
            _baseApiUrl = baseApiUrl;
            _httpClient = new HttpClient();
            _httpClient.BaseAddress = new Uri(_baseApiUrl);
            _httpClient.DefaultRequestHeaders.Accept.Add(new System.Net.Http.Headers.MediaTypeWithQualityHeaderValue("application/json"));
        }

        /// <summary>
        /// 从API获取点表数据
        /// 参数必须包含：
        /// - endpoint: API端点路径
        /// - 可能的其他参数，如查询条件
        /// </summary>
        /// <param name="parameters">获取参数</param>
        /// <returns>点表对象列表</returns>
        public async Task<IEnumerable<ExcelPointData>> GetPointDataAsync(Dictionary<string, object> parameters = null)
        {
            try
            {
                // 验证参数
                if (parameters == null)
                {
                    throw new ArgumentException("必须提供API请求参数");
                }

                string endpoint = parameters.ContainsKey("endpoint") ? parameters["endpoint"].ToString() : "pointdata";
                string queryParams = "";

                // 构建查询参数
                if (parameters.ContainsKey("queryParams") && parameters["queryParams"] is Dictionary<string, string> queryDict)
                {
                    queryParams = "?" + string.Join("&", queryDict.Select(kv => $"{kv.Key}={kv.Value}"));
                }

                // 发送请求
                var response = await _httpClient.GetAsync($"{endpoint}{queryParams}");
                response.EnsureSuccessStatusCode();

                // 读取响应内容
                var content = await response.Content.ReadAsStringAsync();
                
                // 反序列化为ExcelPointData对象列表
                var options = new JsonSerializerOptions
                {
                    PropertyNameCaseInsensitive = true
                };
                
                var pointList = JsonSerializer.Deserialize<List<ExcelPointData>>(content, options);
                return pointList ?? new List<ExcelPointData>();
            }
            catch (Exception ex)
            {
                Console.WriteLine($"从API获取点表数据失败: {ex.Message}");
                throw;
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
            }

            return Task.FromResult(result);
        }

        /// <summary>
        /// 保存点表数据到服务器
        /// </summary>
        /// <param name="pointDataList">点表数据列表</param>
        /// <returns>操作结果</returns>
        public async Task<bool> SavePointDataAsync(IEnumerable<ExcelPointData> pointDataList)
        {
            try
            {
                // 在内存中保存一份
                _currentPointDataList = pointDataList.ToList();
                
                // 为每条数据设置更新时间
                foreach (var point in _currentPointDataList)
                {
                    if (point.CreatedTime == default)
                    {
                        point.CreatedTime = DateTime.Now;
                    }
                    
                    point.UpdatedTime = DateTime.Now;
                }
                
                // 序列化数据
                var options = new JsonSerializerOptions
                {
                    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
                };
                
                var jsonContent = JsonSerializer.Serialize(pointDataList, options);
                var content = new StringContent(jsonContent, System.Text.Encoding.UTF8, "application/json");
                
                // 发送请求
                var response = await _httpClient.PostAsync("pointdata", content);
                response.EnsureSuccessStatusCode();
                
                return true;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"保存点表数据到服务器失败: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 导入点表配置
        /// </summary>
        /// <param name="importAction">导入完成后的回调动作，参数为导入的点表数据列表</param>
        public async Task ImportPointConfigurationAsync(Action<IEnumerable<ExcelPointData>> importAction = null)
        {
            try
            {
                // 显示加载对话框或提示
                MessageBox.Show("正在从API获取点表数据...", "导入中", MessageBoxButton.OK, MessageBoxImage.Information);
                
                // 调用API获取数据
                var parameters = new Dictionary<string, object>
                {
                    { "endpoint", "pointdata/import" }
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
                MessageBox.Show($"导入点表配置失败: {ex.Message}", "导入失败", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }
    }
} 