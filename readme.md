### McPatch2

McPatch第二版，包含客户端和管理端的源代码（也包括web页面源代码）。

### crates说明

| 名称                   | 用途                                                         |
| ---------------------- | ------------------------------------------------------------ |
| client         | 客户端主程序。用来执行文件更新过程，需要配置好后分发给玩家   |
| manager        | 管理端主程序。负责更新包的打包和管理工作，也提供内置开箱即用的内置服务端 |
| config_template_derive | 给客户端用的过程宏，用来根据源代码注释自动化生成客户端配置文件模板 |
| shared         | 客户端和管理端共用的代码部分                                 |
| xtask                  | 用于ci/cd自动化打包的行为和命令                              |

### 常用命令说明

| 命令                                | 作用                                 |
| ----------------------------------- | ------------------------------------ |
| `cargo cc`                          | 开发场景下，启动客户端程序           |
| `cargo mm`                          | 开发场景下，启动管理端程序           |
| `cargo check --all`                 | 开发场景下，执行构建检查             |
| `cargo build --package client`      | 开发场景下，打包客户端程序           |
| `cargo build --package manager`     | 开发场景下，打包管理端程序           |
| `cargo doc --no-deps --open`        | 开发场景下，生成文档页面             |
| `cargo ci <client/manager>`         | 自动构建场景下，打包客户端或者管理端 |
