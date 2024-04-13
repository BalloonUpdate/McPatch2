//! 目录差异对比
//!
//! 计算文件差异是将一个新目录和一个旧目录下面的文件内容进行对比，
//! 然后计算出新目录相较旧目录新增了什么文件，删除了什么文件等操作的过程
//! 
//! 文件差异会分成5类
//! 1. 删除的文件
//! 2. 删除的目录
//! 3. 覆盖的文件（新增和修改都视为覆盖）
//! 4. 创建的目录
//! 5. 移动的文件
//! 
//! 在扫描文件差异时，会遇到各种情况，然后分别记录成不同文件操作，具体的决策表如下
//! 
//! | 决策表           | 现在是目录                                               | 现在是文件                                       | 现在不存在了         |
//! | ---------------- | -------------------------------------------------------- | ------------------------------------------------ | -------------------- |
//! | 之前是目录       | 不做记录，而是进一步对比目录里面的内容                   | 旧目录记录为删除，并将新增文件的记录为覆盖       | 记录这个目录删除行为 |
//! | 之前是文件       | 旧文件记录为删除，并记录新增的目录下的全部文件内容为覆盖 | 先对比修改时间，再对比文件哈希，不同则记录为覆盖 | 记录这个文件删除行为 |
//! | 之前没有这个文件 | 记录新增的目录下的全部文件内容为覆盖                     | 记录这个新增的文件数据为覆盖                     | 什么也不做           |
//! 
//! 其中移动文件的操作无法直接检测出来，但是可以通过检查一下新增文件列表（覆盖文件列表）和删除文件列表。如果发现这两个列表中有哈希值相同的文件存在，那么就可以认为这是一个文件移动操作。此时把这个文件从这俩列表里拿出来，然后插到文件移动列表中

pub mod diff;
pub mod disk_file;
pub mod history_file;
pub mod abstract_file;
pub mod rule_filter;