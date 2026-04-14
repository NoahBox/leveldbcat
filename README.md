# LeveldbCat

LeveldbCat is a small LevelDB viewer with a GUI and a simple CLI mode.

It lets you browse folders, open a LevelDB directory, inspect keys and values, and export parsed entries to CSV.

## Features

- View keys and values as bytes, text, or JSON
- Search parsed entries
- Export parsed entries to CSV

## Build

```bash
cargo build --release
```

The release binary will be created at `target/release/LeveldbCat.exe`.

## Run

Start the GUI:

```bash
cargo run -- --gui
```

Start the GUI and open a specific folder:

```bash
cargo run -- --gui "C:\path\to\folder"
```

Run in CLI mode with explicit flag:

```bash
cargo run -- --cli "C:\path\to\leveldb"
```

Run in CLI mode with a positional path:

```bash
cargo run -- "C:\path\to\leveldb"
```

CLI mode loads the database and prints a short preview of the first few entries.

## Configuration

The config file is stored under your system config directory as:

```text
LeveldbCat/config.json
```
