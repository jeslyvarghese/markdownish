import AppKit
import CoreText
import Foundation

// MARK: - CGColor → NSColor (reliable, component-based)

private func nsColor(_ cg: CGColor) -> NSColor {
    guard let comps = cg.components, comps.count >= 3 else { return .white }
    return NSColor(srgbRed: comps[0], green: comps[1], blue: comps[2],
                   alpha: comps.count >= 4 ? comps[3] : 1.0)
}

// MARK: - Anchor map returned alongside the attributed string

struct BuiltDocument {
    let string: NSAttributedString
    /// heading anchor slug → character index in `string`
    let anchors: [String: Int]
}

// MARK: - Builder

enum DocumentAttributedStringBuilder {

    static func build(document: MarkdownDocument, config: RenderConfig) -> BuiltDocument {
        let result  = NSMutableAttributedString()
        var anchors = [String: Int]()

        for (i, block) in document.blocks.enumerated() {
            if i > 0 { result.append(gap(config.blockGap, config: config)) }
            appendBlock(block, to: result, anchors: &anchors, config: config, indent: 0)
        }

        return BuiltDocument(string: result, anchors: anchors)
    }

    // MARK: - Block

    private static func appendBlock(
        _ block: MarkdownBlock,
        to out: NSMutableAttributedString,
        anchors: inout [String: Int],
        config: RenderConfig,
        indent: Int
    ) {
        switch block {
        case .heading(let level, let anchor, let inlines):
            let scale   = config.headingScale[min(level - 1, 5)]
            let fs      = config.fontSize * scale
            let fName   = level <= 2 ? "Georgia-Bold" : "Georgia-BoldItalic"
            let font    = NSFont(name: fName, size: fs) ?? NSFont.systemFont(ofSize: fs, weight: .bold)
            let color   = nsColor(config.theme.headingColor)
            let ps      = headingParagraphStyle(fontSize: fs, level: level, config: config)
            anchors[anchor] = out.length
            let str = inlineString(inlines, font: font, color: color, ps: ps, config: config)
            out.append(str)

        case .paragraph(let inlines):
            let font  = NSFont(name: "Georgia", size: config.fontSize) ?? NSFont.systemFont(ofSize: config.fontSize)
            let color = nsColor(config.theme.textColor)
            let ps    = bodyParagraphStyle(config: config, indent: indent)
            out.append(inlineString(inlines, font: font, color: color, ps: ps, config: config))

        case .codeBlock(let lang, let code):
            out.append(codeBlock(lang, code: code, config: config))

        case .blockQuote(let blocks):
            out.append(blockQuote(blocks, config: config, anchors: &anchors))

        case .bulletList(let items):
            for (i, item) in items.enumerated() {
                if i > 0 { out.append(gap(config.paraGap, config: config)) }
                out.append(listItem(item, marker: "•", number: nil, config: config, anchors: &anchors, indent: indent))
            }

        case .orderedList(let start, let items):
            for (i, item) in items.enumerated() {
                if i > 0 { out.append(gap(config.paraGap, config: config)) }
                out.append(listItem(item, marker: nil, number: start + i, config: config, anchors: &anchors, indent: indent))
            }

        case .horizontalRule:
            out.append(horizontalRule(config: config))

        case .table(let aligns, let headers, let rows):
            out.append(tableBlock(aligns, headers: headers, rows: rows, config: config))
        }
    }

    // MARK: - Code block

    private static func codeBlock(_ lang: String?, code: String, config: RenderConfig) -> NSAttributedString {
        let monoFs = config.fontSize * 0.88
        let mono   = NSFont(name: "Menlo", size: monoFs)
                  ?? NSFont.monospacedSystemFont(ofSize: monoFs, weight: .regular)
        let bg = nsColor(config.theme.codeBgColor)

        // Box via NSTextTableBlock
        let table = NSTextTable()
        table.numberOfColumns = 1
        table.layoutAlgorithm = .fixedLayoutAlgorithm
        let cell = NSTextTableBlock(table: table, startingRow: 0, rowSpan: 1,
                                    startingColumn: 0, columnSpan: 1)
        cell.backgroundColor = bg
        cell.setValue(100, type: .percentageValueType, for: .minimumWidth)
        cell.setWidth(14, type: .absoluteValueType, for: .padding, edge: .minX)
        cell.setWidth(14, type: .absoluteValueType, for: .padding, edge: .maxX)
        cell.setWidth(12, type: .absoluteValueType, for: .padding, edge: .minY)
        cell.setWidth(12, type: .absoluteValueType, for: .padding, edge: .maxY)

        func cellStyle(_ spacing: CGFloat = monoFs * 0.25) -> NSMutableParagraphStyle {
            let ps = NSMutableParagraphStyle()
            ps.textBlocks = [cell]; ps.lineSpacing = spacing; ps.paragraphSpacing = 0
            return ps
        }

        let result = NSMutableAttributedString()

        // Language label
        if let lang, !lang.isEmpty {
            let lf = NSFont(name: "Menlo-Italic", size: monoFs * 0.8) ?? mono
            let lc = nsColor(config.theme.codeTextColor).withAlphaComponent(0.55)
            result.append(NSAttributedString(string: lang + "\n",
                attributes: [.font: lf, .foregroundColor: lc, .paragraphStyle: cellStyle(2)]))
        }

        // Code with syntax highlighting
        let trimmed = code.hasSuffix("\n") ? String(code.dropLast()) : code
        let codeStr = NSMutableAttributedString(string: trimmed, attributes: [
            .font: mono,
            .foregroundColor: nsColor(config.theme.codeTextColor),
            .paragraphStyle: cellStyle(),
        ])
        for span in SyntaxHighlighter.highlight(code: trimmed, language: lang) {
            codeStr.addAttribute(.foregroundColor,
                                 value: syntaxColor(span.kind, theme: config.theme),
                                 range: span.range)
        }
        result.append(codeStr)

        // Terminator required for NSTextTable cell
        result.append(NSAttributedString(string: "\n",
            attributes: [.font: mono, .paragraphStyle: cellStyle()]))
        return result
    }

    private static func syntaxColor(_ kind: TokenKind, theme: Theme) -> NSColor {
        switch kind {
        case .keyword:  return nsColor(theme.syntaxKeyword)
        case .string:   return nsColor(theme.syntaxString)
        case .comment:  return nsColor(theme.syntaxComment)
        case .number:   return nsColor(theme.syntaxNumber)
        case .typeName: return nsColor(theme.syntaxType)
        case .plain:    return nsColor(theme.codeTextColor)
        }
    }

    // MARK: - Blockquote

    private static func blockQuote(
        _ blocks: [MarkdownBlock],
        config: RenderConfig,
        anchors: inout [String: Int]
    ) -> NSAttributedString {
        let result = NSMutableAttributedString()
        let ps = NSMutableParagraphStyle()
        ps.headIndent          = 20
        ps.firstLineHeadIndent = 20
        ps.tailIndent          = -4
        ps.lineSpacing         = config.fontSize * 0.35
        ps.paragraphSpacing    = config.paraGap

        for (i, block) in blocks.enumerated() {
            if i > 0 { result.append(gap(config.paraGap, config: config)) }
            appendBlock(block, to: result, anchors: &anchors, config: config, indent: 20)
        }

        // Apply blockquote border via paragraph background (approximated with bg color)
        let bg = nsColor(config.theme.blockquoteBgColor)
        result.addAttribute(.backgroundColor, value: bg,
                            range: NSRange(location: 0, length: result.length))
        return result
    }

    // MARK: - List item

    private static func listItem(
        _ item: MarkdownListItem,
        marker: String?,
        number: Int?,
        config: RenderConfig,
        anchors: inout [String: Int],
        indent: Int
    ) -> NSAttributedString {
        let result  = NSMutableAttributedString()
        let fs      = config.fontSize
        let font    = NSFont(name: "Georgia", size: fs) ?? NSFont.systemFont(ofSize: fs)
        let color   = nsColor(config.theme.textColor)
        let indent  = CGFloat(indent) + 24
        let ps      = NSMutableParagraphStyle()
        ps.headIndent          = indent
        ps.firstLineHeadIndent = indent - 18
        ps.lineSpacing         = fs * 0.35
        ps.paragraphSpacing    = config.paraGap

        let bullet = marker ?? "\(number ?? 1)."
        result.append(NSAttributedString(string: bullet + "\t", attributes: [
            .font: font, .foregroundColor: nsColor(config.theme.textColor).withAlphaComponent(0.8)
        ]))

        for (i, block) in item.content.enumerated() {
            if i > 0 { result.append(gap(config.paraGap, config: config)) }
            appendBlock(block, to: result, anchors: &anchors, config: config, indent: Int(indent))
        }
        return result
    }

    // MARK: - Horizontal rule

    private static func horizontalRule(config: RenderConfig) -> NSAttributedString {
        let ps = NSMutableParagraphStyle()
        ps.paragraphSpacing    = 8
        ps.paragraphSpacingBefore = 8
        let color = nsColor(config.theme.hrColor).withAlphaComponent(0.4)
        return NSAttributedString(string: "\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}\u{2015}",
                                  attributes: [.foregroundColor: color,
                                               .font: NSFont.systemFont(ofSize: config.fontSize * 0.5),
                                               .paragraphStyle: ps])
    }

    // MARK: - Table (NSTextTable — proper column layout, inline formatting preserved)

    private static func tableBlock(
        _ alignments: [ColumnAlign],
        headers: [[MarkdownInline]],
        rows: [[[MarkdownInline]]],
        config: RenderConfig
    ) -> NSAttributedString {
        let colCount = max(headers.count, rows.map(\.count).max() ?? 0)
        guard colCount > 0 else { return NSAttributedString() }

        let table = NSTextTable()
        table.numberOfColumns = colCount
        table.layoutAlgorithm = .fixedLayoutAlgorithm

        let result   = NSMutableAttributedString()
        let allRows  = [headers] + rows
        let fs       = config.fontSize * 0.9
        let colPct   = CGFloat(100.0 / Double(colCount))

        for (rowIdx, rowCells) in allRows.enumerated() {
            let isHeader = rowIdx == 0
            let rowBg: NSColor = isHeader
                ? (nsColor(config.theme.codeBgColor))
                : (rowIdx % 2 == 1
                    ? (nsColor(config.theme.codeBgColor).withAlphaComponent(0.4))
                    : NSColor.clear)

            for colIdx in 0..<colCount {
                let cellInlines = colIdx < rowCells.count ? rowCells[colIdx] : []

                // Each cell is an NSTextTableBlock paragraph
                let block = NSTextTableBlock(
                    table: table,
                    startingRow: rowIdx, rowSpan: 1,
                    startingColumn: colIdx, columnSpan: 1
                )
                block.backgroundColor = rowBg
                // Equal percentage column widths
                block.setValue(colPct, type: .percentageValueType, for: .minimumWidth)
                // Cell padding
                block.setWidth(10, type: .absoluteValueType, for: .padding, edge: .minX)
                block.setWidth(10, type: .absoluteValueType, for: .padding, edge: .maxX)
                block.setWidth(8,  type: .absoluteValueType, for: .padding, edge: .minY)
                block.setWidth(8,  type: .absoluteValueType, for: .padding, edge: .maxY)

                let align = colIdx < alignments.count ? alignments[colIdx] : .none
                let ps = NSMutableParagraphStyle()
                ps.textBlocks  = [block]
                ps.lineSpacing = fs * 0.25
                switch align {
                case .center: ps.alignment = .center
                case .right:  ps.alignment = .right
                default:      ps.alignment = .left
                }

                let fontName  = isHeader ? "Georgia-Bold" : "Georgia"
                let font      = NSFont(name: fontName, size: fs) ?? NSFont.systemFont(ofSize: fs)
                let textColor = nsColor(config.theme.textColor)

                // Build cell with full inline formatting
                let cellStr = NSMutableAttributedString()
                for inline in cellInlines {
                    appendInline(inline, font: font, color: textColor, ps: ps,
                                 config: config, to: cellStr, strikethrough: false)
                }
                if cellStr.length == 0 {
                    cellStr.append(NSAttributedString(string: " ",
                                                      attributes: [.font: font,
                                                                   .foregroundColor: textColor,
                                                                   .paragraphStyle: ps]))
                }
                // The paragraph style (with textBlocks) must span the whole cell
                cellStr.addAttribute(.paragraphStyle, value: ps,
                                     range: NSRange(location: 0, length: cellStr.length))

                // NSTextTable cells are separated by paragraph terminators (\n)
                let nl = NSMutableAttributedString(string: "\n",
                                                   attributes: [.paragraphStyle: ps,
                                                                .font: font])
                cellStr.append(nl)
                result.append(cellStr)
            }
        }

        return result
    }

    // MARK: - Inline content

    private static func inlineString(
        _ inlines: [MarkdownInline],
        font: NSFont,
        color: NSColor,
        ps: NSParagraphStyle,
        config: RenderConfig
    ) -> NSAttributedString {
        let result = NSMutableAttributedString()
        for inline in inlines {
            appendInline(inline, font: font, color: color, ps: ps, config: config, to: result, strikethrough: false)
        }
        return result
    }

    private static func appendInline(
        _ inline: MarkdownInline,
        font: NSFont,
        color: NSColor,
        ps: NSParagraphStyle,
        config: RenderConfig,
        to out: NSMutableAttributedString,
        strikethrough: Bool
    ) {
        func base(_ text: String, _ f: NSFont = font, _ c: NSColor = color) {
            var attrs: [NSAttributedString.Key: Any] = [.font: f, .foregroundColor: c, .paragraphStyle: ps]
            if strikethrough { attrs[.strikethroughStyle] = NSUnderlineStyle.single.rawValue }
            out.append(NSAttributedString(string: text, attributes: attrs))
        }
        func children(_ kids: [MarkdownInline], _ f: NSFont, _ c: NSColor = color, st: Bool = strikethrough) {
            for k in kids { appendInline(k, font: f, color: c, ps: ps, config: config, to: out, strikethrough: st) }
        }

        switch inline {
        case .text(let s):
            base(s)
        case .bold(let kids):
            let bf = boldVariant(of: font)
            children(kids, bf)
        case .italic(let kids):
            let itf = italicVariant(of: font)
            children(kids, itf)
        case .strikethrough(let kids):
            for k in kids { appendInline(k, font: font, color: color, ps: ps, config: config, to: out, strikethrough: true) }
        case .code(let s):
            let monoFs = font.pointSize * 0.88
            let mono   = NSFont(name: "Menlo", size: monoFs) ?? NSFont.monospacedSystemFont(ofSize: monoFs, weight: .regular)
            let bg     = nsColor(config.theme.codeBgColor)
            let cc     = nsColor(config.theme.codeTextColor)
            var attrs: [NSAttributedString.Key: Any] = [.font: mono, .foregroundColor: cc, .backgroundColor: bg, .paragraphStyle: ps]
            if strikethrough { attrs[.strikethroughStyle] = NSUnderlineStyle.single.rawValue }
            out.append(NSAttributedString(string: s, attributes: attrs))
        case .link(let url, _, let kids):
            let lc  = nsColor(config.theme.linkColor)
            let lf  = underlineVariant(of: font)
            let sub = NSMutableAttributedString()
            for k in kids { appendInline(k, font: lf, color: lc, ps: ps, config: config, to: sub, strikethrough: strikethrough) }
            // Use custom scheme for internal (#) links so NSTextView can distinguish them
            let linkURL: URL
            if url.hasPrefix("#") {
                linkURL = URL(string: "mdviewer://anchor/\(url.dropFirst())")!
            } else {
                linkURL = URL(string: url) ?? URL(string: "about:blank")!
            }
            sub.addAttribute(.link, value: linkURL, range: NSRange(location: 0, length: sub.length))
            out.append(sub)
        case .image(_, let alt, _):
            base("[\(alt)]", font.withSize(font.pointSize), color.withAlphaComponent(0.45))
        case .softBreak:
            base(" ")
        case .hardBreak:
            base("\n")
        }
    }

    // MARK: - Paragraph styles

    private static func bodyParagraphStyle(config: RenderConfig, indent: Int) -> NSParagraphStyle {
        let ps = NSMutableParagraphStyle()
        ps.lineSpacing            = config.fontSize * 0.4
        ps.paragraphSpacing       = config.paraGap
        ps.headIndent             = CGFloat(indent)
        ps.firstLineHeadIndent    = CGFloat(indent)
        return ps
    }

    private static func headingParagraphStyle(fontSize: CGFloat, level: Int, config: RenderConfig) -> NSParagraphStyle {
        let ps = NSMutableParagraphStyle()
        ps.lineSpacing            = fontSize * 0.2
        ps.paragraphSpacing       = config.paraGap * 0.6
        ps.paragraphSpacingBefore = level <= 2 ? config.blockGap * 0.8 : config.blockGap * 0.4
        return ps
    }

    // MARK: - Gap (invisible spacer)

    private static func gap(_ height: CGFloat, config: RenderConfig) -> NSAttributedString {
        let ps = NSMutableParagraphStyle()
        ps.minimumLineHeight  = height
        ps.maximumLineHeight  = height
        ps.paragraphSpacing   = 0
        return NSAttributedString(string: "\n",
                                  attributes: [.paragraphStyle: ps,
                                               .font: NSFont.systemFont(ofSize: 1)])
    }

    // MARK: - Font helpers

    private static func boldVariant(of font: NSFont) -> NSFont {
        NSFontManager.shared.convert(font, toHaveTrait: .boldFontMask)
    }
    private static func italicVariant(of font: NSFont) -> NSFont {
        NSFontManager.shared.convert(font, toHaveTrait: .italicFontMask)
    }
    private static func underlineVariant(of font: NSFont) -> NSFont { font }
}

private extension NSFont {
    func withSize(_ s: CGFloat) -> NSFont {
        NSFont(name: fontName, size: s) ?? self
    }
}
