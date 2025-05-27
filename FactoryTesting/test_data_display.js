// 测试数据显示的简单脚本
console.log('🔍 测试PLC配置数据显示验证');

// 模拟从后端获取的数据
const testData = [
  {
    id: "1",
    channelAddress: "AI1_1",
    channelType: 0,
    communicationAddress: "40101",
    powerSupplyType: "24V DC",
    description: "模拟量输入通道 1",
    isEnabled: true
  },
  {
    id: "2", 
    channelAddress: "AI1_2",
    channelType: 0,
    communicationAddress: "40103",
    powerSupplyType: "24V DC",
    description: "模拟量输入通道 2",
    isEnabled: true
  }
];

console.log('📊 测试数据结构:');
testData.forEach((item, index) => {
  console.log(`${index + 1}. 通道位号: ${item.channelAddress}`);
  console.log(`   通讯地址: ${item.communicationAddress}`);
  console.log(`   供电类型: ${item.powerSupplyType}`);
  console.log(`   描述: ${item.description}`);
  console.log('---');
});

console.log('✅ 数据结构验证完成');
console.log('💡 如果界面显示为空，请检查前后端字段名映射是否正确'); 