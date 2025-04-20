# Li Domain Scanner

一个用于扫描.li域名可用性的命令行工具。

## 功能特点

- 支持扫描1-4字符的域名
- 支持纯字母域名扫描模式
- 可配置并发数和延迟时间
- 自动保存扫描结果和进度
- 支持重复模式域名扫描

## 安装

从 [Releases](../../releases) 页面下载适合你系统的最新版本。下载后解压并运行：

```bash
# Linux/macOS
chmod +x li-domain-checker
./li-domain-checker

# Windows
li-domain-checker.exe
```

## 使用方法

```bash
li-domain-checker [选项]

选项:
    -w, --workers <workers>      并发数 [默认: 50]
    -d, --delay <delay>         延迟时间(秒) [默认: 1.0]
    -o, --output <output>       输出目录 [默认: "li_domain_results"]
    -f, --full-scan            是否扫描4字符域名（全扫描模式）
    -l, --letters-only         是否扫描4字符纯字母域名
    -y, --skip-warning         跳过运行时间提示
    -h, --help                 显示帮助信息
    -V, --version              显示版本信息
```

## 扫描模式说明

### 1. 普通模式（默认）
- 扫描1-3字符域名
- 扫描重复模式域名（如aaaa、aaab等）
- 包括字母、数字和连字符组合

### 2. 全扫描模式（--full-scan）
- 扫描所有1-4字符的域名组合
- 包括字母、数字和连字符
- 4字符域名总数：约1,679,616个（36^4）
- 建议使用较大的延迟时间（如2-3秒）

### 3. 纯字母模式（--letters-only）
- 仅扫描纯字母域名
- 不包括数字和连字符
- 4字符域名总数：约456,976个（26^4）
- 可与全扫描模式组合使用

### 4. 重复模式域名
- 四个相同字符：aaaa, bbbb, cccc...
- 三个相同字符加一个不同字符：aaab, bbba...
- 两个相同字符加两个相同字符：aabb, abab...

## 长时间运行建议

由于扫描可能需要较长时间，建议使用tmux、screen或nohup等工具来防止程序意外中断：

```bash
# 使用tmux（推荐）
tmux new -s scan
li-domain-checker --full-scan
# 按Ctrl+b然后d分离会话
# 使用tmux attach -t scan重新连接

# 使用nohup
nohup li-domain-checker --full-scan --skip-warning > scan.log 2>&1 &
```

## 输出文件

程序会在指定的输出目录中创建以下文件：

- `available_domains.txt`: 可用域名列表
- `scan_results.json`: 完整扫描结果

## 注意事项

1. 建议使用合理的延迟时间，避免对WHOIS服务器造成过大压力
2. 全扫描模式会消耗较长时间，请确保网络稳定

## 构建

如果你想自己构建程序：

```bash
git clone https://github.com/yourusername/li-domain-scanner
cd li-domain-scanner
cargo build --release
```

## 许可证

MIT License 
