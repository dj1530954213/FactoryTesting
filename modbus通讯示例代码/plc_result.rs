use serde::{Deserialize, Serialize};

/// PLC通信结果结构体，类似于标准库的Result但包含更多信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcResult<T> {
    /// 操作是否成功
    pub success: bool,
    /// 错误信息（如果操作失败）
    pub error_message: Option<String>,
    /// 返回的数据
    pub data: Option<T>,
}

impl<T> PlcResult<T> {
    /// 创建一个成功的结果
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            error_message: None,
            data: Some(data),
        }
    }
    
    /// 创建一个失败的结果
    pub fn err(message: String) -> Self {
        Self {
            success: false,
            error_message: Some(message),
            data: None,
        }
    }
    
    /// 检查结果是否成功
    pub fn is_ok(&self) -> bool {
        self.success
    }
    
    /// 检查结果是否失败
    pub fn is_err(&self) -> bool {
        !self.success
    }
    
    /// 获取数据，如果失败则返回None
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }
    
    /// 将结果映射到新类型
    /// U: 一个新的类型，表示转换后的数据类型
    /// F: 一个函数类型，满足 FnOnce(T) -> U 特性，表示它接收一个T类型参数并返回U类型结果
    /// self: 接收结构体所有权（非引用）
    /// op: F类型的函数/闭包，用于转换数据
    /// PlcResult<U>: 返回值类型，与原始的PlcResult<T>结构相同，但包装的数据类型变成了U
    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> PlcResult<U> {
        ///关于map的使用示例1
        ///假设我们有一个成功的浮点数值结果
        /// let temperature = PlcResult::ok(25.5); // PlcResult<f32>
        /// 我们想将温度从摄氏度转换为华氏度
        /// let fahrenheit = temperature.map(|celsius| celsius * 9.0/5.0 + 32.0); // PlcResult<f32>
        /// 如果temperature是成功的，fahrenheit也是成功的，且包含转换后的值
        /// 例如，fahrenheit.data 可能是 Some(77.9)
        ///
        /// MAP的使用示例2
        /// // 假设我们有一个成功的浮点数值结果
        /// let temperature = PlcResult::ok(25.5); // PlcResult<f32>
        /// 我们想把它转换成一个状态字符串
        /// let status = temperature.map(|temp| {
        ///     if temp > 30.0 {
        ///         "过热".to_string()
        ///     } else if temp < 10.0 {
        ///         "过冷".to_string()
        ///     } else {
        ///         "正常".to_string()
        ///     }
        /// }); // PlcResult<String>
        ///现在status是PlcResult<String>类型，包含温度状态描述
        ///
        /// MAP的使用示例3
        /// 假设我们有一个失败的结果
        /// let error_result = PlcResult::<f32>::err("读取温度失败".to_string());
        ///尝试转换它，但因为是错误结果，转换函数不会被调用
        /// let converted = error_result.map(|temp| temp * 2.0);
        /// converted仍然是错误结果，error_message保持不变，data仍然是None
        /// assert!(converted.is_err());
        /// assert_eq!(converted.error_message, Some("读取温度失败".to_string()));
        /// assert_eq!(converted.data, None);
        if self.success {
            PlcResult {
                success: true,
                error_message: None,
                //data: self.data.map(op): 重点在这里
                // self.data是Option<T>类型
                // Option有一个内置的map方法，可以对Some包含的值应用函数
                // 当self.data是Some(value)时，self.data.map(op)会计算Some(op(value))
                // 当self.data是None时，结果仍然是None，操作函数不会被调用
                data: self.data.map(op),
            }
        } else {
            PlcResult {
                success: false,
                error_message: self.error_message,
                data: None,
            }
        }
    }
} 