import SwiftUI

struct FileBrowserView: View {
    @Environment(AppState.self) private var appState
    @State private var directoryContents: [String: [FileItem]] = [:]

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text("Files")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(.secondary)
                    .textCase(.uppercase)
                    .tracking(0.8)
                Spacer()
                Button {
                    pickFolder()
                } label: {
                    Image(systemName: "folder.badge.plus")
                        .font(.system(size: 12))
                }
                .buttonStyle(.plain)
                .help("Open Folder")
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)

            Divider()

            // File tree
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 0) {
                    if !appState.fileBrowserRoot.isEmpty,
                       let items = directoryContents[appState.fileBrowserRoot] {
                        ForEach(items) { item in
                            FileRowView(item: item, depth: 0,
                                        directoryContents: $directoryContents)
                        }
                    } else {
                        Text("Open a folder to browse files")
                            .font(.system(size: 12))
                            .foregroundStyle(.tertiary)
                            .padding(16)
                    }
                }
                .padding(.vertical, 4)
            }
        }
        .background(Color(cgColor: appState.theme.sidebarBgColor))
        .onAppear { reloadAll() }
    }

    // Reload root + all previously expanded dirs so the tree is fully restored.
    private func reloadAll() {
        guard !appState.fileBrowserRoot.isEmpty else { return }
        loadDirectory(appState.fileBrowserRoot)
        for dir in appState.fileBrowserExpandedDirs { loadDirectory(dir) }
    }

    private func pickFolder() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK, let url = panel.url {
            appState.fileBrowserExpandedDirs = []
            directoryContents = [:]
            appState.fileBrowserRoot = url.path
            loadDirectory(url.path)
        }
    }

    func loadDirectory(_ path: String) {
        let fm = FileManager.default
        guard let entries = try? fm.contentsOfDirectory(atPath: path) else { return }
        let items = entries
            .filter { !$0.hasPrefix(".") }
            .sorted { a, b in
                let aDir = (try? fm.attributesOfItem(atPath: "\(path)/\(a)")[.type] as? FileAttributeType) == .typeDirectory
                let bDir = (try? fm.attributesOfItem(atPath: "\(path)/\(b)")[.type] as? FileAttributeType) == .typeDirectory
                if aDir != bDir { return aDir }
                return a.localizedCaseInsensitiveCompare(b) == .orderedAscending
            }
            .compactMap { name -> FileItem? in
                let full = "\(path)/\(name)"
                let attrs = try? fm.attributesOfItem(atPath: full)
                let isDir = attrs?[.type] as? FileAttributeType == .typeDirectory
                if isDir { return FileItem(path: full, name: name, isDirectory: true) }
                if MarkdownBridge.isMarkdownFile(path: full) {
                    return FileItem(path: full, name: name, isDirectory: false)
                }
                return nil
            }
        directoryContents[path] = items
    }

    struct FileItem: Identifiable {
        let id: String
        let path: String
        let name: String
        let isDirectory: Bool

        init(path: String, name: String, isDirectory: Bool) {
            self.id = path; self.path = path; self.name = name; self.isDirectory = isDirectory
        }
    }
}

// MARK: - Row

struct FileRowView: View {
    @Environment(AppState.self) private var appState
    let item: FileBrowserView.FileItem
    let depth: Int
    @Binding var directoryContents: [String: [FileBrowserView.FileItem]]

    private var isExpanded: Bool { appState.fileBrowserExpandedDirs.contains(item.path) }
    private var isActive: Bool   { appState.activeDocument?.path == item.path }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 4) {
                Spacer().frame(width: CGFloat(depth) * 16 + 8)

                if item.isDirectory {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.system(size: 9, weight: .medium))
                        .frame(width: 12)
                        .foregroundStyle(.secondary)
                } else {
                    Spacer().frame(width: 12)
                }

                Image(systemName: item.isDirectory ? "folder" : "doc.text")
                    .font(.system(size: 11))
                    .foregroundStyle(isActive ? Color(cgColor: appState.theme.headingColor) : .secondary)

                Text(item.name)
                    .font(.system(size: 12))
                    .foregroundStyle(isActive ? Color(cgColor: appState.theme.headingColor) : .primary)
                    .lineLimit(1)

                Spacer()
            }
            .padding(.vertical, 4)
            .background(isActive ? Color(cgColor: appState.theme.headingColor).opacity(0.12) : Color.clear)
            .contentShape(Rectangle())
            .onTapGesture {
                if item.isDirectory { toggleExpand() } else { appState.open(path: item.path) }
            }

            // Children
            if item.isDirectory && isExpanded {
                if let children = directoryContents[item.path] {
                    ForEach(children) { child in
                        FileRowView(item: child, depth: depth + 1,
                                    directoryContents: $directoryContents)
                    }
                }
            }
        }
    }

    private func toggleExpand() {
        if isExpanded {
            appState.fileBrowserExpandedDirs.remove(item.path)
        } else {
            appState.fileBrowserExpandedDirs.insert(item.path)
            loadIfNeeded()
        }
    }

    private func loadIfNeeded() {
        guard directoryContents[item.path] == nil else { return }
        let fm = FileManager.default
        guard let entries = try? fm.contentsOfDirectory(atPath: item.path) else { return }
        let items = entries
            .filter { !$0.hasPrefix(".") }
            .sorted { a, b in
                let aDir = (try? fm.attributesOfItem(atPath: "\(item.path)/\(a)")[.type] as? FileAttributeType) == .typeDirectory
                let bDir = (try? fm.attributesOfItem(atPath: "\(item.path)/\(b)")[.type] as? FileAttributeType) == .typeDirectory
                if aDir != bDir { return aDir }
                return a.localizedCaseInsensitiveCompare(b) == .orderedAscending
            }
            .compactMap { name -> FileBrowserView.FileItem? in
                let full = "\(item.path)/\(name)"
                let attrs = try? fm.attributesOfItem(atPath: full)
                let isDir = attrs?[.type] as? FileAttributeType == .typeDirectory
                if isDir { return FileBrowserView.FileItem(path: full, name: name, isDirectory: true) }
                if MarkdownBridge.isMarkdownFile(path: full) {
                    return FileBrowserView.FileItem(path: full, name: name, isDirectory: false)
                }
                return nil
            }
        directoryContents[item.path] = items
    }
}
