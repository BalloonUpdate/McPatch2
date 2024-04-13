//! 命令行主逻辑
//! 
//! 目前支持的功能：
//! 
//! + [x] 打包新版本
//! + [x] 合并新版本
//! + [x] 解压并测试现有的文件
//! + [x] 合并版本
//! + [x] 检查工作空间目录修改情况
//! + [ ] 启动内置服务端

pub mod check;
pub mod combine;
pub mod pack;
pub mod test;
