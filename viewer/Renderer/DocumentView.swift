import SwiftUI
import AppKit

// MARK: - CGColor → NSColor helper (shared with builder)

private func nsColor(_ cg: CGColor) -> NSColor {
    guard let comps = cg.components, comps.count >= 3 else { return .white }
    return NSColor(srgbRed: comps[0], green: comps[1], blue: comps[2],
                   alpha: comps.count >= 4 ? comps[3] : 1.0)
}

// MARK: - SwiftUI wrapper

struct DocumentView: NSViewRepresentable {
    let document: MarkdownDocument
    let theme: Theme
    let fontSize: CGFloat
    @Environment(AppState.self) private var appState

    func makeNSView(context: Context) -> NSScrollView {
        let textView = makeTextView(coordinator: context.coordinator)
        let scrollView = NSScrollView()
        scrollView.documentView             = textView
        scrollView.hasVerticalScroller      = true
        scrollView.hasHorizontalScroller    = false
        scrollView.autohidesScrollers       = true
        scrollView.drawsBackground          = false
        context.coordinator.textView        = textView
        context.coordinator.scrollView      = scrollView
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? NSTextView else { return }
        let config = makeConfig(scrollView: scrollView)
        let built  = DocumentAttributedStringBuilder.build(document: document, config: config)
        context.coordinator.anchors = built.anchors

        textView.textStorage?.setAttributedString(built.string)
        applyAppearance(textView: textView, scrollView: scrollView, config: config)
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    // MARK: - Helpers

    private func makeConfig(scrollView: NSScrollView) -> RenderConfig {
        RenderConfig(
            theme:          appState.theme,
            fontSize:       appState.fontSize,
            contentMaxWidth: appState.contentMaxWidth,
            scale:          scrollView.window?.backingScaleFactor ?? 2.0,
            viewportWidth:  scrollView.contentView.bounds.width
        )
    }

    private func makeTextView(coordinator: Coordinator) -> NSTextView {
        let textView = NSTextView()
        textView.isEditable                       = false
        textView.isSelectable                     = true
        textView.allowsUndo                       = false
        textView.isRichText                       = true
        textView.usesFindBar                      = true
        textView.isAutomaticLinkDetectionEnabled  = false
        textView.isAutomaticQuoteSubstitutionEnabled = false
        textView.drawsBackground                  = true
        textView.autoresizingMask                 = [.width]
        textView.isVerticallyResizable            = true
        textView.isHorizontallyResizable          = false
        textView.textContainer?.widthTracksTextView = true
        textView.textContainer?.heightTracksTextView = false
        textView.delegate                         = coordinator
        textView.linkTextAttributes               = [:]   // handled via delegate
        return textView
    }

    private func applyAppearance(textView: NSTextView, scrollView: NSScrollView, config: RenderConfig) {
        let bg = nsColor(config.theme.backgroundColor)
        textView.drawsBackground   = true
        textView.backgroundColor   = bg
        scrollView.drawsBackground = true
        scrollView.backgroundColor = bg
        // Center content up to max width with horizontal padding
        let viewW  = scrollView.contentView.bounds.width
        let hPad   = max(48, (viewW - config.contentMaxWidth) / 2)
        textView.textContainerInset = NSSize(width: hPad, height: 56)

        // Link color
        textView.linkTextAttributes = [
            .foregroundColor: nsColor(config.theme.linkColor),
            .cursor: NSCursor.pointingHand,
        ]
    }
}

// MARK: - Coordinator (link + scroll)

final class Coordinator: NSObject, NSTextViewDelegate {
    weak var textView:   NSTextView?
    weak var scrollView: NSScrollView?
    var anchors: [String: Int] = [:]

    func textView(_ textView: NSTextView, clickedOnLink link: Any, at charIndex: Int) -> Bool {
        let url: URL?
        if let u = link as? URL        { url = u }
        else if let s = link as? String { url = URL(string: s) }
        else { return false }

        guard let url else { return false }

        if url.scheme == "mdviewer", url.host == "anchor" {
            let slug = url.pathComponents.dropFirst().joined(separator: "/")
            scrollToAnchor(slug, in: textView)
            return true
        }

        if url.scheme == "http" || url.scheme == "https" || url.scheme == "mailto" {
            NSWorkspace.shared.open(url)
            return true
        }

        // Relative / file links — try opening with default app
        NSWorkspace.shared.open(url)
        return true
    }

    private func scrollToAnchor(_ slug: String, in textView: NSTextView) {
        guard let charIdx = anchors[slug] else { return }
        let range = NSRange(location: charIdx, length: 0)
        textView.scrollRangeToVisible(range)
        // Nudge up slightly so heading isn't at the very top edge
        if let sv = scrollView {
            let pt = sv.documentVisibleRect.origin
            sv.documentView?.scroll(NSPoint(x: pt.x, y: max(0, pt.y - 56)))
        }
    }
}

// MARK: - RenderConfig lives here now (used by both builder and view)

extension RenderConfig {
    init(appState: AppState, viewWidth: CGFloat, scale: CGFloat) {
        self.init(
            theme:           appState.theme,
            fontSize:        appState.fontSize,
            contentMaxWidth: appState.contentMaxWidth,
            scale:           scale,
            viewportWidth:   viewWidth
        )
    }
}
