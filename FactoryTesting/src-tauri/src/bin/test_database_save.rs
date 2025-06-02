// 测试数据库保存功能
use sea_orm::Database;
use app_lib::models::structs::ChannelPointDefinition;
use app_lib::models::enums::{ModuleType, PointDataType};
use app_lib::services::infrastructure::persistence::{SqliteOrmPersistenceService, PersistenceConfig};
use app_lib::services::traits::PersistenceService;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试数据库保存功能");

    // 连接数据库
    let db_url = "sqlite://./factory_testing_data.sqlite?mode=rwc";
    let db = Database::connect(db_url).await?;
    println!("✅ 数据库连接成功");

    // 创建持久化服务
    let config = PersistenceConfig::default();
    let persistence_service = SqliteOrmPersistenceService::new(config, Some(Path::new("./factory_testing_data.sqlite"))).await?;
    println!("✅ 持久化服务创建成功");

    // 创建测试通道定义
    let test_definition = ChannelPointDefinition {
        id: uuid::Uuid::new_v4().to_string(),
        tag: "TEST_AI_001".to_string(),
        variable_name: "测试模拟量输入".to_string(),
        variable_description: "测试用的模拟量输入通道".to_string(),
        station_name: "测试站".to_string(),
        module_name: "AI模块".to_string(),
        module_type: ModuleType::AI,
        channel_tag_in_module: "AI_1".to_string(),
        data_type: PointDataType::Float,
        power_supply_type: "有源".to_string(),
        wire_system: "4线制".to_string(),
        plc_absolute_address: Some("%MD100".to_string()),
        plc_communication_address: "40001".to_string(),
        range_lower_limit: Some(0.0),
        range_upper_limit: Some(100.0),
        engineering_unit: Some("℃".to_string()),
        sll_set_value: None,
        sll_set_point_address: None,
        sll_set_point_plc_address: None,
        sll_set_point_communication_address: None,
        sll_feedback_address: None,
        sll_feedback_plc_address: None,
        sll_feedback_communication_address: None,
        sl_set_value: None,
        sl_set_point_address: None,
        sl_set_point_plc_address: None,
        sl_set_point_communication_address: None,
        sl_feedback_address: None,
        sl_feedback_plc_address: None,
        sl_feedback_communication_address: None,
        sh_set_value: None,
        sh_set_point_address: None,
        sh_set_point_plc_address: None,
        sh_set_point_communication_address: None,
        sh_feedback_address: None,
        sh_feedback_plc_address: None,
        sh_feedback_communication_address: None,
        shh_set_value: None,
        shh_set_point_address: None,
        shh_set_point_plc_address: None,
        shh_set_point_communication_address: None,
        shh_feedback_address: None,
        shh_feedback_plc_address: None,
        shh_feedback_communication_address: None,
        maintenance_value_set_point_address: None,
        maintenance_value_set_point_plc_address: None,
        maintenance_value_set_point_communication_address: None,
        maintenance_enable_switch_point_address: None,
        maintenance_enable_switch_point_plc_address: None,
        maintenance_enable_switch_point_communication_address: None,
        access_property: None,
        save_history: Some(true),
        power_failure_protection: Some(false),
        test_rig_plc_address: None,
    };

    println!("📝 创建测试通道定义: ID={}, Tag={}", test_definition.id, test_definition.tag);

    // 尝试保存到数据库
    println!("💾 开始保存到数据库...");
    match persistence_service.save_channel_definition(&test_definition).await {
        Ok(_) => {
            println!("✅ 保存成功！");
            
            // 立即验证保存结果
            println!("🔍 验证保存结果...");
            match persistence_service.load_channel_definition(&test_definition.id).await {
                Ok(Some(loaded_def)) => {
                    println!("✅ 验证成功！从数据库加载的定义:");
                    println!("   ID: {}", loaded_def.id);
                    println!("   Tag: {}", loaded_def.tag);
                    println!("   Variable Name: {}", loaded_def.variable_name);
                    println!("   Module Type: {:?}", loaded_def.module_type);
                    println!("   Channel Tag: {}", loaded_def.channel_tag_in_module);
                    println!("   Power Supply Type: {}", loaded_def.power_supply_type);
                }
                Ok(None) => {
                    println!("❌ 验证失败：保存后立即查询找不到记录");
                }
                Err(e) => {
                    println!("❌ 验证失败：查询出错 - {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ 保存失败: {}", e);
            return Err(e.into());
        }
    }

    // 查询所有通道定义
    println!("📊 查询所有通道定义...");
    match persistence_service.load_all_channel_definitions().await {
        Ok(definitions) => {
            println!("✅ 查询成功，共找到 {} 个通道定义", definitions.len());
            for (index, def) in definitions.iter().enumerate() {
                println!("   {}. ID={}, Tag={}, Type={:?}", 
                    index + 1, def.id, def.tag, def.module_type);
            }
        }
        Err(e) => {
            println!("❌ 查询失败: {}", e);
        }
    }

    println!("🎉 测试完成！");
    Ok(())
}
