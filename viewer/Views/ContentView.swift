import SwiftUI
import UniformTypeIdentifiers

struct ContentView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        @Bindable var state = appState

        HStack(spacing: 0) {
            // Sidebar — always in hierarchy so @State survives hide/show
            FileBrowserView()
                .frame(width: appState.showFileBrowser ? appState.fileBrowserWidth : 0)
                .opacity(appState.showFileBrowser ? 1 : 0)
                .clipped()

            if appState.showFileBrowser {
                Divider()
            }

            // Main area
            VStack(spacing: 0) {
                // Tab bar
                if !appState.documents.isEmpty {
                    TabBar()
                }

                // Document or welcome screen
                if let doc = appState.activeDocument {
                    if let err = doc.error {
                        ErrorView(message: err)
                    } else if let document = doc.document {
                        DocumentView(document: document, theme: appState.theme, fontSize: appState.fontSize)
                    } else {
                        ProgressView()
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    }
                } else {
                    WelcomeView()
                }
            }

            // Settings panel
            if appState.showSettings {
                Divider()
                SettingsView()
                    .frame(width: 280)
                    .transition(.move(edge: .trailing))
            }
        }
        .toolbar {
            ToolbarItemGroup(placement: .navigation) {
                Button {
                    withAnimation(.spring(duration: 0.25)) {
                        appState.showFileBrowser.toggle()
                    }
                } label: {
                    Image(systemName: "sidebar.left")
                }
                .help("Toggle Sidebar (⌘B)")
            }

            ToolbarItemGroup(placement: .primaryAction) {
                Button {
                    withAnimation(.spring(duration: 0.25)) {
                        appState.showSettings.toggle()
                    }
                } label: {
                    Image(systemName: "slider.horizontal.3")
                }
                .help("Settings")
            }
        }
        .onDrop(of: [.fileURL], isTargeted: nil) { providers in
            Task {
                for provider in providers {
                    if let url = try? await provider.loadItem(forTypeIdentifier: UTType.fileURL.identifier) as? URL,
                       MarkdownBridge.isMarkdownFile(path: url.path) {
                        await MainActor.run { appState.open(path: url.path) }
                    }
                }
            }
            return true
        }
        .background(Color(cgColor: appState.theme.backgroundColor))
        .preferredColorScheme(appState.theme == .dark ? .dark : .light)
    }
}

// MARK: - Tab bar

struct TabBar: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 0) {
                ForEach(appState.documents.indices, id: \.self) { idx in
                    let doc = appState.documents[idx]
                    let isActive = idx == appState.activeIndex

                    HStack(spacing: 6) {
                        Image(systemName: "doc.text")
                            .font(.system(size: 11))
                            .opacity(0.6)
                        Text(doc.title)
                            .font(.system(size: 12, weight: isActive ? .medium : .regular))
                            .lineLimit(1)
                        Button {
                            appState.close(at: idx)
                        } label: {
                            Image(systemName: "xmark")
                                .font(.system(size: 9, weight: .medium))
                        }
                        .buttonStyle(.plain)
                        .opacity(isActive ? 1 : 0.4)
                        .padding(.leading, 2)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 7)
                    .background(isActive
                        ? Color(cgColor: appState.theme.backgroundColor)
                        : Color.clear)
                    .overlay(alignment: .bottom) {
                        if isActive {
                            Rectangle()
                                .frame(height: 2)
                                .foregroundStyle(Color(cgColor: appState.theme.headingColor))
                        }
                    }
                    .contentShape(Rectangle())
                    .onTapGesture { appState.activeIndex = idx }
                }
            }
        }
        .frame(height: 34)
        .background(.ultraThinMaterial)
        .overlay(alignment: .bottom) {
            Divider()
        }
    }
}

// MARK: - Welcome screen

struct WelcomeView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        VStack(spacing: 24) {
            Image(systemName: "doc.text.magnifyingglass")
                .font(.system(size: 56, weight: .ultraLight))
                .foregroundStyle(.tertiary)

            VStack(spacing: 6) {
                Text("Markdownish")
                    .font(.system(size: 22, weight: .semibold, design: .serif))
                Text("Open a file to begin reading")
                    .font(.system(size: 14))
                    .foregroundStyle(.secondary)
            }

            Button("Open File…") {
                openFile()
            }
            .keyboardShortcut("o", modifiers: .command)
            .controlSize(.large)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(cgColor: appState.theme.backgroundColor))
    }

    private func openFile() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.init(filenameExtension: "md")!]
        panel.allowsMultipleSelection = true
        if panel.runModal() == .OK {
            for url in panel.urls {
                appState.open(path: url.path)
            }
        }
    }
}

// MARK: - Error view

struct ErrorView: View {
    let message: String
    @Environment(AppState.self) private var appState

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 40, weight: .ultraLight))
                .foregroundStyle(.orange)
            Text("Failed to open document")
                .font(.system(size: 16, weight: .medium))
            Text(message)
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .padding(40)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(cgColor: appState.theme.backgroundColor))
    }
}
