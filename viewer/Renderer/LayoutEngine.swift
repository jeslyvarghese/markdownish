import CoreText
import CoreGraphics
import Foundation

// MARK: - Config

struct RenderConfig {
    var theme: Theme
    var fontSize: CGFloat
    var contentMaxWidth: CGFloat
    var scale: CGFloat
    var viewportWidth: CGFloat

    var hPad:         CGFloat { 48 }
    var contentWidth: CGFloat { min(contentMaxWidth, viewportWidth - hPad * 2) }
    var lineSpacingAdd: CGFloat { fontSize * 0.55 }   // added between lines
    var headingScale: [CGFloat] { [2.0, 1.6, 1.3, 1.15, 1.0, 0.9] }
    var blockGap:     CGFloat { fontSize * 1.1 }
    var paraGap:      CGFloat { fontSize * 0.5 }
}

// MARK: - CTParagraphStyle helper

private func makeParagraphStyle(
    alignment: CTTextAlignment = .natural,
    lineSpacingAdd: CGFloat = 0
) -> CTParagraphStyle {
    var al  = alignment
    var lsa = lineSpacingAdd
    return withUnsafeBytes(of: &al) { alBuf in
        withUnsafeBytes(of: &lsa) { lsBuf in
            let settings: [CTParagraphStyleSetting] = [
                CTParagraphStyleSetting(spec: .alignment,
                                        valueSize: MemoryLayout<CTTextAlignment>.size,
                                        value: alBuf.baseAddress!),
                CTParagraphStyleSetting(spec: .lineSpacingAdjustment,
                                        valueSize: MemoryLayout<CGFloat>.size,
                                        value: lsBuf.baseAddress!),
            ]
            return CTParagraphStyleCreate(settings, settings.count)
        }
    }
}

// MARK: - CFMutableAttributedString builder

/// Accumulates runs of text with CoreText attributes.
/// Tracks strikethrough ranges for post-draw manual rendering.
private final class AttrBuilder {
    private let ms: CFMutableAttributedString = CFAttributedStringCreateMutable(nil, 0)!
    private(set) var strikeRanges: [CFRange] = []

    /// Current character length in UTF-16 code units.
    var length: Int { CFAttributedStringGetLength(ms) }

    func append(
        _ text: String,
        font: CTFont,
        color: CGColor,
        paraStyle: CTParagraphStyle,
        strikethrough: Bool = false
    ) {
        guard !text.isEmpty else { return }
        let start  = CFAttributedStringGetLength(ms)
        let cfText = text as CFString
        CFAttributedStringReplaceString(ms, CFRange(location: start, length: 0), cfText)
        let len   = CFStringGetLength(cfText)
        let range = CFRange(location: start, length: len)
        CFAttributedStringSetAttribute(ms, range, kCTFontAttributeName, font)
        CFAttributedStringSetAttribute(ms, range, kCTForegroundColorAttributeName, color)
        CFAttributedStringSetAttribute(ms, range, kCTParagraphStyleAttributeName, paraStyle)
        if strikethrough { strikeRanges.append(range) }
    }

    func build() -> CFAttributedString { ms }
}

// MARK: - CTTextAlignment from ColumnAlign

private func ctAlign(_ align: ColumnAlign) -> CTTextAlignment {
    switch align {
    case .left:   return .left
    case .center: return .center
    case .right:  return .right
    case .none:   return .natural
    }
}

// MARK: - LayoutEngine

enum LayoutEngine {

    // MARK: Public

    static func documentHeight(document: MarkdownDocument, config: RenderConfig) -> CGFloat {
        var y = config.blockGap
        for block in document.blocks {
            y += blockHeight(block, config: config) + config.blockGap
        }
        return y
    }

    static func render(
        document: MarkdownDocument,
        scrollY: CGFloat,
        viewportWidth: CGFloat,
        viewportHeight: CGFloat,
        config: RenderConfig
    ) -> CGImage? {
        let pw = Int(viewportWidth  * config.scale)
        let ph = Int(viewportHeight * config.scale)
        guard pw > 0, ph > 0 else { return nil }

        let cs   = CGColorSpaceCreateDeviceRGB()
        let info = CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue
        guard let ctx = CGContext(data: nil, width: pw, height: ph,
                                  bitsPerComponent: 8, bytesPerRow: pw * 4,
                                  space: cs, bitmapInfo: info) else { return nil }

        // Background
        ctx.setFillColor(config.theme.backgroundColor)
        ctx.fill(CGRect(x: 0, y: 0, width: pw, height: ph))

        // Flip y so document y=0 is at top; undo for CoreText via textMatrix later
        ctx.translateBy(x: 0, y: CGFloat(ph))
        ctx.scaleBy(x: config.scale, y: -config.scale)
        ctx.translateBy(x: 0, y: -scrollY)

        // CoreText expects text matrix un-flipped within the flipped context
        ctx.textMatrix = CGAffineTransform(scaleX: 1, y: -1)

        let xOff = (viewportWidth - config.contentWidth) / 2
        let r    = BlockRenderer(ctx: ctx, config: config, x: xOff)
        var y    = config.blockGap
        for block in document.blocks {
            y += r.renderBlock(block, y: y) + config.blockGap
        }
        return ctx.makeImage()
    }

    // MARK: Height measurement (no drawing)

    static func blockHeight(_ block: MarkdownBlock, config: RenderConfig) -> CGFloat {
        switch block {
        case .heading(let level, _, let inlines):
            let fs = config.fontSize * config.headingScale[min(level - 1, 5)]
            return measureTextHeight(inlines.plainText, fontSize: fs, bold: true, width: config.contentWidth, config: config)
                   + (level <= 2 ? 14 : 6)
        case .paragraph(let inlines):
            return measureTextHeight(inlines.plainText, fontSize: config.fontSize, bold: false,
                                     width: config.contentWidth, config: config)
        case .codeBlock(_, let code):
            let mono = CTFontCreateWithName("Menlo" as CFString, config.fontSize * 0.88, nil)
            let lines = max(code.components(separatedBy: "\n").count, 1)
            return CGFloat(lines) * (CTFontGetAscent(mono) + CTFontGetDescent(mono) + config.lineSpacingAdd) + 24
        case .blockQuote(let blocks):
            return blocks.reduce(0) { $0 + blockHeight($1, config: config) + config.paraGap } + 24
        case .bulletList(let items), .orderedList(_, let items):
            return items.reduce(0) { $0 + itemHeight($1, config: config) }
        case .horizontalRule:
            return 24
        case .table(_, _, let rows):
            let rowH = config.fontSize * 1.5 + 16
            return rowH * CGFloat(rows.count + 1) + 4
        }
    }

    private static func itemHeight(_ item: MarkdownListItem, config: RenderConfig) -> CGFloat {
        item.content.reduce(0) { $0 + blockHeight($1, config: config) + config.paraGap }
    }

    static func measureTextHeight(
        _ text: String, fontSize: CGFloat, bold: Bool, width: CGFloat, config: RenderConfig
    ) -> CGFloat {
        let fontName = (bold ? "Georgia-Bold" : "Georgia") as CFString
        let font     = CTFontCreateWithName(fontName, fontSize, nil)
        let ps       = makeParagraphStyle(lineSpacingAdd: config.lineSpacingAdd)
        let builder  = AttrBuilder()
        builder.append(text.isEmpty ? " " : text, font: font, color: CGColor(gray: 0, alpha: 1), paraStyle: ps)
        return measureHeight(builder.build(), width: width)
    }

    // Measure layout height for a CFAttributedString without drawing
    static func measureHeight(_ attrStr: CFAttributedString, width: CGFloat) -> CGFloat {
        let fs   = CTFramesetterCreateWithAttributedString(attrStr)
        let size = CTFramesetterSuggestFrameSizeWithConstraints(
            fs, CFRangeMake(0, 0), nil,
            CGSize(width: width, height: .greatestFiniteMagnitude), nil)
        return size.height
    }
}

// MARK: - BlockRenderer

private final class BlockRenderer {
    let ctx: CGContext
    let cfg: RenderConfig
    let x: CGFloat

    init(ctx: CGContext, config: RenderConfig, x: CGFloat) {
        self.ctx = ctx; self.cfg = config; self.x = x
    }

    @discardableResult
    func renderBlock(_ block: MarkdownBlock, y: CGFloat) -> CGFloat {
        switch block {
        case .heading(let level, _, let inlines):     return renderHeading(inlines, level: level, y: y)
        case .paragraph(let inlines):                 return renderParagraph(inlines, y: y)
        case .codeBlock(let lang, let code):          return renderCode(lang, code: code, y: y)
        case .blockQuote(let blocks):                 return renderBlockQuote(blocks, y: y)
        case .bulletList(let items):                  return renderBullet(items, y: y)
        case .orderedList(let start, let items):      return renderOrdered(items, start: start, y: y)
        case .horizontalRule:                         return renderHR(y: y)
        case .table(let al, let hdrs, let rows):      return renderTable(al, headers: hdrs, rows: rows, y: y)
        }
    }

    // MARK: - Block types

    private func renderHeading(_ inlines: [MarkdownInline], level: Int, y: CGFloat) -> CGFloat {
        let scale   = cfg.headingScale[min(level - 1, 5)]
        let fs      = cfg.fontSize * scale
        let fName   = (level <= 2 ? "Georgia-Bold" : "Georgia-BoldItalic") as CFString
        let font    = CTFontCreateWithName(fName, fs, nil)
        let ps      = makeParagraphStyle(lineSpacingAdd: cfg.lineSpacingAdd)
        let builder = AttrBuilder()
        appendInlines(inlines, font: font, color: cfg.theme.headingColor, ps: ps, to: builder)
        let h = drawText(builder.build(), strikes: builder.strikeRanges, x: x, y: y, w: cfg.contentWidth)

        if level <= 2 {
            ctx.setStrokeColor(withAlpha(cfg.theme.headingColor, 0.2))
            ctx.setLineWidth(1)
            ctx.move(to: CGPoint(x: x, y: y + h + 5))
            ctx.addLine(to: CGPoint(x: x + cfg.contentWidth, y: y + h + 5))
            ctx.strokePath()
            return h + 14
        }
        return h + 6
    }

    private func renderParagraph(_ inlines: [MarkdownInline], y: CGFloat) -> CGFloat {
        let font    = CTFontCreateWithName("Georgia" as CFString, cfg.fontSize, nil)
        let ps      = makeParagraphStyle(lineSpacingAdd: cfg.lineSpacingAdd)
        let builder = AttrBuilder()
        appendInlines(inlines, font: font, color: cfg.theme.textColor, ps: ps, to: builder)
        return drawText(builder.build(), strikes: builder.strikeRanges, x: x, y: y, w: cfg.contentWidth)
    }

    private func renderCode(_ lang: String?, code: String, y: CGFloat) -> CGFloat {
        let monoFs  = cfg.fontSize * 0.88
        let mono    = CTFontCreateWithName("Menlo" as CFString, monoFs, nil)
        let ps      = makeParagraphStyle(lineSpacingAdd: monoFs * 0.3)
        let builder = AttrBuilder()
        let trimmed = code.hasSuffix("\n") ? String(code.dropLast()) : code
        builder.append(trimmed, font: mono, color: cfg.theme.codeTextColor, paraStyle: ps)
        let textH   = LayoutEngine.measureHeight(builder.build(), width: cfg.contentWidth - 32)
        let totalH  = textH + 24

        // Background rounded rect
        fillRoundRect(CGRect(x: x, y: y, width: cfg.contentWidth, height: totalH), r: 6, color: cfg.theme.codeBgColor)

        // Language label
        if let lang, !lang.isEmpty {
            let lblFont = CTFontCreateWithName("Menlo-Italic" as CFString, monoFs * 0.72, nil)
            let lps     = makeParagraphStyle()
            let lb      = AttrBuilder()
            lb.append(lang, font: lblFont, color: withAlpha(cfg.theme.codeTextColor, 0.8), paraStyle: lps)
            drawText(lb.build(), strikes: [], x: x + 12, y: y + 6, w: cfg.contentWidth - 24)
        }

        drawText(builder.build(), strikes: [], x: x + 16, y: y + 14, w: cfg.contentWidth - 32)
        return totalH
    }

    private func renderBlockQuote(_ blocks: [MarkdownBlock], y: CGFloat) -> CGFloat {
        let indent  = CGFloat(20)
        let contentH = blocks.reduce(CGFloat(0)) {
            $0 + LayoutEngine.blockHeight($1, config: cfg) + cfg.paraGap
        }
        let totalH = contentH + 24

        ctx.setFillColor(cfg.theme.blockquoteBorderColor)
        ctx.fill(CGRect(x: x, y: y, width: 3, height: totalH))
        ctx.setFillColor(cfg.theme.blockquoteBgColor)
        ctx.fill(CGRect(x: x + 3, y: y, width: cfg.contentWidth - 3, height: totalH))

        var innerY = y + 12
        let sub    = BlockRenderer(ctx: ctx, config: cfg, x: x + indent)
        for block in blocks {
            innerY += sub.renderBlock(block, y: innerY) + cfg.paraGap
        }
        return totalH
    }

    private func renderBullet(_ items: [MarkdownListItem], y: CGFloat) -> CGFloat {
        let indent  = CGFloat(24)
        let bFont   = CTFontCreateWithName("Georgia" as CFString, cfg.fontSize, nil)
        let bps     = makeParagraphStyle()
        var totalH  = CGFloat(0)

        for item in items {
            let bb = AttrBuilder()
            bb.append("•", font: bFont, color: withAlpha(cfg.theme.textColor, 0.5), paraStyle: bps)
            drawText(bb.build(), strikes: [], x: x + 6, y: y + totalH, w: 16)
            let sub  = BlockRenderer(ctx: ctx, config: cfg, x: x + indent)
            var itemH = CGFloat(0)
            for block in item.content {
                itemH += sub.renderBlock(block, y: y + totalH + itemH) + cfg.paraGap
            }
            totalH += max(itemH, CTFontGetAscent(bFont) + CTFontGetDescent(bFont) + cfg.lineSpacingAdd)
        }
        return totalH
    }

    private func renderOrdered(_ items: [MarkdownListItem], start: Int, y: CGFloat) -> CGFloat {
        let indent = CGFloat(32)
        let nFont  = CTFontCreateWithName("Georgia" as CFString, cfg.fontSize, nil)
        let nps    = makeParagraphStyle()
        var totalH = CGFloat(0)

        for (i, item) in items.enumerated() {
            let nb = AttrBuilder()
            nb.append("\(start + i).", font: nFont, color: withAlpha(cfg.theme.textColor, 0.5), paraStyle: nps)
            drawText(nb.build(), strikes: [], x: x, y: y + totalH, w: indent - 4)
            let sub   = BlockRenderer(ctx: ctx, config: cfg, x: x + indent)
            var itemH = CGFloat(0)
            for block in item.content {
                itemH += sub.renderBlock(block, y: y + totalH + itemH) + cfg.paraGap
            }
            totalH += max(itemH, CTFontGetAscent(nFont) + CTFontGetDescent(nFont) + cfg.lineSpacingAdd)
        }
        return totalH
    }

    private func renderHR(y: CGFloat) -> CGFloat {
        ctx.setStrokeColor(withAlpha(cfg.theme.hrColor, 0.5))
        ctx.setLineWidth(1)
        ctx.move(to: CGPoint(x: x, y: y + 12))
        ctx.addLine(to: CGPoint(x: x + cfg.contentWidth, y: y + 12))
        ctx.strokePath()
        return 24
    }

    private func renderTable(
        _ alignments: [ColumnAlign],
        headers: [[MarkdownInline]],
        rows: [[[MarkdownInline]]],
        y: CGFloat
    ) -> CGFloat {
        let cols  = max(headers.count, rows.map(\.count).max() ?? 0)
        guard cols > 0 else { return 0 }
        let colW  = cfg.contentWidth / CGFloat(cols)
        let rowH  = cfg.fontSize * 1.5 + 16
        let hdrH  = rowH + 4
        var rowY  = y

        // Header row background
        ctx.setFillColor(withAlpha(cfg.theme.codeBgColor, 0.85))
        ctx.fill(CGRect(x: x, y: rowY, width: cfg.contentWidth, height: hdrH))

        for (col, cell) in headers.enumerated() {
            let al   = col < alignments.count ? alignments[col] : .none
            let font = CTFontCreateWithName("Georgia-Bold" as CFString, cfg.fontSize * 0.9, nil)
            let ps   = makeParagraphStyle(alignment: ctAlign(al), lineSpacingAdd: cfg.lineSpacingAdd)
            let b    = AttrBuilder()
            appendInlines(cell, font: font, color: cfg.theme.textColor, ps: ps, to: b)
            drawText(b.build(), strikes: b.strikeRanges, x: x + CGFloat(col) * colW + 8, y: rowY + 10, w: colW - 16)
        }
        rowY += hdrH

        for (ri, row) in rows.enumerated() {
            if ri % 2 == 1 {
                ctx.setFillColor(withAlpha(cfg.theme.codeBgColor, 0.35))
                ctx.fill(CGRect(x: x, y: rowY, width: cfg.contentWidth, height: rowH))
            }
            for (col, cell) in row.enumerated() {
                let al   = col < alignments.count ? alignments[col] : .none
                let font = CTFontCreateWithName("Georgia" as CFString, cfg.fontSize * 0.9, nil)
                let ps   = makeParagraphStyle(alignment: ctAlign(al), lineSpacingAdd: cfg.lineSpacingAdd)
                let b    = AttrBuilder()
                appendInlines(cell, font: font, color: cfg.theme.textColor, ps: ps, to: b)
                drawText(b.build(), strikes: b.strikeRanges, x: x + CGFloat(col) * colW + 8, y: rowY + 8, w: colW - 16)
            }
            rowY += rowH
        }

        // Bottom border line
        ctx.setStrokeColor(withAlpha(cfg.theme.hrColor, 0.5))
        ctx.setLineWidth(1)
        ctx.move(to: CGPoint(x: x, y: rowY)); ctx.addLine(to: CGPoint(x: x + cfg.contentWidth, y: rowY))
        ctx.strokePath()

        return rowY - y
    }

    // MARK: - Inline builder

    private func appendInlines(
        _ inlines: [MarkdownInline],
        font: CTFont, color: CGColor, ps: CTParagraphStyle,
        to b: AttrBuilder,
        strikethrough: Bool = false
    ) {
        for inline in inlines {
            appendInline(inline, font: font, color: color, ps: ps, to: b, strikethrough: strikethrough)
        }
    }

    private func appendInline(
        _ inline: MarkdownInline,
        font: CTFont, color: CGColor, ps: CTParagraphStyle,
        to b: AttrBuilder,
        strikethrough: Bool
    ) {
        switch inline {
        case .text(let s):
            b.append(s, font: font, color: color, paraStyle: ps, strikethrough: strikethrough)

        case .bold(let children):
            let bFont = CTFontCreateCopyWithSymbolicTraits(font, 0, nil, .boldTrait, .boldTrait)
                     ?? CTFontCreateWithName("Georgia-Bold" as CFString, CTFontGetSize(font), nil)
            appendInlines(children, font: bFont, color: color, ps: ps, to: b, strikethrough: strikethrough)

        case .italic(let children):
            let iFont = CTFontCreateCopyWithSymbolicTraits(font, 0, nil, .italicTrait, .italicTrait)
                     ?? CTFontCreateWithName("Georgia-Italic" as CFString, CTFontGetSize(font), nil)
            appendInlines(children, font: iFont, color: color, ps: ps, to: b, strikethrough: strikethrough)

        case .strikethrough(let children):
            appendInlines(children, font: font, color: color, ps: ps, to: b, strikethrough: true)

        case .code(let s):
            let mono = CTFontCreateWithName("Menlo" as CFString, CTFontGetSize(font) * 0.88, nil)
            let mps  = makeParagraphStyle(lineSpacingAdd: cfg.lineSpacingAdd)
            b.append(s, font: mono, color: cfg.theme.codeTextColor, paraStyle: mps, strikethrough: strikethrough)

        case .link(let url, _, let children):
            // Draw in link color; URL stored for future hit-testing
            _ = url
            appendInlines(children, font: font, color: cfg.theme.linkColor, ps: ps, to: b, strikethrough: strikethrough)

        case .image(_, let alt, _):
            b.append("[\(alt)]", font: font, color: withAlpha(color, 0.45), paraStyle: ps)

        case .softBreak:
            b.append(" ", font: font, color: color, paraStyle: ps)

        case .hardBreak:
            b.append("\n", font: font, color: color, paraStyle: ps)
        }
    }

    // MARK: - CoreText draw primitive

    /// Lay out and draw `attrStr` using CTFramesetter.
    /// After drawing, renders manual strikethrough lines for `strikes` ranges.
    /// Returns the text height.
    @discardableResult
    func drawText(
        _ attrStr: CFAttributedString,
        strikes: [CFRange],
        x: CGFloat, y: CGFloat, w: CGFloat
    ) -> CGFloat {
        let fs   = CTFramesetterCreateWithAttributedString(attrStr)
        let sz   = CTFramesetterSuggestFrameSizeWithConstraints(
            fs, CFRangeMake(0, 0), nil,
            CGSize(width: w, height: .greatestFiniteMagnitude), nil)

        let pathRect = CGRect(x: x, y: y, width: w, height: sz.height + 2)
        let path     = CGPath(rect: pathRect, transform: nil)
        let frame    = CTFramesetterCreateFrame(fs, CFRangeMake(0, 0), path, nil)
        CTFrameDraw(frame, ctx)

        // Manual strikethrough
        if !strikes.isEmpty {
            drawStrikethrough(frame: frame, strikes: strikes, x: x, y: y, w: w)
        }

        return sz.height
    }

    /// Draw horizontal strikethrough lines for the given character ranges.
    /// Uses CTLine metrics to compute the exact vertical position.
    private func drawStrikethrough(
        frame: CTFrame,
        strikes: [CFRange],
        x: CGFloat, y: CGFloat, w: CGFloat
    ) {
        let lines = CTFrameGetLines(frame) as! [CTLine]
        var origins = [CGPoint](repeating: .zero, count: lines.count)
        CTFrameGetLineOrigins(frame, CFRangeMake(0, 0), &origins)

        for (line, origin) in zip(lines, origins) {
            var ascent: CGFloat = 0, descent: CGFloat = 0, leading: CGFloat = 0
            let lineWidth = CGFloat(CTLineGetTypographicBounds(line, &ascent, &descent, &leading))
            let lineRange = CTLineGetStringRange(line)
            let lineEnd   = lineRange.location + lineRange.length

            // Find strike ranges that overlap this line
            for sr in strikes {
                let overlapStart = max(sr.location, lineRange.location)
                let overlapEnd   = min(sr.location + sr.length, lineEnd)
                guard overlapStart < overlapEnd else { continue }

                let startX = CTLineGetOffsetForStringIndex(line, overlapStart, nil)
                let endX: CGFloat
                if overlapEnd >= lineEnd {
                    endX = lineWidth
                } else {
                    endX = CTLineGetOffsetForStringIndex(line, overlapEnd, nil)
                }

                // In our flipped context, origin.y from CTFrame is measured from the path rect top
                // CTFrameGetLineOrigins returns y from bottom of path rect; we added 2pt padding
                let lineBaseY = y + (sz_height_of_frame(frame) + 2 - origin.y) - descent
                let strikeY   = lineBaseY - ascent * 0.35    // ~35% above baseline ≈ x-height/2

                ctx.setStrokeColor(cfg.theme.textColor)
                ctx.setLineWidth(max(1, ascent * 0.06))
                ctx.move(to: CGPoint(x: x + origin.x + startX, y: strikeY))
                ctx.addLine(to: CGPoint(x: x + origin.x + endX, y: strikeY))
                ctx.strokePath()
            }
        }
    }

    // MARK: - Utilities

    private func fillRoundRect(_ rect: CGRect, r: CGFloat, color: CGColor) {
        let path = CGMutablePath()
        path.addRoundedRect(in: rect, cornerWidth: r, cornerHeight: r)
        ctx.setFillColor(color)
        ctx.addPath(path)
        ctx.fillPath()
    }
}

// MARK: - Strikethrough frame height helper

/// Returns the rendered text height from a CTFrame (distance from first line top to last line bottom).
private func sz_height_of_frame(_ frame: CTFrame) -> CGFloat {
    let lines = CTFrameGetLines(frame) as! [CTLine]
    guard !lines.isEmpty else { return 0 }
    var origins = [CGPoint](repeating: .zero, count: lines.count)
    CTFrameGetLineOrigins(frame, CFRangeMake(0, 0), &origins)
    // CTFrameGetLineOrigins gives origin.y relative to path rect bottom
    // The highest y is the first line
    var ascent: CGFloat = 0, descent: CGFloat = 0, leading: CGFloat = 0
    CTLineGetTypographicBounds(lines[0], &ascent, &descent, &leading)
    return origins[0].y + ascent
}

// MARK: - CGColor alpha helper

private func withAlpha(_ color: CGColor, _ alpha: CGFloat) -> CGColor {
    color.copy(alpha: alpha) ?? color
}
