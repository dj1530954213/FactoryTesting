use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_modbus::prelude::*;
use tokio_modbus::client::Context as ModbusContext;
use tokio::task::JoinHandle;
use tokio::sync::oneshot;
use crate::plc_communication_service::PlcCommunicationService;
use crate::service::communication_management_service::plc_communication_service::plc_result::PlcResult;
use crate::utils::byte_order::{ByteOrder, ByteOrderConverter, ByteOrderUtils};

pub struct ModbusTcpClient {
    socket_addr: SocketAddr,
    slave: Slave,
    connection_state: Arc<Mutex<bool>>,
    context: Arc<Mutex<Option<ModbusContext>>>,
    check_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    cancel_channel: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    byte_order: ByteOrder,
}

impl ModbusTcpClient {
    /// 创建新的Modbus TCP客户端
    /// 
    /// # 参数
    /// * `socket_addr` - 服务器地址
    /// * `slave_id` - 从站ID，默认为1
    /// * `byte_order` - 浮点数字节顺序，默认为CDAB
    pub fn new(socket_addr: SocketAddr, slave_id: u8, byte_order: ByteOrder) -> ModbusTcpClient {
        ModbusTcpClient {
            socket_addr,
            slave: Slave(slave_id),
            connection_state: Arc::new(Mutex::new(false)),
            context: Arc::new(Mutex::new(None)),
            check_task: Arc::new(Mutex::new(None)),
            cancel_channel: Arc::new(Mutex::new(None)),
            byte_order,
        }
    }
    
    /// 使用默认从站ID创建新的Modbus TCP客户端
    pub fn new_default(socket_addr: SocketAddr) -> ModbusTcpClient {
        Self::new(socket_addr, 1, ByteOrder::CDAB)
    }
    
    /// 使用自定义从站ID和字节顺序创建新的Modbus TCP客户端
    pub fn new_with_byte_order(socket_addr: SocketAddr, slave_id: u8, byte_order: ByteOrder) -> ModbusTcpClient {
        Self::new(socket_addr, slave_id, byte_order)
    }
}

impl PlcCommunicationService for ModbusTcpClient {
    async fn connect_async(&self) -> PlcResult<()> {
        // 尝试连接到Modbus TCP服务器
        match tcp::connect_slave(self.socket_addr, self.slave).await {
            Ok(ctx) => {
                // 连接成功，更新状态和上下文
                {
                    let mut state = self.connection_state.lock().await;
                    *state = true;
                    
                    let mut context = self.context.lock().await;
                    *context = Some(ctx);
                }
                
                // 启动连接检查任务
                self.start_connection_check().await;
                
                PlcResult::ok(())
            },
            Err(e) => {
                PlcResult::err(format!("连接失败: {}", e))
            }
        }
    }

    async fn dic_connect_async(&self) -> PlcResult<()> {
        // 停止连接检查任务
        self.stop_connection_check().await;
        
        // 关闭连接并更新状态
        {
            let mut context = self.context.lock().await;
            if let Some(mut ctx) = context.take() {
                // 断开连接
                let _ = ctx.disconnect().await;
            }
            
            let mut state = self.connection_state.lock().await;
            *state = false;
        }
        
        PlcResult::ok(())
    }

    async fn reconnect(&self) -> PlcResult<()> {
        // 先断开当前连接
        {
            let mut context = self.context.lock().await;
            if let Some(mut ctx) = context.take() {
                // 断开连接
                let _ = ctx.disconnect().await;
            }
        }
        
        // 尝试重新连接
        loop {
            match tcp::connect_slave(self.socket_addr, self.slave).await {
                Ok(ctx) => {
                    // 连接成功，更新状态和上下文
                    {
                        let mut state = self.connection_state.lock().await;
                        *state = true;
                        
                        let mut context = self.context.lock().await;
                        *context = Some(ctx);
                    }
                    
                    return PlcResult::ok(());
                },
                Err(_) => {
                    // 连接失败，等待2秒后重试
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    async fn check_connection(&self, _check_address: &str) -> PlcResult<()> {
        // 获取上下文的异步锁
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            // 尝试读取一个寄存器来验证连接是否正常
            match ctx.read_holding_registers(0, 1).await {
                Ok(_) => PlcResult::ok(()),
                Err(e) => PlcResult::err(format!("连接检查失败: {}", e))
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn get_connection_state(&self) -> PlcResult<bool> {
        let state = self.connection_state.lock().await;
        PlcResult::ok(*state)
    }

    async fn read_signal_analog_value_async(&self, address: &str) -> PlcResult<f32> {
        // 解析地址字符串，格式如"40001"表示第一个保持寄存器
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let register_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr-1, 
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 获取上下文并执行读取操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "4" => { // 保持寄存器 (Holding Register)
                    match ctx.read_holding_registers(register_address, 2).await {
                        Ok(register_result) => {
                            println!("地址: {:?}", register_address);
                            // 处理内部的Result
                            match register_result {
                                Ok(values) => {
                                    // 检查向量是否包含两个寄存器值
                                    if values.len() < 2 {
                                        PlcResult::err("读取保持寄存器返回值不足".to_string())
                                    } else {
                                        // 将两个寄存器组合为单精度浮点数
                                        println!("值: {:?}", values);
                                        
                                        // 使用ByteOrderConverter转换寄存器值为浮点数
                                        let float_value = ByteOrderConverter::registers_to_float(
                                            values[0], 
                                            values[1], 
                                            self.byte_order
                                        );
                                        
                                        println!("转换后的浮点值: {}", float_value);
                                        PlcResult::ok(float_value)
                                    }
                                },
                                Err(exception) => PlcResult::err(format!("读取保持寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取保持寄存器失败: {}", e))
                    }
                },
                "3" => { // 输入寄存器 (Input Register)
                    match ctx.read_input_registers(register_address, 2).await {
                        Ok(register_result) => {
                            // 处理内部的Result
                            match register_result {
                                Ok(values) => {
                                    // 检查向量是否包含两个寄存器值
                                    if values.len() < 2 {
                                        PlcResult::err("读取输入寄存器返回值不足".to_string())
                                    } else {
                                        // 使用ByteOrderConverter转换寄存器值为浮点数
                                        let float_value = ByteOrderConverter::registers_to_float(
                                            values[0], 
                                            values[1], 
                                            self.byte_order
                                        );
                                        PlcResult::ok(float_value)
                                    }
                                },
                                Err(exception) => PlcResult::err(format!("读取输入寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取输入寄存器失败: {}", e))
                    }
                },
                _ => PlcResult::err(format!("不支持的寄存器类型: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn write_signal_analog_value_async(&self, address: &str, value: f32) -> PlcResult<()> {
        // 解析地址字符串，格式如"40001"表示第一个保持寄存器
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let register_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 将f32值转换为两个u16寄存器值，使用当前字节顺序
        let (reg1, reg2) = ByteOrderConverter::float_to_registers(value, self.byte_order);
        
        // 存储两个寄存器的值
        let registers = vec![reg1, reg2];
        
        // 获取上下文并执行写入操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "4" => { // 保持寄存器 (Holding Register)
                    match ctx.write_multiple_registers(register_address, &registers).await {
                        Ok(write_result) => {
                            match write_result {
                                Ok(_) => PlcResult::ok(()),
                                Err(exception) => PlcResult::err(format!("写入保持寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("写入保持寄存器失败: {}", e)),
                    }
                },
                _ => PlcResult::err(format!("不支持的寄存器类型或不可写: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn read_signal_analog_values_async(&self, address: &str, count: i32) -> PlcResult<Vec<f32>> {
        // 解析地址字符串，格式如"40001"表示第一个保持寄存器
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let register_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 由于每个浮点数需要2个寄存器，所以实际需要读取的寄存器数量是浮点数数量的两倍
        let register_count = (count * 2) as u16;
        
        // 获取上下文并执行读取操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            let result: Result<Vec<f32>, String> = match address_type {
                "4" => { // 保持寄存器 (Holding Register)
                    match ctx.read_holding_registers(register_address, register_count).await {
                        Ok(register_result) => {
                            match register_result {
                                Ok(registers) => {
                                    // 将寄存器值两两配对转换为浮点数
                                    let mut float_values = Vec::new();
                                    for chunk in registers.chunks(2) {
                                        if chunk.len() == 2 {
                                            // 使用ByteOrderConverter转换寄存器值为浮点数
                                            let float_value = ByteOrderConverter::registers_to_float(
                                                chunk[0], 
                                                chunk[1], 
                                                self.byte_order
                                            );
                                            float_values.push(float_value);
                                        }
                                    }
                                    Ok(float_values)
                                },
                                Err(exception) => Err(format!("读取保持寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => Err(format!("读取保持寄存器失败: {}", e))
                    }
                },
                "3" => { // 输入寄存器 (Input Register)
                    match ctx.read_input_registers(register_address, register_count).await {
                        Ok(register_result) => {
                            match register_result {
                                Ok(registers) => {
                                    // 将寄存器值两两配对转换为浮点数
                                    let mut float_values = Vec::new();
                                    for chunk in registers.chunks(2) {
                                        if chunk.len() == 2 {
                                            // 使用ByteOrderConverter转换寄存器值为浮点数
                                            let float_value = ByteOrderConverter::registers_to_float(
                                                chunk[0], 
                                                chunk[1], 
                                                self.byte_order
                                            );
                                            float_values.push(float_value);
                                        }
                                    }
                                    Ok(float_values)
                                },
                                Err(exception) => Err(format!("读取输入寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => Err(format!("读取输入寄存器失败: {}", e))
                    }
                },
                _ => Err(format!("不支持的寄存器类型: {}", address_type)),
            };
            
            match result {
                Ok(values) => PlcResult::ok(values),
                Err(msg) => PlcResult::err(msg),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn write_signal_analog_values_async(&self, address: &str, values: Vec<f32>) -> PlcResult<()> {
        // 解析地址字符串，格式如"40001"表示第一个保持寄存器
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let register_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 将每个f32值转换为两个u16寄存器值
        let mut values_u16 = Vec::with_capacity(values.len() * 2);
        for value in values {
            let (reg1, reg2) = ByteOrderConverter::float_to_registers(value, self.byte_order);
            values_u16.push(reg1);
            values_u16.push(reg2);
        }
        
        // 获取上下文并执行写入操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "4" => { // 保持寄存器 (Holding Register)
                    match ctx.write_multiple_registers(register_address, &values_u16).await {
                        Ok(write_result) => {
                            match write_result {
                                Ok(_) => PlcResult::ok(()),
                                Err(exception) => PlcResult::err(format!("写入保持寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("写入保持寄存器失败: {}", e)),
                    }
                },
                _ => PlcResult::err(format!("不支持的寄存器类型或不可写: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn read_signal_bool_async(&self, address: &str) -> PlcResult<bool> {
        // 解析地址字符串，格式如"00001"表示第一个线圈
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let bit_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 获取上下文并执行读取操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "0" => { // 线圈 (Coil)
                    match ctx.read_coils(bit_address, 1).await {
                        Ok(coil_result) => {
                            match coil_result {
                                Ok(values) => {
                                    // 检查向量是否为空
                                    if values.is_empty() {
                                        PlcResult::err("读取线圈返回了空值".to_string())
                                    } else {
                                        PlcResult::ok(values[0])
                                    }
                                },
                                Err(exception) => PlcResult::err(format!("读取线圈出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取线圈失败: {}", e))
                    }
                },
                "1" => { // 离散输入 (Discrete Input)
                    match ctx.read_discrete_inputs(bit_address, 1).await {
                        Ok(input_result) => {
                            match input_result {
                                Ok(values) => {
                                    // 检查向量是否为空
                                    if values.is_empty() {
                                        PlcResult::err("读取离散输入返回了空值".to_string())
                                    } else {
                                        PlcResult::ok(values[0])
                                    }
                                },
                                Err(exception) => PlcResult::err(format!("读取离散输入出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取离散输入失败: {}", e))
                    }
                },
                _ => PlcResult::err(format!("不支持的寄存器类型: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn write_signal_bool_async(&self, address: &str, value: bool) -> PlcResult<()> {
        // 解析地址字符串，格式如"00001"表示第一个线圈
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let bit_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 获取上下文并执行写入操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "0" => { // 线圈 (Coil)
                    match ctx.write_single_coil(bit_address, value).await {
                        Ok(write_result) => {
                            match write_result {
                                Ok(_) => PlcResult::ok(()),
                                Err(exception) => PlcResult::err(format!("写入线圈出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("写入线圈失败: {}", e))
                    }
                },
                _ => PlcResult::err(format!("不支持的寄存器类型或该寄存器类型不可写入: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn write_signal_boolean_value_async(&self, address: &str, value: bool) -> PlcResult<()> {
        // 解析地址字符串，格式如"00001"表示第一个线圈
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let coil_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 获取上下文并执行写入操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "0" => { // 线圈 (Coil)
                    match ctx.write_single_coil(coil_address, value).await {
                        Ok(write_result) => {
                            match write_result {
                                Ok(_) => PlcResult::ok(()),
                                Err(exception) => PlcResult::err(format!("写入线圈出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("写入线圈失败: {}", e)),
                    }
                },
                _ => PlcResult::err(format!("不支持的线圈类型或不可写: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn read_signal_boolean_values_async(&self, address: &str, count: i32) -> PlcResult<Vec<bool>> {
        // 解析地址字符串，格式如"00001"表示第一个线圈
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let coil_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 获取上下文并执行读取操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "0" => { // 线圈 (Coil)
                    match ctx.read_coils(coil_address, count as u16).await {
                        Ok(coil_result) => {
                            match coil_result {
                                Ok(coil_values) => PlcResult::ok(coil_values),
                                Err(exception) => PlcResult::err(format!("读取线圈失败，出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取线圈失败: {}", e)),
                    }
                },
                "1" => { // 离散输入 (Discrete Input)
                    match ctx.read_discrete_inputs(coil_address, count as u16).await {
                        Ok(discrete_result) => {
                            match discrete_result {
                                Ok(discrete_values) => PlcResult::ok(discrete_values),
                                Err(exception) => PlcResult::err(format!("读取离散输入失败，出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取离散输入失败: {}", e)),
                    }
                },
                _ => PlcResult::err(format!("不支持的线圈/离散输入类型: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn write_signal_boolean_values_async(&self, address: &str, values: Vec<bool>) -> PlcResult<()> {
        // 解析地址字符串，格式如"00001"表示第一个线圈
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let coil_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 获取上下文并执行写入操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "0" => { // 线圈 (Coil)
                    match ctx.write_multiple_coils(coil_address, &values).await {
                        Ok(write_result) => {
                            match write_result {
                                Ok(_) => PlcResult::ok(()),
                                Err(exception) => PlcResult::err(format!("写入线圈失败，出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("写入线圈失败: {}", e)),
                    }
                },
                _ => PlcResult::err(format!("不支持的线圈类型或不可写: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn read_signal_int_async(&self, address: &str) -> PlcResult<i32> {
        // 解析地址字符串，格式如"40001"表示第一个保持寄存器
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let register_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 获取上下文并执行读取操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "3" => { // 输入寄存器 (Input Register)
                    match ctx.read_input_registers(register_address, 1).await {
                        Ok(register_result) => {
                            match register_result {
                                Ok(values) => {
                                    // 检查向量是否为空
                                    if values.is_empty() {
                                        PlcResult::err("读取输入寄存器时获取到空数据".to_string())
                                    } else {
                                        // 使用安全的类型转换
                                        PlcResult::ok(i32::from(values[0]))
                                    }
                                },
                                Err(exception) => PlcResult::err(format!("读取输入寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取输入寄存器失败: {}", e))
                    }
                },
                "4" => { // 保持寄存器 (Holding Register)
                    match ctx.read_holding_registers(register_address, 1).await {
                        Ok(register_result) => {
                            match register_result {
                                Ok(values) => {
                                    // 检查向量是否为空
                                    if values.is_empty() {
                                        PlcResult::err("读取保持寄存器时获取到空数据".to_string())
                                    } else {
                                        // 使用安全的类型转换
                                        PlcResult::ok(i32::from(values[0]))
                                    }
                                },
                                Err(exception) => PlcResult::err(format!("读取保持寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("读取保持寄存器失败: {}", e))
                    }
                },
                _ => PlcResult::err(format!("不支持的寄存器类型: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn write_signal_int_async(&self, address: &str, value: i32) -> PlcResult<()> {
        // 解析地址字符串，格式如"40001"表示第一个保持寄存器
        if address.len() < 2 {
            return PlcResult::err(format!("无效的地址格式: {}", address));
        }
        
        let address_type = &address[0..1];
        let register_address = match address[1..].parse::<u16>() {
            Ok(addr) => addr - 1, // Modbus地址从0开始，但用户习惯从1开始计数
            Err(_) => return PlcResult::err(format!("无效的地址格式: {}", address)),
        };
        
        // 将i32值安全地转换为u16
        let register_value = if value < 0 {
            0
        } else if value > u16::MAX as i32 {
            u16::MAX
        } else {
            value as u16
        };
        
        // 获取上下文并执行写入操作
        let mut context_guard = self.context.lock().await;
        if let Some(ctx) = &mut *context_guard {
            match address_type {
                "4" => { // 只支持写入保持寄存器 (Holding Register)
                    match ctx.write_single_register(register_address, register_value).await {
                        Ok(write_result) => {
                            match write_result {
                                Ok(_) => PlcResult::ok(()),
                                Err(exception) => PlcResult::err(format!("写入保持寄存器出现异常: {:?}", exception))
                            }
                        },
                        Err(e) => PlcResult::err(format!("写入保持寄存器失败: {}", e))
                    }
                },
                _ => PlcResult::err(format!("不支持写入的寄存器类型: {}", address_type)),
            }
        } else {
            PlcResult::err("未连接到设备".to_string())
        }
    }

    async fn read_signal_int_batch_async(&self, addresses: Vec<&str>) -> PlcResult<Vec<i32>> {
        let mut results = Vec::with_capacity(addresses.len());
        
        // 依次读取每个地址的值
        for address in addresses {
            match self.read_signal_int_async(address).await {
                result if result.is_ok() => {
                    if let Some(value) = result.data() {
                        results.push(*value);
                    }
                },
                result => return PlcResult::err(format!(
                    "批量读取整数值失败，地址 {}: {}", 
                    address, 
                    result.error_message.unwrap_or_else(|| "未知错误".to_string())
                )),
            }
        }
        
        PlcResult::ok(results)
    }
    
    async fn write_signal_int_batch_async(&self, addresses: Vec<&str>, values: Vec<i32>) -> PlcResult<()> {
        // 检查地址和值的数量是否匹配
        if addresses.len() != values.len() {
            return PlcResult::err(format!(
                "地址和值的数量不匹配: 地址数量 = {}, 值数量 = {}", 
                addresses.len(), values.len()
            ));
        }
        
        // 依次写入每个地址的值
        for (address, value) in addresses.iter().zip(values.iter()) {
            let result = self.write_signal_int_async(address, *value).await;
            if result.is_err() {
                return PlcResult::err(format!(
                    "批量写入整数值失败，地址 {}: {}", 
                    address, 
                    result.error_message.unwrap_or_else(|| "未知错误".to_string())
                ));
            }
        }
        
        PlcResult::ok(())
    }

    async fn read_signal_boolean_value_async(&self, address: &str) -> PlcResult<bool> {
        // 实现与read_signal_bool_async相同的功能
        self.read_signal_bool_async(address).await
    }

    fn get_byte_order(&self) -> ByteOrder {
        self.byte_order
    }
}

impl ModbusTcpClient {
    // 启动连接检查任务
    async fn start_connection_check(&self) {
        // 停止已有的检查任务
        self.stop_connection_check().await;
        
        // 创建取消通道
        let (tx, mut rx) = oneshot::channel();
        {
            let mut cancel = self.cancel_channel.lock().await;
            *cancel = Some(tx);
        }
        
        // 创建新的检查任务
        let connection_state = Arc::clone(&self.connection_state);
        let context = Arc::clone(&self.context);
        let socket_addr = self.socket_addr;
        let slave = self.slave;
        
        let handle = tokio::spawn(async move {
            loop {
                // 检查是否收到取消信号
                if rx.try_recv().is_ok() {
                    break;
                }
                
                // 检查连接状态
                let need_reconnect = {
                    let mut ctx_guard = context.lock().await;
                    if let Some(ctx) = &mut *ctx_guard {
                        // 直接使用上下文进行读取操作
                        let result = ctx.read_holding_registers(0, 1).await;
                        match result {
                            Ok(_) => false, // 连接正常
                            Err(_) => true   // 连接异常，需要重连
                        }
                    } else {
                        true // 没有上下文，需要连接
                    }
                };
                
                if need_reconnect {
                    // 更新连接状态
                    {
                        let mut state = connection_state.lock().await;
                        *state = false;
                    }
                    
                    // 尝试重新连接
                    loop {
                        // 检查是否收到取消信号
                        if rx.try_recv().is_ok() {
                            return;
                        }
                        
                        match tcp::connect_slave(socket_addr, slave).await {
                            Ok(new_ctx) => {
                                // 连接成功，更新状态和上下文
                                {
                                    let mut state = connection_state.lock().await;
                                    *state = true;
                                    
                                    let mut ctx_guard = context.lock().await;
                                    *ctx_guard = Some(new_ctx);
                                }
                                break;
                            },
                            Err(_) => {
                                // 连接失败，等待2秒后重试
                                sleep(Duration::from_secs(2)).await;
                            }
                        }
                    }
                }
                
                // 等待一段时间再次检查
                sleep(Duration::from_secs(5)).await;
            }
        });
        
        // 保存任务句柄
        let mut task = self.check_task.lock().await;
        *task = Some(handle);
    }
    
    // 停止连接检查任务
    async fn stop_connection_check(&self) {
        // 发送取消信号
        {
            let mut cancel = self.cancel_channel.lock().await;
            if let Some(tx) = cancel.take() {
                let _ = tx.send(());
            }
        }
        
        // 等待任务结束并清理
        {
            let mut task = self.check_task.lock().await;
            if let Some(handle) = task.take() {
                let _ = handle.await;
            }
        }
    }
}