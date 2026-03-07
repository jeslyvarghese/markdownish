import Foundation
import CMarkdownCore

enum MarkdownBridgeError: Error {
    case parseFailure
    case loadFailure(path: String)
    case decodingFailure(underlying: Error)
}

struct MarkdownBridge {

    /// Parse a markdown string and return the decoded document.
    static func parse(_ text: String) throws -> MarkdownDocument {
        guard let json = text.withCString({ ptr in markdown_parse(ptr) }) else {
            throw MarkdownBridgeError.parseFailure
        }
        defer { markdown_free_string(json) }

        let data = Data(bytes: json, count: strlen(json))
        do {
            return try JSONDecoder().decode(MarkdownDocument.self, from: data)
        } catch {
            throw MarkdownBridgeError.decodingFailure(underlying: error)
        }
    }

    /// Load a file from disk and parse it.
    static func loadAndParse(path: String) throws -> MarkdownDocument {
        guard let content = path.withCString({ ptr in markdown_load_file(ptr) }) else {
            throw MarkdownBridgeError.loadFailure(path: path)
        }
        defer { markdown_free_string(content) }

        let text = String(cString: content)
        return try parse(text)
    }

    /// Check whether a file path has a markdown extension.
    static func isMarkdownFile(path: String) -> Bool {
        path.withCString { ptr in markdown_is_markdown_file(ptr) }
    }
}
