import SwiftUI

@main
struct MarkdownViewerApp: App {
    @State private var appState = AppState()

    var body: some Scene {
        Window("Markdown Viewer", id: "main") {
            ContentView()
                .environment(appState)
                .frame(minWidth: 500, minHeight: 400)
        }
        .defaultSize(width: 1100, height: 780)
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open File…") {
                    openFile()
                }
                .keyboardShortcut("o", modifiers: .command)
            }
            CommandGroup(after: .newItem) {
                Button("Close Tab") {
                    appState.closeActive()
                }
                .keyboardShortcut("w", modifiers: .command)

                Button("Reload") {
                    appState.reload()
                }
                .keyboardShortcut("r", modifiers: .command)
            }
            CommandGroup(replacing: .sidebar) {
                Button(appState.showFileBrowser ? "Hide Sidebar" : "Show Sidebar") {
                    appState.showFileBrowser.toggle()
                }
                .keyboardShortcut("b", modifiers: .command)
            }
        }
    }

    private func openFile() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.init(filenameExtension: "md")!,
                                     .init(filenameExtension: "markdown")!]
        panel.allowsMultipleSelection = true
        panel.canChooseDirectories = false
        if panel.runModal() == .OK {
            for url in panel.urls {
                appState.open(path: url.path)
            }
        }
    }
}
