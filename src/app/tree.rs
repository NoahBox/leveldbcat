impl LevelDbBrowserApp {
    fn expand_to_directory(&mut self, path: &Path) {
        for ancestor in ancestor_chain(path) {
            ensure_root_for_path(&mut self.roots, &ancestor);
            self.expanded.insert(ancestor.clone());
            if let Err(error) = self.ensure_children_loaded(&ancestor) {
                self.show_error(error);
                break;
            }
        }
    }

    fn ensure_children_loaded(&mut self, path: &Path) -> Result<(), String> {
        if self.children_cache.contains_key(path) {
            return Ok(());
        }

        let mut items = Vec::new();
        let read_dir = fs::read_dir(path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;

        for entry in read_dir {
            let entry = entry
                .map_err(|error| format!("Failed to enumerate {}: {error}", path.display()))?;
            let entry_path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("Failed to inspect {}: {error}", entry_path.display()))?;
            let metadata = entry.metadata().ok();
            let is_link = metadata.as_ref().is_some_and(is_reparse_point);

            let kind = if file_type.is_dir() && !is_link {
                BrowserEntryKind::Directory
            } else if is_link {
                BrowserEntryKind::Link
            } else {
                BrowserEntryKind::File {
                    size_bytes: metadata.as_ref().map(fs::Metadata::len),
                }
            };

            items.push(BrowserEntry {
                path: entry_path.clone(),
                label: display_name(&entry_path),
                kind,
            });
        }

        items.sort_by(|left, right| {
            left.kind
                .sort_priority()
                .cmp(&right.kind.sort_priority())
                .then_with(|| left.label.to_lowercase().cmp(&right.label.to_lowercase()))
        });

        self.children_cache.insert(path.to_path_buf(), items);
        Ok(())
    }

    fn current_browser_entries(&self) -> Vec<BrowserEntry> {
        self.children_cache
            .get(&self.selected_dir)
            .cloned()
            .unwrap_or_default()
    }

    fn visible_tree_rows(&self) -> Vec<TreeRow> {
        let mut rows = Vec::new();

        for root in &self.roots {
            rows.push(TreeRow {
                path: root.clone(),
                label: display_name(root),
                depth: 0,
                expanded: self.expanded.contains(root),
                selected: self.selected_dir == *root,
            });

            if self.expanded.contains(root) {
                self.collect_tree_rows(root, 1, &mut rows);
            }
        }

        rows
    }

    fn tree_content_width(&self) -> Pixels {
        let rows = self.visible_tree_rows();
        let base = px(120.0);
        rows.into_iter().fold(base, |width, row| {
            let estimated = px(28.0
                + row.depth as f32 * 14.0
                + estimate_text_width(&row.label, self.config.font_size_px));
            width.max(estimated)
        })
    }

    fn collect_tree_rows(&self, path: &Path, depth: usize, rows: &mut Vec<TreeRow>) {
        let Some(children) = self.children_cache.get(path) else {
            return;
        };

        for child in children.iter().filter(|child| child.kind.is_directory()) {
            rows.push(TreeRow {
                path: child.path.clone(),
                label: child.label.clone(),
                depth,
                expanded: self.expanded.contains(&child.path),
                selected: self.selected_dir == child.path,
            });

            if self.expanded.contains(&child.path) {
                self.collect_tree_rows(&child.path, depth + 1, rows);
            }
        }
    }

    fn loaded_database_label(&self, i18n: I18n) -> String {
        self.loaded_db_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| i18n.no_loaded_database())
    }
}
