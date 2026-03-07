#!/usr/bin/env swift
// Generates MarkdownViewer.icns from programmatic Core Graphics drawing

import AppKit
import CoreGraphics

let SIZES = [16, 32, 64, 128, 256, 512, 1024]

func drawIcon(size: Int) -> CGImage? {
    let s = CGFloat(size)
    let cs = CGColorSpaceCreateDeviceRGB()
    guard let ctx = CGContext(
        data: nil, width: size, height: size,
        bitsPerComponent: 8, bytesPerRow: 0,
        space: cs, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
    ) else { return nil }

    ctx.saveGState()

    // ── Background rounded rect ──────────────────────────────────────────────
    let radius = s * 0.2237  // macOS icon radius ratio
    let bgRect = CGRect(x: 0, y: 0, width: s, height: s)
    let bgPath = CGPath(
        roundedRect: bgRect,
        cornerWidth: radius, cornerHeight: radius,
        transform: nil
    )
    ctx.addPath(bgPath)
    ctx.clip()

    // Gradient background: Catppuccin Mocha base → mantle
    let gradColors = [
        CGColor(red: 0x31/255, green: 0x32/255, blue: 0x44/255, alpha: 1), // surface0
        CGColor(red: 0x1e/255, green: 0x1e/255, blue: 0x2e/255, alpha: 1), // base
        CGColor(red: 0x18/255, green: 0x18/255, blue: 0x25/255, alpha: 1)  // mantle
    ]
    let locs: [CGFloat] = [0, 0.45, 1.0]
    let grad = CGGradient(colorsSpace: cs, colors: gradColors as CFArray, locations: locs)!
    ctx.drawLinearGradient(
        grad,
        start: CGPoint(x: s * 0.3, y: s),
        end: CGPoint(x: s * 0.7, y: 0),
        options: []
    )

    // ── Document card ────────────────────────────────────────────────────────
    let margin = s * 0.14
    let docW = s - margin * 2
    let docH = docW * 1.25
    let docX = margin
    let docY = (s - docH) / 2

    // Card shadow
    ctx.setShadow(
        offset: CGSize(width: 0, height: -s * 0.025),
        blur: s * 0.06,
        color: CGColor(red: 0, green: 0, blue: 0, alpha: 0.45)
    )

    let cardRect = CGRect(x: docX, y: docY, width: docW, height: docH)
    let cardRadius = s * 0.055
    let cardPath = CGPath(
        roundedRect: cardRect,
        cornerWidth: cardRadius, cornerHeight: cardRadius,
        transform: nil
    )
    ctx.setFillColor(CGColor(red: 0x1e/255, green: 0x1e/255, blue: 0x2e/255, alpha: 1))
    ctx.addPath(cardPath)
    ctx.fillPath()

    // Card border subtle
    ctx.setShadow(offset: .zero, blur: 0, color: nil)
    ctx.setStrokeColor(CGColor(red: 1, green: 1, blue: 1, alpha: 0.07))
    ctx.setLineWidth(s * 0.008)
    ctx.addPath(cardPath)
    ctx.strokePath()

    // ── Text lines (markdown content simulation) ─────────────────────────────
    let pad = s * 0.065
    let lineH = s * 0.058
    let startX = docX + pad
    let lineW = docW - pad * 2

    func drawLine(y: CGFloat, width: CGFloat, color: CGColor, height h: CGFloat) {
        let r = h / 2
        let rect = CGRect(x: startX, y: y, width: width, height: h)
        let path = CGPath(roundedRect: rect, cornerWidth: r, cornerHeight: r, transform: nil)
        ctx.setFillColor(color)
        ctx.addPath(path)
        ctx.fillPath()
    }

    // Heading prefix mark '#' — blue pill
    let hashW = s * 0.065
    let headY = docY + pad * 1.1
    let headH = lineH * 0.85
    drawLine(y: headY, width: hashW, color: CGColor(red: 0x89/255, green: 0xb4/255, blue: 0xfa/255, alpha: 0.9), height: headH)

    // Heading text
    drawLine(
        y: headY,
        width: lineW * 0.62,
        color: CGColor(red: 0xcdd6f4.r, green: 0xcdd6f4.g, blue: 0xcdd6f4.b, alpha: 0.92),
        height: headH
    )

    // Body lines
    let bodyStart = headY + headH + s * 0.042
    let bodyColor = CGColor(red: 0x9399b2.r, green: 0x9399b2.g, blue: 0x9399b2.b, alpha: 0.65)
    let bodyH = lineH * 0.62
    let lineSpacing = bodyH + s * 0.036

    let widths: [CGFloat] = [0.88, 0.75, 0.92, 0.55]
    for (i, w) in widths.enumerated() {
        drawLine(y: bodyStart + CGFloat(i) * lineSpacing, width: lineW * w, color: bodyColor, height: bodyH)
    }

    // Sub-heading (second heading)
    let h2Y = bodyStart + CGFloat(widths.count) * lineSpacing + s * 0.02
    let h2Color = CGColor(red: 0x89/255, green: 0xdc/255, blue: 0xeb/255, alpha: 0.85) // teal
    drawLine(y: h2Y, width: hashW * 0.75, color: h2Color, height: headH * 0.82)
    drawLine(y: h2Y, width: lineW * 0.45, color: CGColor(red: 0xcdd6f4.r, green: 0xcdd6f4.g, blue: 0xcdd6f4.b, alpha: 0.80), height: headH * 0.82)

    // More body lines
    let body2Start = h2Y + headH * 0.82 + s * 0.04
    let widths2: [CGFloat] = [0.80, 0.68]
    for (i, w) in widths2.enumerated() {
        drawLine(y: body2Start + CGFloat(i) * lineSpacing, width: lineW * w, color: bodyColor, height: bodyH)
    }

    // ── Accent glow behind card top edge ─────────────────────────────────────
    ctx.restoreGState()

    // Blue glow at top
    let glowGrad = CGGradient(
        colorsSpace: cs,
        colors: [
            CGColor(red: 0x89/255, green: 0xb4/255, blue: 0xfa/255, alpha: 0.18),
            CGColor(red: 0x89/255, green: 0xb4/255, blue: 0xfa/255, alpha: 0.0)
        ] as CFArray,
        locations: [0, 1]
    )!
    ctx.drawRadialGradient(
        glowGrad,
        startCenter: CGPoint(x: s * 0.5, y: s * 0.85),
        startRadius: 0,
        endCenter: CGPoint(x: s * 0.5, y: s * 0.85),
        endRadius: s * 0.55,
        options: []
    )

    return ctx.makeImage()
}

// Hex int color component helpers
extension Int {
    var r: CGFloat { CGFloat((self >> 16) & 0xFF) / 255 }
    var g: CGFloat { CGFloat((self >> 8) & 0xFF) / 255 }
    var b: CGFloat { CGFloat(self & 0xFF) / 255 }
}

// ── Main ─────────────────────────────────────────────────────────────────────
let args = CommandLine.arguments
guard args.count >= 2 else {
    print("Usage: gen_icon.swift <output-dir>")
    exit(1)
}

let outDir = args[1]
let fm = FileManager.default
try? fm.createDirectory(atPath: outDir, withIntermediateDirectories: true)

for size in SIZES {
    guard let img = drawIcon(size: size) else {
        print("Failed to draw size \(size)"); continue
    }
    let nsImg = NSBitmapImageRep(cgImage: img)
    nsImg.size = NSSize(width: size, height: size)
    guard let pngData = nsImg.representation(using: .png, properties: [:]) else {
        print("Failed PNG encode \(size)"); continue
    }
    let path = "\(outDir)/icon_\(size)x\(size).png"
    try! pngData.write(to: URL(fileURLWithPath: path))
    print("✓ \(path)")

    // @2x for sizes up to 512
    if size <= 512 {
        guard let img2x = drawIcon(size: size * 2) else { continue }
        let nsImg2x = NSBitmapImageRep(cgImage: img2x)
        nsImg2x.size = NSSize(width: size * 2, height: size * 2)
        guard let png2x = nsImg2x.representation(using: .png, properties: [:]) else { continue }
        let path2x = "\(outDir)/icon_\(size)x\(size)@2x.png"
        try! png2x.write(to: URL(fileURLWithPath: path2x))
        print("✓ \(path2x)")
    }
}
print("Done.")
