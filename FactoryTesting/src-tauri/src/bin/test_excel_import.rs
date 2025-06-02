use app_lib::services::infrastructure::excel::excel_importer::ExcelImporter;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🔥 测试Excel导入功能 - 验证PLC地址和通讯地址字段映射");

    // 测试文件路径
    let test_file_path = PathBuf::from("../../测试文件/测试IO.xlsx");

    if !test_file_path.exists() {
        eprintln!("❌ 测试文件不存在: {:?}", test_file_path);
        return Ok(());
    }

    println!("📁 测试文件路径: {:?}", test_file_path);

    // 导入Excel文件
    println!("📊 开始导入Excel文件...");
    let definitions = match ExcelImporter::parse_excel_file(test_file_path.to_str().unwrap()).await {
        Ok(defs) => defs,
        Err(e) => {
            eprintln!("❌ 导入失败: {}", e);
            return Ok(());
        }
    };
    
    println!("✅ 成功导入 {} 个通道定义", definitions.len());

    // 连接数据库并保存数据
    use sea_orm::{Database, EntityTrait};
    use app_lib::models::entities::channel_point_definition::{Entity as ChannelPointDefinition, ActiveModel};

    let db_path = std::path::PathBuf::from("./factory_testing_data.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await?;

    println!("💾 保存数据到数据库...");

    // 清空现有数据
    ChannelPointDefinition::delete_many().exec(&db).await?;

    // 保存新数据
    for definition in &definitions {
        let active_model: ActiveModel = definition.into();
        ChannelPointDefinition::insert(active_model).exec(&db).await?;
    }

    println!("✅ 已保存 {} 个定义到数据库", definitions.len());

    // 验证前几个定义的字段映射
    println!("\n🔍 验证字段映射（前5个定义）:");
    for (i, def) in definitions.iter().take(5).enumerate() {
        println!("\n--- 定义 {} ---", i + 1);
        println!("位号: {}", def.tag);
        println!("变量名: {}", def.variable_name);
        println!("模块类型: {:?}", def.module_type);
        println!("PLC绝对地址: {:?}", def.plc_absolute_address);
        println!("上位机通讯地址: {}", def.plc_communication_address);
        
        // 检查报警设定点位的PLC地址和通讯地址
        if def.sll_set_point_plc_address.is_some() || def.sll_set_point_communication_address.is_some() {
            println!("SLL设定点位_PLC地址: {:?}", def.sll_set_point_plc_address);
            println!("SLL设定点位_通讯地址: {:?}", def.sll_set_point_communication_address);
        }
        
        if def.sl_set_point_plc_address.is_some() || def.sl_set_point_communication_address.is_some() {
            println!("SL设定点位_PLC地址: {:?}", def.sl_set_point_plc_address);
            println!("SL设定点位_通讯地址: {:?}", def.sl_set_point_communication_address);
        }
        
        if def.sh_set_point_plc_address.is_some() || def.sh_set_point_communication_address.is_some() {
            println!("SH设定点位_PLC地址: {:?}", def.sh_set_point_plc_address);
            println!("SH设定点位_通讯地址: {:?}", def.sh_set_point_communication_address);
        }
        
        if def.shh_set_point_plc_address.is_some() || def.shh_set_point_communication_address.is_some() {
            println!("SHH设定点位_PLC地址: {:?}", def.shh_set_point_plc_address);
            println!("SHH设定点位_通讯地址: {:?}", def.shh_set_point_communication_address);
        }
        
        // 检查反馈地址
        if def.sll_feedback_plc_address.is_some() || def.sll_feedback_communication_address.is_some() {
            println!("LL报警_PLC地址: {:?}", def.sll_feedback_plc_address);
            println!("LL报警_通讯地址: {:?}", def.sll_feedback_communication_address);
        }
        
        // 检查维护相关地址
        if def.maintenance_value_set_point_plc_address.is_some() || def.maintenance_value_set_point_communication_address.is_some() {
            println!("维护值设定点位_PLC地址: {:?}", def.maintenance_value_set_point_plc_address);
            println!("维护值设定点位_通讯地址: {:?}", def.maintenance_value_set_point_communication_address);
        }
        
        if def.maintenance_enable_switch_point_plc_address.is_some() || def.maintenance_enable_switch_point_communication_address.is_some() {
            println!("维护使能开关点位_PLC地址: {:?}", def.maintenance_enable_switch_point_plc_address);
            println!("维护使能开关点位_通讯地址: {:?}", def.maintenance_enable_switch_point_communication_address);
        }
    }
    
    // 统计有多少个定义包含PLC地址和通讯地址信息
    let mut plc_address_count = 0;
    let mut comm_address_count = 0;
    let mut alarm_plc_count = 0;
    let mut alarm_comm_count = 0;
    let mut maintenance_plc_count = 0;
    let mut maintenance_comm_count = 0;
    
    for def in &definitions {
        if def.plc_absolute_address.is_some() {
            plc_address_count += 1;
        }
        if !def.plc_communication_address.is_empty() {
            comm_address_count += 1;
        }
        
        // 统计报警相关地址
        if def.sll_set_point_plc_address.is_some() || def.sl_set_point_plc_address.is_some() ||
           def.sh_set_point_plc_address.is_some() || def.shh_set_point_plc_address.is_some() ||
           def.sll_feedback_plc_address.is_some() || def.sl_feedback_plc_address.is_some() ||
           def.sh_feedback_plc_address.is_some() || def.shh_feedback_plc_address.is_some() {
            alarm_plc_count += 1;
        }
        
        if def.sll_set_point_communication_address.is_some() || def.sl_set_point_communication_address.is_some() ||
           def.sh_set_point_communication_address.is_some() || def.shh_set_point_communication_address.is_some() ||
           def.sll_feedback_communication_address.is_some() || def.sl_feedback_communication_address.is_some() ||
           def.sh_feedback_communication_address.is_some() || def.shh_feedback_communication_address.is_some() {
            alarm_comm_count += 1;
        }
        
        // 统计维护相关地址
        if def.maintenance_value_set_point_plc_address.is_some() || def.maintenance_enable_switch_point_plc_address.is_some() {
            maintenance_plc_count += 1;
        }
        
        if def.maintenance_value_set_point_communication_address.is_some() || def.maintenance_enable_switch_point_communication_address.is_some() {
            maintenance_comm_count += 1;
        }
    }
    
    println!("\n📊 统计结果:");
    println!("总定义数: {}", definitions.len());
    println!("包含PLC绝对地址的定义: {}", plc_address_count);
    println!("包含上位机通讯地址的定义: {}", comm_address_count);
    println!("包含报警PLC地址的定义: {}", alarm_plc_count);
    println!("包含报警通讯地址的定义: {}", alarm_comm_count);
    println!("包含维护PLC地址的定义: {}", maintenance_plc_count);
    println!("包含维护通讯地址的定义: {}", maintenance_comm_count);
    
    // 验证是否修复了之前的问题
    if plc_address_count > 0 && comm_address_count > 0 {
        println!("\n✅ 字段映射修复成功！");
        println!("   - PLC绝对地址字段正常解析");
        println!("   - 上位机通讯地址字段正常解析");
    } else {
        println!("\n❌ 字段映射仍有问题！");
        if plc_address_count == 0 {
            println!("   - PLC绝对地址字段为空");
        }
        if comm_address_count == 0 {
            println!("   - 上位机通讯地址字段为空");
        }
    }
    
    if alarm_comm_count > 0 || maintenance_comm_count > 0 {
        println!("✅ 新增通讯地址字段解析成功！");
        println!("   - 报警通讯地址字段: {} 个定义", alarm_comm_count);
        println!("   - 维护通讯地址字段: {} 个定义", maintenance_comm_count);
    } else {
        println!("❌ 新增通讯地址字段解析失败！");
    }
    
    Ok(())
}
