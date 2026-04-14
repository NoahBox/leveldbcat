mod text_key;
mod translations;

use crate::config::AppLanguage;
use std::path::Path;
use translations::{chinese_text, english_text, japanese_text, traditional_chinese_text};

pub use text_key::TextKey;

#[derive(Clone, Copy)]
pub struct I18n {
    language: AppLanguage,
}

impl I18n {
    pub fn new(language: AppLanguage) -> Self {
        Self { language }
    }

    pub fn language(self) -> AppLanguage {
        self.language
    }

    pub fn text(self, key: TextKey) -> &'static str {
        match self.language {
            AppLanguage::English => english_text(key),
            AppLanguage::Chinese => chinese_text(key),
            AppLanguage::TraditionalChinese => traditional_chinese_text(key),
            AppLanguage::Japanese => japanese_text(key),
        }
    }

    pub fn language_name(self, language: AppLanguage) -> &'static str {
        match language {
            AppLanguage::English => "English",
            AppLanguage::Chinese => "简体中文",
            AppLanguage::TraditionalChinese => "繁體中文",
            AppLanguage::Japanese => "日本語",
        }
    }

    pub fn about_text(self) -> &'static str {
        match self.language {
            AppLanguage::English => {
                "LeveldbCat\nVersion 0.1.1\nPowered by NoahTie@XD Forensics\nThis project is part of the Forensics Cat tool suite.\nFor learning and communication only.\nLicense: MIT"
            }
            AppLanguage::Chinese => {
                "LeveldbCat\n版本 0.1.1\nPowered by NoahTie@XD Forensics\n该项目是取证猫工具套组的一部分.\n仅用于学习交流.\n许可条款: MIT"
            }
            AppLanguage::TraditionalChinese => {
                "LeveldbCat\n版本 0.1.1\nPowered by NoahTie@XD Forensics\n該專案是取證貓工具套組的一部分.\n僅用於學習交流.\n授權條款: MIT"
            }
            AppLanguage::Japanese => {
                "LeveldbCat\nバージョン 0.1.1\nPowered by NoahTie@XD Forensics\nこのプロジェクトは取证猫ツールスイートの一部です.\n学習と交流目的のみに使用してください.\nLicense: MIT"
            }
        }
    }

    pub fn loaded_entries(self, count: usize, path: &Path) -> String {
        match self.language {
            AppLanguage::English => format!("Loaded {count} entries from {}", path.display()),
            AppLanguage::Chinese => format!("已从 {} 加载 {count} 条记录", path.display()),
            AppLanguage::TraditionalChinese => {
                format!("已從 {} 載入 {count} 筆記錄", path.display())
            }
            AppLanguage::Japanese => {
                format!(
                    "{} から {count} 件のエントリを読み込みました",
                    path.display()
                )
            }
        }
    }

    pub fn refreshed(self, path: &Path) -> String {
        match self.language {
            AppLanguage::English => format!("Refreshed {}", path.display()),
            AppLanguage::Chinese => format!("已刷新 {}", path.display()),
            AppLanguage::TraditionalChinese => format!("已重新整理 {}", path.display()),
            AppLanguage::Japanese => format!("{} を更新しました", path.display()),
        }
    }

    pub fn directory_missing(self, path: &Path) -> String {
        match self.language {
            AppLanguage::English => format!("Directory does not exist: {}", path.display()),
            AppLanguage::Chinese => format!("目录不存在: {}", path.display()),
            AppLanguage::TraditionalChinese => format!("目錄不存在: {}", path.display()),
            AppLanguage::Japanese => format!("ディレクトリが存在しません: {}", path.display()),
        }
    }

    pub fn config_save_failed(self, error: &str) -> String {
        match self.language {
            AppLanguage::English => format!("Failed to save config: {error}"),
            AppLanguage::Chinese => format!("保存配置失败: {error}"),
            AppLanguage::TraditionalChinese => format!("儲存設定失敗: {error}"),
            AppLanguage::Japanese => format!("設定の保存に失敗しました: {error}"),
        }
    }

    pub fn no_entries_to_export(self) -> String {
        match self.language {
            AppLanguage::English => "No parsed entries available to export.".to_owned(),
            AppLanguage::Chinese => "当前没有可导出的解析结果。".to_owned(),
            AppLanguage::TraditionalChinese => "目前沒有可匯出的解析結果。".to_owned(),
            AppLanguage::Japanese => "エクスポートできる解析済みエントリがありません。".to_owned(),
        }
    }

    pub fn export_success(self, path: &Path) -> String {
        match self.language {
            AppLanguage::English => format!("Exported CSV to {}", path.display()),
            AppLanguage::Chinese => format!("CSV 已导出到 {}", path.display()),
            AppLanguage::TraditionalChinese => format!("CSV 已匯出到 {}", path.display()),
            AppLanguage::Japanese => format!("CSV を {} にエクスポートしました", path.display()),
        }
    }

    pub fn export_failed(self, error: &str) -> String {
        match self.language {
            AppLanguage::English => format!("CSV export failed: {error}"),
            AppLanguage::Chinese => format!("CSV 导出失败: {error}"),
            AppLanguage::TraditionalChinese => format!("CSV 匯出失敗: {error}"),
            AppLanguage::Japanese => format!("CSV のエクスポートに失敗しました: {error}"),
        }
    }

    pub fn parsed_entries_count(self, count: usize) -> String {
        match self.language {
            AppLanguage::English => format!("{count} entries"),
            AppLanguage::Chinese => format!("{count} 条记录"),
            AppLanguage::TraditionalChinese => format!("{count} 筆記錄"),
            AppLanguage::Japanese => format!("{count} 件のエントリ"),
        }
    }

    pub fn no_loaded_database(self) -> String {
        match self.language {
            AppLanguage::English => "No LevelDB folder parsed yet".to_owned(),
            AppLanguage::Chinese => "尚未解析任何 LevelDB 文件夹".to_owned(),
            AppLanguage::TraditionalChinese => "尚未解析任何 LevelDB 資料夾".to_owned(),
            AppLanguage::Japanese => "まだ LevelDB フォルダは解析されていません".to_owned(),
        }
    }

    pub fn font_size_value(self, font_size_px: f32) -> String {
        match self.language {
            AppLanguage::English => format!("{font_size_px:.0} px"),
            AppLanguage::Chinese => format!("{font_size_px:.0} 像素"),
            AppLanguage::TraditionalChinese => format!("{font_size_px:.0} 像素"),
            AppLanguage::Japanese => format!("{font_size_px:.0} px"),
        }
    }
}
