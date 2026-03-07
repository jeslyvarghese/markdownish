import Foundation

// MARK: - Root

struct MarkdownDocument: Decodable {
    let version: Int
    let blocks: [MarkdownBlock]
}

// MARK: - Block

enum MarkdownBlock: Decodable {
    case heading(level: Int, anchor: String, inlines: [MarkdownInline])
    case paragraph(inlines: [MarkdownInline])
    case codeBlock(language: String?, code: String)
    case blockQuote(blocks: [MarkdownBlock])
    case bulletList(items: [MarkdownListItem])
    case orderedList(start: Int, items: [MarkdownListItem])
    case horizontalRule
    case table(alignments: [ColumnAlign], headers: [[MarkdownInline]], rows: [[[MarkdownInline]]])

    private enum CodingKeys: String, CodingKey {
        case type, level, anchor, inlines, language, code
        case blocks, items, start, alignments, headers, rows
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let type_ = try c.decode(String.self, forKey: .type)
        switch type_ {
        case "heading":
            self = .heading(
                level: try c.decode(Int.self, forKey: .level),
                anchor: try c.decode(String.self, forKey: .anchor),
                inlines: try c.decode([MarkdownInline].self, forKey: .inlines)
            )
        case "paragraph":
            self = .paragraph(inlines: try c.decode([MarkdownInline].self, forKey: .inlines))
        case "code_block":
            self = .codeBlock(
                language: try c.decodeIfPresent(String.self, forKey: .language),
                code: try c.decode(String.self, forKey: .code)
            )
        case "block_quote":
            self = .blockQuote(blocks: try c.decode([MarkdownBlock].self, forKey: .blocks))
        case "bullet_list":
            self = .bulletList(items: try c.decode([MarkdownListItem].self, forKey: .items))
        case "ordered_list":
            self = .orderedList(
                start: try c.decode(Int.self, forKey: .start),
                items: try c.decode([MarkdownListItem].self, forKey: .items)
            )
        case "horizontal_rule":
            self = .horizontalRule
        case "table":
            self = .table(
                alignments: try c.decode([ColumnAlign].self, forKey: .alignments),
                headers: try c.decode([[MarkdownInline]].self, forKey: .headers),
                rows: try c.decode([[[MarkdownInline]]].self, forKey: .rows)
            )
        default:
            self = .paragraph(inlines: [])
        }
    }
}

// MARK: - Inline

indirect enum MarkdownInline: Decodable {
    case text(String)
    case bold([MarkdownInline])
    case italic([MarkdownInline])
    case strikethrough([MarkdownInline])
    case code(String)
    case link(url: String, title: String?, children: [MarkdownInline])
    case image(url: String, alt: String, title: String?)
    case softBreak
    case hardBreak

    private enum CodingKeys: String, CodingKey {
        case type, content, children, url, title, alt
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let type_ = try c.decode(String.self, forKey: .type)
        switch type_ {
        case "text":
            self = .text(try c.decode(String.self, forKey: .content))
        case "bold":
            self = .bold(try c.decode([MarkdownInline].self, forKey: .children))
        case "italic":
            self = .italic(try c.decode([MarkdownInline].self, forKey: .children))
        case "strikethrough":
            self = .strikethrough(try c.decode([MarkdownInline].self, forKey: .children))
        case "code":
            self = .code(try c.decode(String.self, forKey: .content))
        case "link":
            self = .link(
                url: try c.decode(String.self, forKey: .url),
                title: try c.decodeIfPresent(String.self, forKey: .title),
                children: try c.decode([MarkdownInline].self, forKey: .children)
            )
        case "image":
            self = .image(
                url: try c.decode(String.self, forKey: .url),
                alt: try c.decode(String.self, forKey: .alt),
                title: try c.decodeIfPresent(String.self, forKey: .title)
            )
        case "soft_break":
            self = .softBreak
        case "hard_break":
            self = .hardBreak
        default:
            self = .text("")
        }
    }
}

// MARK: - Supporting types

struct MarkdownListItem: Decodable {
    let content: [MarkdownBlock]
}

enum ColumnAlign: String, Decodable {
    case none, left, center, right
}

// MARK: - Plain text extraction

extension [MarkdownInline] {
    var plainText: String {
        map { $0.plainText }.joined()
    }
}

extension MarkdownInline {
    var plainText: String {
        switch self {
        case .text(let s): return s
        case .bold(let c), .italic(let c), .strikethrough(let c): return c.plainText
        case .code(let s): return s
        case .link(_, _, let c): return c.plainText
        case .image(_, let alt, _): return alt
        case .softBreak: return " "
        case .hardBreak: return "\n"
        }
    }
}
