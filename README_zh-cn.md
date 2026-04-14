# LeveldbCat

LeveldbCat 是一个轻量的 LevelDB 查看工具，提供图形界面和简单的命令行模式。

它可以浏览文件夹、打开 LevelDB 目录、查看键值内容，并将解析结果导出为 CSV。

## 功能

- 以字节、文本或 JSON 方式查看键和值
- 搜索已解析的记录
- 将解析结果导出为 CSV

## 构建

```bash
cargo build --release
```

生成的发布版可执行文件位于 `target/release/LeveldbCat.exe`。

## 运行

启动图形界面：

```bash
cargo run -- --gui
```

启动图形界面并打开指定目录：

```bash
cargo run -- --gui "C:\path\to\folder"
```

使用显式参数启动 CLI 模式：

```bash
cargo run -- --cli "C:\path\to\leveldb"
```

使用位置参数启动 CLI 模式：

```bash
cargo run -- "C:\path\to\leveldb"
```

CLI 模式会加载数据库，并输出前几条记录的预览。

## 配置

配置文件会保存在系统配置目录下的：

```text
LeveldbCat/config.json
```
