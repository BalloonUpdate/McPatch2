### McPatch2

McPatch第二版，包含客户端和管理端的源代码，目前还在早期开发阶段

crates说明：

+ config_template_derive：给客户端用的过程宏，用来根据源代码注释自动化生成客户端配置文件模板
+ mcpatch-client：客户端程序，用来执行文件更新过程，需要配置好后分发给玩家（二进制crate）
+ mcpatch-manager：管理端程序，负责更新包的打包和管理工作，也提供内置开箱即用的内置服务端（二进制crate）
+ mcpatch-shared：客户端和管理端共用的代码部分

常用命令说明：

+ 执行构建检查：`cargo check --all`
+ 构建客户端可执行文件：`cargo build --bin mcpatch-client`
+ 构建管理端可执行文件：`cargo build --bin mcpatch-manager`
+ 生成文档页面：`cargo doc --no-deps --open`

