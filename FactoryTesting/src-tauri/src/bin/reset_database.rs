#![cfg(FALSE)]
// 重置数据�?
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 重置数据�?===");
    
    let db_path = PathBuf::from("data/factory_testing_data.sqlite");
    
    // 删除旧的数据库文�?
    if db_path.exists() {
        println!("删除旧的数据库文�?..");
        fs::remove_file(&db_path)?;
        println!("�?旧数据库文件已删�?);
    } else {
        println!("数据库文件不存在，无需删除");
    }
    
    // 确保data目录存在
    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
        println!("�?创建data目录");
    }
    
    println!("🎉 数据库重置完成！下次运行时将自动创建新的数据库�?);
    
    Ok(())
}

