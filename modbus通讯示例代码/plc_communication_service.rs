use crate::service::communication_management_service::plc_communication_service::plc_result::PlcResult;
use crate::utils::byte_order::ByteOrder;

/// PLC通信服务接口
/// 定义了与PLC设备通信的各种异步操作
pub trait PlcCommunicationService {
    /// 连接PLC
    /// 
    /// # 参数
    /// * `ip_address` - PLC的IP地址
    /// * `port` - PLC的端口号
    async fn connect_async(&self) -> PlcResult<()>;
    
    /// 断开PLC连接
    async fn dic_connect_async(&self) -> PlcResult<()>;

    ///重连plc
    async fn reconnect(&self) -> PlcResult<()>;

    ///通过循环读取固定地址来检查并保持连接
    async fn check_connection(&self,check_address:&str) -> PlcResult<()>;

    ///获取连接状态
    async fn get_connection_state(&self)->PlcResult<bool>;

    /// 读取单个单浮点型模拟量
    /// 
    /// # 参数
    /// * `address` - 要读取的PLC地址
    async fn read_signal_analog_value_async(&self, address: &str) -> PlcResult<f32>;
    
    /// 写入单个单浮点型模拟量
    /// 
    /// # 参数
    /// * `address` - 要写入的PLC地址
    /// * `value` - 要写入的浮点值
    async fn write_signal_analog_value_async(&self, address: &str, value: f32) -> PlcResult<()>;
    
    /// 读取多个单浮点型模拟量
    /// 
    /// # 参数
    /// * `address` - 起始PLC地址
    /// * `count` - 要读取的数量
    async fn read_signal_analog_values_async(&self, address: &str, count: i32) -> PlcResult<Vec<f32>>;
    
    /// 写入多个单浮点型模拟量
    /// 
    /// # 参数
    /// * `address` - 起始PLC地址
    /// * `values` - 要写入的浮点值数组
    async fn write_signal_analog_values_async(&self, address: &str, values: Vec<f32>) -> PlcResult<()>;
    
    /// 读取单个布尔型数据
    /// 
    /// # 参数
    /// * `address` - 要读取的PLC地址
    async fn read_signal_boolean_value_async(&self, address: &str) -> PlcResult<bool>;
    
    /// 写入单个布尔型数据
    /// 
    /// # 参数
    /// * `address` - 要写入的PLC地址
    /// * `value` - 要写入的布尔值
    async fn write_signal_boolean_value_async(&self, address: &str, value: bool) -> PlcResult<()>;
    
    /// 读取多个布尔型数据
    /// 
    /// # 参数
    /// * `address` - 起始PLC地址
    /// * `count` - 要读取的数量
    async fn read_signal_boolean_values_async(&self, address: &str, count: i32) -> PlcResult<Vec<bool>>;
    
    /// 写入多个布尔型数据
    /// 
    /// # 参数
    /// * `address` - 起始PLC地址
    /// * `values` - 要写入的布尔值数组
    async fn write_signal_boolean_values_async(&self, address: &str, values: Vec<bool>) -> PlcResult<()>;
    
    /// 读取单个布尔型数据 (别名)
    /// 
    /// # 参数
    /// * `address` - 要读取的PLC地址
    async fn read_signal_bool_async(&self, address: &str) -> PlcResult<bool>;
    
    /// 写入单个布尔型数据 (别名)
    /// 
    /// # 参数
    /// * `address` - 要写入的PLC地址
    /// * `value` - 要写入的布尔值
    async fn write_signal_bool_async(&self, address: &str, value: bool) -> PlcResult<()>;
    
    /// 读取单个整数值
    /// 
    /// # 参数
    /// * `address` - 要读取的PLC地址
    async fn read_signal_int_async(&self, address: &str) -> PlcResult<i32>;
    
    /// 写入单个整数值
    /// 
    /// # 参数
    /// * `address` - 要写入的PLC地址
    /// * `value` - 要写入的整数值
    async fn write_signal_int_async(&self, address: &str, value: i32) -> PlcResult<()>;
    
    /// 批量读取多个整数值
    /// 
    /// # 参数
    /// * `addresses` - 要读取的PLC地址列表
    async fn read_signal_int_batch_async(&self, addresses: Vec<&str>) -> PlcResult<Vec<i32>>;
    
    /// 批量写入多个整数值
    /// 
    /// # 参数
    /// * `addresses` - 要写入的PLC地址列表
    /// * `values` - 要写入的整数值列表
    async fn write_signal_int_batch_async(&self, addresses: Vec<&str>, values: Vec<i32>) -> PlcResult<()>;
    
    /// 获取当前使用的浮点数字节顺序
    fn get_byte_order(&self) -> ByteOrder;
}