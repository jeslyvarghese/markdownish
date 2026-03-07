// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "MarkdownViewer",
    platforms: [.macOS(.v14)],
    targets: [
        .systemLibrary(
            name: "CMarkdownCore",
            path: "viewer/Bridge"
        ),
        .executableTarget(
            name: "MarkdownViewer",
            dependencies: ["CMarkdownCore"],
            path: "viewer",
            exclude: [
                "Bridge/MarkdownCore.h",
                "Bridge/MarkdownViewer-Bridging-Header.h",
                "Bridge/module.modulemap",
                "Renderer/Shaders.metal",   // shader is embedded as source string in DocumentRenderer.swift
            ],
            sources: [
                "App/AppState.swift",
                "App/MarkdownViewerApp.swift",
                "Bridge/MarkdownAST.swift",
                "Bridge/MarkdownBridge.swift",
                "Views/ContentView.swift",
                "Views/FileBrowserView.swift",
                "Views/SettingsView.swift",
                "Renderer/LayoutEngine.swift",
                "Renderer/SyntaxHighlighter.swift",
                "Renderer/DocumentAttributedStringBuilder.swift",
                "Renderer/DocumentView.swift",
            ],
            linkerSettings: [
                .linkedFramework("AppKit"),
                .linkedFramework("CoreText"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("UniformTypeIdentifiers"),
                .unsafeFlags(["-L", "core/target/release"]),
            ]
        ),
    ]
)
