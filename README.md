# LI Domain Scanner

一个用Rust编写的`.li`域名扫描工具，用于快速检测`.li`域名的可用性。

## 功能特点

- 支持多线程并发扫描
- 可配置扫描延迟时间
- 自动生成域名组合
- 支持特殊域名模式扫描（如重复字符模式）
- 支持4字符全扫描模式
- 支持4字符纯字母扫描模式
- 实时显示扫描进度
- 结果自动保存为JSON和文本格式
- 彩色终端输出

## 安装要求

- Rust 1.70.0 或更高版本
- Cargo 包管理器

## 安装步骤

1. 克隆仓库：
```bash
git clone https://github.com/stx-x/li-domain-checker.git
cd li-domain-checker
```

2. 编译项目：
```bash
cargo build --release
```

## 使用方法

基本用法：
```bash
./target/release/li_domain_scanner
```

参数说明：
- `-w, --workers <NUM>`: 设置并发数（默认：50）
- `-d, --delay <SECONDS>`: 设置延迟时间（默认：1.0秒）
- `-o, --output <DIR>`: 设置输出目录（默认：li_domain_results）
- `-f, --full-scan`: 启用4字符全扫描模式
- `-l, --letters-only`: 启用4字符纯字母扫描模式

示例：
```bash
# 普通模式（扫描1-3字符域名和重复模式域名）
./target/release/li_domain_scanner

# 4字符全扫描模式
./target/release/li_domain_scanner --full-scan

# 4字符纯字母扫描模式
./target/release/li_domain_scanner --letters-only

# 组合使用
./target/release/li_domain_scanner -w 100 -d 0.5 -o my_results --full-scan --letters-only
```

## 扫描模式说明

1. 普通模式（默认）：
   - 扫描1-3字符域名
   - 扫描重复模式域名（如aaaa、aaab等）

2. 全扫描模式（--full-scan）：
   - 扫描所有1-4字符的域名组合
   - 包括字母、数字和连字符

3. 纯字母模式（--letters-only）：
   - 仅扫描纯字母域名
   - 不包括数字和连字符
   - 可与全扫描模式组合使用

## 输出结果

程序会在指定的输出目录中创建以下文件：
- `available_domains.txt`: 包含所有可用的域名列表
- `scan_results.json`: 包含完整的扫描结果，包括每个域名的状态和详细信息

## 注意事项

- 请合理设置并发数和延迟时间，避免对域名服务器造成过大压力
- 建议在扫描大量域名时使用较长的延迟时间
- 输出目录会自动创建，无需手动创建
- 4字符全扫描模式会生成大量域名，请确保有足够的存储空间
- 纯字母模式可以减少扫描数量，但可能会错过一些有价值的域名

## 许可证

MIT License 