import AppKit
import CoreGraphics
import Observation

// MARK: - CGColor helper

private func rgb(_ r: Int, _ g: Int, _ b: Int, alpha: CGFloat = 1.0) -> CGColor {
    let cs = CGColorSpaceCreateDeviceRGB()
    let comps: [CGFloat] = [CGFloat(r)/255, CGFloat(g)/255, CGFloat(b)/255, alpha]
    return CGColor(colorSpace: cs, components: comps)!
}

// MARK: - AppState

@Observable
final class AppState {

    struct OpenDocument: Identifiable {
        let id = UUID()
        var path: String
        var title: String
        var document: MarkdownDocument?
        var error: String?
        var scrollOffset: CGFloat = 0
    }

    var documents: [OpenDocument] = []
    var activeIndex: Int = 0

    init() {
        restoreSession()
        NotificationCenter.default.addObserver(
            forName: NSApplication.willTerminateNotification,
            object: nil, queue: .main
        ) { [weak self] _ in self?.saveSession() }
    }

    var activeDocument: OpenDocument? {
        guard documents.indices.contains(activeIndex) else { return nil }
        return documents[activeIndex]
    }

    var showFileBrowser: Bool = true
    var showSettings: Bool = false
    var fileBrowserWidth: CGFloat = 240
    var fileBrowserRoot: String = "" {
        didSet { saveSession() }
    }
    var fileBrowserExpandedDirs: Set<String> = [] {
        didSet { saveSession() }
    }

    var theme: Theme = .dark
    var fontSize: CGFloat = 16
    var contentWidth: ContentWidthMode = .medium

    var contentMaxWidth: CGFloat {
        switch contentWidth {
        case .narrow: return 600
        case .medium: return 800
        case .wide:   return 1100
        case .full:   return 10_000
        }
    }

    func open(path: String) {
        if let idx = documents.firstIndex(where: { $0.path == path }) {
            activeIndex = idx
            return
        }
        var doc = OpenDocument(
            path: path,
            title: URL(fileURLWithPath: path).lastPathComponent
        )
        do {
            doc.document = try MarkdownBridge.loadAndParse(path: path)
        } catch {
            doc.error = error.localizedDescription
        }
        documents.append(doc)
        activeIndex = documents.count - 1
        saveSession()
    }

    func close(at index: Int) {
        guard documents.indices.contains(index) else { return }
        documents.remove(at: index)
        activeIndex = max(0, min(activeIndex, documents.count - 1))
        saveSession()
    }

    func closeActive() { close(at: activeIndex) }

    func reload() {
        guard var doc = activeDocument else { return }
        do {
            doc.document = try MarkdownBridge.loadAndParse(path: doc.path)
            doc.error = nil
        } catch {
            doc.error = error.localizedDescription
        }
        documents[activeIndex] = doc
    }

    func setScrollOffset(_ offset: CGFloat) {
        guard documents.indices.contains(activeIndex) else { return }
        documents[activeIndex].scrollOffset = offset
    }

    func toggleTheme() {
        theme = (theme == .dark) ? .light : .dark
    }

    // MARK: - Session persistence

    func saveSession() {
        let ud = UserDefaults.standard
        ud.set(documents.map { $0.path }, forKey: "session.paths")
        ud.set(activeIndex,               forKey: "session.activeIndex")
        ud.set(theme.rawValue,            forKey: "session.theme")
        ud.set(Double(fontSize),          forKey: "session.fontSize")
        ud.set(contentWidth.rawValue,     forKey: "session.contentWidth")
        ud.set(showFileBrowser,           forKey: "session.showFileBrowser")
        ud.set(fileBrowserRoot,           forKey: "session.fileBrowserRoot")
        ud.set(Array(fileBrowserExpandedDirs), forKey: "session.fileBrowserExpandedDirs")
    }

    private func restoreSession() {
        let ud = UserDefaults.standard
        if let t = ud.string(forKey: "session.theme"),
           let restored = Theme(rawValue: t)             { theme = restored }
        let fs = ud.double(forKey: "session.fontSize")
        if fs > 0                                        { fontSize = CGFloat(fs) }
        if let w = ud.string(forKey: "session.contentWidth"),
           let restored = ContentWidthMode(rawValue: w)  { contentWidth = restored }
        if ud.object(forKey: "session.showFileBrowser") != nil {
            showFileBrowser = ud.bool(forKey: "session.showFileBrowser")
        }

        if let root = ud.string(forKey: "session.fileBrowserRoot"),
           FileManager.default.fileExists(atPath: root) { fileBrowserRoot = root }
        if let dirs = ud.stringArray(forKey: "session.fileBrowserExpandedDirs") {
            fileBrowserExpandedDirs = Set(dirs.filter { FileManager.default.fileExists(atPath: $0) })
        }

        let paths = ud.stringArray(forKey: "session.paths") ?? []
        let savedActive = ud.integer(forKey: "session.activeIndex")
        for path in paths {
            guard FileManager.default.fileExists(atPath: path) else { continue }
            open(path: path)
        }
        if documents.indices.contains(savedActive) { activeIndex = savedActive }
    }
}

// MARK: - Theme

enum Theme: String {
    case light, dark

    var backgroundColor: CGColor {
        self == .dark ? rgb(30, 30, 46) : rgb(250, 249, 247)
    }
    var textColor: CGColor {
        self == .dark ? rgb(205, 214, 244) : rgb(40, 36, 32)
    }
    var headingColor: CGColor {
        self == .dark ? rgb(137, 180, 250) : rgb(26, 82, 160)
    }
    var linkColor: CGColor {
        self == .dark ? rgb(137, 220, 235) : rgb(10, 85, 104)
    }
    var codeBgColor: CGColor {
        self == .dark ? rgb(49, 50, 68) : rgb(238, 236, 233)
    }
    var codeTextColor: CGColor {
        // Neutral default for plain identifiers; strings get syntaxString
        self == .dark ? rgb(205, 214, 244) : rgb(40, 36, 32)
    }
    var syntaxKeyword: CGColor {
        self == .dark ? rgb(203, 166, 247) : rgb(114, 31, 174)
    }
    var syntaxString: CGColor {
        self == .dark ? rgb(166, 227, 161) : rgb(45, 80, 32)
    }
    var syntaxComment: CGColor {
        self == .dark ? rgb(127, 132, 156) : rgb(107, 114, 128)
    }
    var syntaxNumber: CGColor {
        self == .dark ? rgb(250, 179, 135) : rgb(154, 52, 18)
    }
    var syntaxType: CGColor {
        self == .dark ? rgb(137, 220, 235) : rgb(10, 85, 104)
    }
    var blockquoteBorderColor: CGColor {
        self == .dark ? rgb(108, 112, 134) : rgb(180, 160, 140)
    }
    var blockquoteBgColor: CGColor {
        self == .dark ? rgb(40, 41, 56) : rgb(245, 242, 238)
    }
    var hrColor: CGColor { blockquoteBorderColor }
    var sidebarBgColor: CGColor {
        self == .dark ? rgb(24, 24, 37, alpha: 0.95) : rgb(242, 240, 237, alpha: 0.95)
    }
}

// MARK: - Enums

enum ContentWidthMode: String, CaseIterable {
    case narrow = "Narrow"
    case medium = "Medium"
    case wide   = "Wide"
    case full   = "Full"
}
