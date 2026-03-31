# ptd-cli

[English](README.en.md)

[PT-Depiler](https://github.com/pt-plugins/PT-depiler) 浏览器扩展的命令行工具，通过 Chrome Native Messaging 与扩展通信。

在终端中搜索种子、管理下载、查询用户信息、管理辅种任务——所有操作通过运行中的浏览器扩展执行，复用其 Cookie、站点配置和下载器设置。

## 架构

```
ptd search "avatar" --site chdbits
  |
  v
Unix socket --> ptd-host 守护进程 --stdout--> Chrome --> 扩展通信桥
                                                          |
                                                    sendMessage("getSiteSearchResult", {...})
                                                          |
                                                    offscreen 处理器 (HTTP + DOM 解析)
                                                          |
CLI <-- Unix socket <-- ptd-host <--stdin-- Chrome <------+
```

三个组件：

- **`ptd`** — CLI 客户端。发现运行中的浏览器实例，通过 Unix socket 连接，发送命令，输出结果。
- **`ptd-host`** — Native messaging 守护进程。Chrome 为每个浏览器配置文件启动一个实例，负责桥接 CLI 请求到扩展并路由响应。
- **扩展通信桥** — PT-Depiler 后台脚本中的模块，通过已有的 `sendMessage()` 系统分发 CLI 请求。

## 安装

### 1. 下载预编译二进制

从 [GitHub Releases](https://github.com/pt-plugins/ptd-cli/releases) 下载最新版本的 `ptd` 和 `ptd-host`，解压后将两个文件放在同一目录下，并添加到 `PATH`。

> **AI Agent 用户：** 请直接从 Release 页面下载预编译二进制，无需从源码构建。

<details>
<summary>从源码构建</summary>

```bash
cargo build --release
# 生成 target/release/ptd 和 target/release/ptd-host
```

将 `ptd` 和 `ptd-host` 放在同一目录下，并添加到 `PATH`。

</details>

### 2. 注册 Native Messaging Host

> **重要：** 请在安装或启用 PT-Depiler 扩展**之前**完成此步骤。
> Chrome 仅在启动时读取 native messaging host 注册信息。
> 如果在 Chrome 运行时注册，必须**完全退出 Chrome**（包括后台进程）后重新启动。
> Windows 上请使用 `taskkill /f /im chrome.exe` 或检查系统托盘——如果开启了"关闭 Google Chrome 后继续运行后台应用"，仅关闭窗口是不够的。

```bash
# Chrome
ptd install --browser chrome --extension-id <扩展ID>

# Firefox
ptd install --browser firefox

# Chromium / Edge
ptd install --browser chromium --extension-id <扩展ID>
ptd install --browser edge --extension-id <扩展ID>
```

扩展 ID 可在 `chrome://extensions` 中开启开发者模式后查看。

### 3. 在扩展中启用

打开 PT-Depiler 扩展设置页，进入 **设置 > 常规设置 > 原生通信桥** 标签页：

1. 点击 **授予权限** 启用 `nativeMessaging` 权限
2. 开启 **启用原生通信桥** 开关
3. 点击 **测试连接** 验证连接

### 4. 验证

```bash
ptd status
# 应显示一个健康的实例
```

如果 `ptd status` 未显示实例，请确认：
1. 浏览器正在运行且已启用 PT-Depiler 扩展
2. 运行 `ptd install` 后已完全重启浏览器
3. 扩展已授予 `nativeMessaging` 权限并启用了原生通信桥

## 使用

### 发现

```bash
ptd status                                    # 查看运行中的浏览器实例
ptd site list                                 # 列出所有已配置站点（ID、名称、URL）
ptd site list --table                         # 表格格式
ptd downloader list                           # 列出所有下载器（ID、名称、类型、地址）
ptd downloader list --table                   # 表格格式
```

> **提示：** 在执行需要站点 ID 或下载器 ID 的命令前，先运行 `ptd site list` 和 `ptd downloader list` 获取有效的 ID。

### 搜索

```bash
# 搜索所有已配置站点
ptd search "avatar"

# 搜索指定站点
ptd search "avatar" --site chdbits
ptd search "avatar" --site chdbits --site btschool

# 格式化输出
ptd search "avatar" --site chdbits --pretty

# 使用搜索配置文件进行高级搜索
ptd search "avatar" --site chdbits --entry-file ./search-config.json
```

### 下载

```bash
# 按上次搜索结果的索引下载
ptd download 0 --downloader <下载器ID>

# 使用完整选项文件下载
ptd download --option-file ./download-option.json
```

### 下载器

```bash
ptd downloader status <下载器ID>
ptd downloader config <下载器ID>
ptd downloader version <下载器ID>
```

### 用户信息

```bash
ptd user-info current <站点ID>          # 获取实时用户数据
ptd user-info history <站点ID>          # 查看历史数据
ptd user-info remove <站点ID> <日期>    # 删除记录
ptd user-info cancel                    # 取消待处理的请求
```

### 站点配置

```bash
ptd site config <站点ID>
ptd site favicon <站点ID> [--flush]
```

### 下载历史

```bash
ptd download-history                     # 列出所有记录
ptd download-history get <id>            # 查看指定记录
ptd download-history delete <id>         # 删除记录
ptd download-history clear               # 清空所有记录
```

### 辅种任务

```bash
ptd keep-upload list
ptd keep-upload get <任务ID>
ptd keep-upload create --file ./task.json
ptd keep-upload update --file ./task.json
ptd keep-upload delete <任务ID>
ptd keep-upload clear
```

### 安装与发现

```bash
ptd install --browser chrome --extension-id <扩展ID>
ptd uninstall --browser chrome
ptd status
```

## 全局选项

```
--instance <id>       选择浏览器/配置实例（前缀匹配，如 --instance fe4c）
--timeout <秒数>      请求超时时间（默认：30）
--format <格式>       输出格式：json（默认）、pretty、table
--pretty              --format pretty 的简写
--table               --format table 的简写
```

环境变量：`PTD_INSTANCE=<id>`

## 多实例支持

如果你运行多个浏览器或配置文件，每个都会有自己的守护进程和 socket。只有一个运行时 CLI 会自动选择：

```bash
$ ptd status
  fe4cb61e [healthy] browser=chrome ext=icblbk... since=2026-03-30T07:20:51Z
  a3d91f02 [healthy] browser=firefox ext=ptdep... since=2026-03-30T08:15:22Z

2 instance(s), 2 healthy

$ ptd --instance fe4c search "test" --site chdbits   # 前缀匹配
```

## 退出码

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 命令失败（扩展错误、超时、输入错误） |
| 2 | 未找到健康的实例 |
| 3 | 存在多个实例，未指定选择 |

## 输出

默认输出紧凑 JSON，适合通过管道传递给 `jq`：

```bash
ptd search "test" --site chdbits | jq '.[0].title'
ptd user-info current chdbits | jq '.ratio'
```

## 许可证

MIT
