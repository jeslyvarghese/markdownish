use std::path::{Path, PathBuf};

use crate::file_loader::FileLoader;

/// A node in the file tree
#[derive(Debug, Clone)]
pub enum TreeNode {
    File(PathBuf),
    Dir {
        path: PathBuf,
        children: Vec<TreeNode>,
        expanded: bool,
    },
}

impl TreeNode {
    pub fn path(&self) -> &Path {
        match self {
            TreeNode::File(p) => p,
            TreeNode::Dir { path, .. } => path,
        }
    }

    pub fn name(&self) -> &str {
        self.path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, TreeNode::Dir { .. })
    }
}

/// Build a tree for a directory, sorted: dirs first then files, both alphabetical.
pub fn build_tree(path: &Path) -> TreeNode {
    if path.is_file() {
        return TreeNode::File(path.to_path_buf());
    }

    let mut children: Vec<TreeNode> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();

        for entry in entries.flatten() {
            let p = entry.path();
            // Skip hidden files/dirs
            if p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }
            if p.is_dir() {
                dirs.push(p);
            } else if FileLoader::is_markdown_file(&p) {
                files.push(p);
            }
        }

        dirs.sort();
        files.sort();

        for d in dirs {
            children.push(build_tree(&d));
        }
        for f in files {
            children.push(TreeNode::File(f));
        }
    }

    TreeNode::Dir {
        path: path.to_path_buf(),
        children,
        expanded: true,
    }
}

/// The file browser panel state
pub struct FileBrowser {
    pub roots: Vec<TreeNode>,
    pub visible: bool,
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            visible: true,
        }
    }
}

impl FileBrowser {
    /// Add a file or directory to the browser. Returns true if something was added.
    pub fn add_path(&mut self, path: PathBuf) -> bool {
        // Don't add duplicates
        if self.roots.iter().any(|r| r.path() == path) {
            return false;
        }
        self.roots.push(build_tree(&path));
        true
    }

    pub fn remove_root(&mut self, idx: usize) {
        if idx < self.roots.len() {
            self.roots.remove(idx);
        }
    }

    /// Render the browser and return any file path that was clicked.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        palette: &BrowserPalette,
    ) -> Option<PathBuf> {
        let mut clicked: Option<PathBuf> = None;

        if self.roots.is_empty() {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No files open")
                        .color(palette.muted)
                        .size(13.0),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Open a file or folder\nto see it here")
                        .color(palette.muted)
                        .size(11.0),
                );
            });
            return None;
        }

        let mut remove_idx: Option<usize> = None;

        for (i, root) in self.roots.iter_mut().enumerate() {
            // Root header with close button
            ui.horizontal(|ui| {
                let icon = if root.is_dir() { "📁" } else { "📄" };
                let name = root.name().to_string();
                ui.label(
                    egui::RichText::new(format!("{} {}", icon, name))
                        .color(palette.heading)
                        .size(12.0)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(egui::RichText::new("✕").size(10.0).color(palette.muted)).clicked() {
                        remove_idx = Some(i);
                    }
                });
            });

            let result = show_tree_node(ui, root, 0, palette);
            if result.is_some() {
                clicked = result;
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
        }

        if let Some(idx) = remove_idx {
            self.remove_root(idx);
        }

        clicked
    }
}

fn show_tree_node(
    ui: &mut egui::Ui,
    node: &mut TreeNode,
    depth: usize,
    palette: &BrowserPalette,
) -> Option<PathBuf> {
    let indent = depth as f32 * 14.0;

    match node {
        TreeNode::File(path) => {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let response = ui.horizontal(|ui| {
                ui.add_space(indent + 4.0);
                ui.label(egui::RichText::new(format!("📄 {}", name)).color(palette.file).size(12.0))
            });
            if response.inner.clicked() {
                return Some(path.clone());
            }
            None
        }
        TreeNode::Dir { path, children, expanded } => {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let arrow = if *expanded { "▾" } else { "▸" };

            let header = ui.horizontal(|ui| {
                ui.add_space(indent);
                ui.label(
                    egui::RichText::new(format!("{} 📁 {}", arrow, name))
                        .color(palette.dir)
                        .size(12.0),
                )
            });
            if header.inner.clicked() {
                *expanded = !*expanded;
            }

            if *expanded {
                for child in children.iter_mut() {
                    let result = show_tree_node(ui, child, depth + 1, palette);
                    if result.is_some() {
                        return result;
                    }
                }
            }

            None
        }
    }
}

pub struct BrowserPalette {
    pub heading: egui::Color32,
    pub file: egui::Color32,
    pub dir: egui::Color32,
    pub muted: egui::Color32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_build_tree_file() {
        let mut f = NamedTempFile::new().unwrap();
        let path = f.path().with_extension("md");
        // rename temp file to .md
        std::fs::rename(f.path(), &path).unwrap();
        let node = build_tree(&path);
        assert!(matches!(node, TreeNode::File(_)));
        assert_eq!(node.name(), path.file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn test_build_tree_dir() {
        let dir = tempdir().unwrap();
        // create some .md files
        for name in &["a.md", "b.md"] {
            let p = dir.path().join(name);
            let mut f = std::fs::File::create(&p).unwrap();
            writeln!(f, "# {}", name).unwrap();
        }
        // create a non-md file (should be excluded)
        std::fs::File::create(dir.path().join("ignored.txt")).unwrap();

        let node = build_tree(dir.path());
        match node {
            TreeNode::Dir { children, .. } => {
                // Only the 2 markdown files
                assert_eq!(children.len(), 2);
                assert!(children.iter().all(|c| matches!(c, TreeNode::File(_))));
            }
            _ => panic!("Expected Dir"),
        }
    }

    #[test]
    fn test_build_tree_nested() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("sub");
        std::fs::create_dir(&subdir).unwrap();
        let mut f = std::fs::File::create(subdir.join("nested.md")).unwrap();
        writeln!(f, "# Nested").unwrap();

        let node = build_tree(dir.path());
        match node {
            TreeNode::Dir { children, .. } => {
                assert!(!children.is_empty());
                assert!(matches!(&children[0], TreeNode::Dir { .. }));
            }
            _ => panic!("Expected Dir"),
        }
    }

    #[test]
    fn test_file_browser_add_path() {
        let mut browser = FileBrowser::default();
        let dir = tempdir().unwrap();
        assert!(browser.add_path(dir.path().to_path_buf()));
        assert_eq!(browser.roots.len(), 1);
        // Duplicate should not be added
        assert!(!browser.add_path(dir.path().to_path_buf()));
        assert_eq!(browser.roots.len(), 1);
    }

    #[test]
    fn test_file_browser_remove_root() {
        let mut browser = FileBrowser::default();
        let dir = tempdir().unwrap();
        browser.add_path(dir.path().to_path_buf());
        browser.remove_root(0);
        assert!(browser.roots.is_empty());
    }

    #[test]
    fn test_hidden_files_excluded() {
        let dir = tempdir().unwrap();
        std::fs::File::create(dir.path().join(".hidden.md")).unwrap();
        std::fs::File::create(dir.path().join("visible.md")).unwrap();
        let node = build_tree(dir.path());
        match node {
            TreeNode::Dir { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(children[0].name(), "visible.md");
            }
            _ => panic!("Expected Dir"),
        }
    }
}
