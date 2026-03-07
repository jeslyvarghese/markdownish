import Foundation

// MARK: - Public types

enum TokenKind {
    case keyword, string, comment, number, typeName, plain
}

struct HighlightSpan {
    let range: NSRange
    let kind: TokenKind
}

// MARK: - Entry point

enum SyntaxHighlighter {
    static func highlight(code: String, language: String?) -> [HighlightSpan] {
        guard let raw = language?.lowercased().trimmingCharacters(in: .whitespaces),
              !raw.isEmpty,
              let def = langDef(raw) else { return [] }
        return tokenize(code, def: def)
    }
}

// MARK: - Language definition

private struct LangDef {
    let keywords: Set<String>
    let lineComments: [String]
    let blockComments: [(open: String, close: String)]
    let strings: [(open: String, close: String, escape: Bool)]
    let isType: (String) -> Bool
}

private let uppercaseFirst: (String) -> Bool = { $0.first?.isUppercase == true }
private let neverType:      (String) -> Bool = { _ in false }

private func langDef(_ lang: String) -> LangDef? {
    switch lang {
    case "swift":
        return LangDef(
            keywords: ["func","let","var","if","else","for","while","in","return",
                       "class","struct","enum","protocol","extension","import","guard",
                       "switch","case","break","continue","do","try","catch","throw",
                       "throws","async","await","self","super","init","deinit","nil",
                       "true","false","static","final","override","private","public",
                       "internal","fileprivate","open","weak","unowned","lazy",
                       "mutating","some","any","associatedtype","typealias","where",
                       "inout","defer","repeat","as","is","get","set","willSet","didSet",
                       "@escaping","@objc","@available","@discardableResult","@main",
                       "@State","@Binding","@Environment","@Published","nonmutating"],
            lineComments: ["//"], blockComments: [("/*","*/")],
            strings: [("\"\"\"","\"\"\"",true),("\"","\"",true)],
            isType: uppercaseFirst)

    case "rust","rs":
        return LangDef(
            keywords: ["fn","let","mut","if","else","for","while","in","return",
                       "struct","enum","trait","impl","use","mod","pub","super",
                       "self","Self","true","false","match","loop","break","continue",
                       "async","await","move","ref","const","static","type","where",
                       "dyn","crate","extern","unsafe","as","typeof"],
            lineComments: ["//"], blockComments: [("/*","*/")],
            strings: [("\"","\"",true),("'","'",false)],
            isType: uppercaseFirst)

    case "python","py":
        return LangDef(
            keywords: ["def","class","if","elif","else","for","while","in","return",
                       "import","from","as","try","except","finally","raise","with",
                       "lambda","yield","del","pass","break","continue","True","False",
                       "None","and","or","not","is","global","nonlocal","async","await"],
            lineComments: ["#"], blockComments: [],
            strings: [("\"\"\"","\"\"\"",true),("'''","'''",true),
                      ("\"","\"",true),("'","'",true)],
            isType: uppercaseFirst)

    case "javascript","js","typescript","ts","jsx","tsx":
        return LangDef(
            keywords: ["function","const","let","var","if","else","for","while",
                       "return","class","import","export","default","from","async",
                       "await","try","catch","finally","throw","new","this","super",
                       "true","false","null","undefined","typeof","instanceof",
                       "in","of","switch","case","break","continue","do","delete",
                       "void","type","interface","extends","implements","readonly","enum"],
            lineComments: ["//"], blockComments: [("/*","*/")],
            strings: [("\"","\"",true),("'","'",true),("`","`",true)],
            isType: uppercaseFirst)

    case "go":
        return LangDef(
            keywords: ["func","var","const","type","if","else","for","range",
                       "return","struct","interface","package","import","switch",
                       "case","default","break","continue","go","defer","select",
                       "chan","map","true","false","nil","make","new","len",
                       "cap","append","copy","delete","panic","recover","close"],
            lineComments: ["//"], blockComments: [("/*","*/")],
            strings: [("\"","\"",true),("`","`",false)],
            isType: uppercaseFirst)

    case "sh","bash","shell","zsh","fish":
        return LangDef(
            keywords: ["if","then","else","elif","fi","for","do","done","while",
                       "case","esac","function","in","return","export","local",
                       "echo","cd","mkdir","rm","cp","mv","ls","cat","grep",
                       "sed","awk","source","set","unset","readonly"],
            lineComments: ["#"], blockComments: [],
            strings: [("\"","\"",true),("'","'",false)],
            isType: neverType)

    case "json":
        return LangDef(
            keywords: ["true","false","null"],
            lineComments: [], blockComments: [],
            strings: [("\"","\"",true)],
            isType: neverType)

    case "toml":
        return LangDef(
            keywords: ["true","false"],
            lineComments: ["#"], blockComments: [],
            strings: [("\"\"\"","\"\"\"",true),("'''","'''",false),
                      ("\"","\"",true),("'","'",false)],
            isType: neverType)

    case "yaml","yml":
        return LangDef(
            keywords: ["true","false","null","yes","no","on","off"],
            lineComments: ["#"], blockComments: [],
            strings: [("\"","\"",true),("'","'",false)],
            isType: neverType)

    case "c","cpp","c++","h","hpp","cc":
        return LangDef(
            keywords: ["int","float","double","char","void","bool","long","short",
                       "unsigned","signed","const","static","extern","if","else",
                       "for","while","do","switch","case","break","continue","return",
                       "struct","union","enum","typedef","sizeof","nullptr","true",
                       "false","NULL","include","define","ifdef","ifndef","endif",
                       "class","public","private","protected","virtual","new","delete",
                       "namespace","using","template","typename","auto","this"],
            lineComments: ["//"], blockComments: [("/*","*/")],
            strings: [("\"","\"",true),("'","'",false)],
            isType: uppercaseFirst)

    case "java","kotlin","kt":
        return LangDef(
            keywords: ["class","interface","fun","val","var","if","else","for",
                       "while","return","import","package","object","companion",
                       "data","sealed","enum","when","in","is","as","null",
                       "true","false","override","abstract","open","public","private",
                       "protected","static","final","void","new","this","super",
                       "try","catch","finally","throw","throws","extends","implements",
                       "by","lazy","init","constructor","suspend","inline","reified"],
            lineComments: ["//"], blockComments: [("/*","*/")],
            strings: [("\"\"\"","\"\"\"",true),("\"","\"",true)],
            isType: uppercaseFirst)

    default: return nil
    }
}

// MARK: - Tokeniser

private func tokenize(_ code: String, def: LangDef) -> [HighlightSpan] {
    var spans: [HighlightSpan] = []
    var i = code.startIndex

    while i < code.endIndex {
        if let (end, kind) = scanBlockComment(code, at: i, def: def) {
            spans.append(HighlightSpan(range: NSRange(i..<end, in: code), kind: kind))
            i = end; continue
        }
        if let end = scanLineComment(code, at: i, def: def) {
            spans.append(HighlightSpan(range: NSRange(i..<end, in: code), kind: .comment))
            i = end; continue
        }
        if let (end, kind) = scanString(code, at: i, def: def) {
            spans.append(HighlightSpan(range: NSRange(i..<end, in: code), kind: kind))
            i = end; continue
        }
        if let end = scanNumber(code, at: i) {
            spans.append(HighlightSpan(range: NSRange(i..<end, in: code), kind: .number))
            i = end; continue
        }
        if let (end, kind) = scanIdentifier(code, at: i, def: def) {
            if kind != .plain {
                spans.append(HighlightSpan(range: NSRange(i..<end, in: code), kind: kind))
            }
            i = end; continue
        }
        i = code.index(after: i)
    }
    return spans
}

// MARK: - Scan helpers

private func scanBlockComment(_ s: String, at i: String.Index, def: LangDef) -> (String.Index, TokenKind)? {
    for (open, close) in def.blockComments {
        guard s[i...].hasPrefix(open) else { continue }
        var j = advance(s, from: i, by: open.count)
        while j < s.endIndex && !s[j...].hasPrefix(close) {
            j = s.index(after: j)
        }
        if j < s.endIndex { j = advance(s, from: j, by: close.count) }
        return (j, .comment)
    }
    return nil
}

private func scanLineComment(_ s: String, at i: String.Index, def: LangDef) -> String.Index? {
    for prefix in def.lineComments {
        guard s[i...].hasPrefix(prefix) else { continue }
        var j = i
        while j < s.endIndex && s[j] != "\n" { j = s.index(after: j) }
        return j
    }
    return nil
}

private func scanString(_ s: String, at i: String.Index, def: LangDef) -> (String.Index, TokenKind)? {
    // Sort by open length descending so longer delimiters (""") are tried first
    let sorted = def.strings.sorted { $0.open.count > $1.open.count }
    for delim in sorted {
        guard s[i...].hasPrefix(delim.open) else { continue }
        var j = advance(s, from: i, by: delim.open.count)
        while j < s.endIndex {
            if delim.escape && s[j] == "\\" {
                j = s.index(after: j)
                if j < s.endIndex { j = s.index(after: j) }
            } else if s[j...].hasPrefix(delim.close) {
                j = advance(s, from: j, by: delim.close.count)
                return (j, .string)
            } else {
                j = s.index(after: j)
            }
        }
        return (j, .string) // unterminated
    }
    return nil
}

private func scanNumber(_ s: String, at i: String.Index) -> String.Index? {
    guard s[i].isNumber else { return nil }
    var j = i
    // Hex / binary / octal prefix
    if s[i] == "0" {
        let next = advance(s, from: i, by: 1)
        if next < s.endIndex && (s[next] == "x" || s[next] == "b" || s[next] == "o") {
            j = advance(s, from: next, by: 1)
            while j < s.endIndex && (s[j].isHexDigit || s[j] == "_") {
                j = s.index(after: j)
            }
            return j
        }
    }
    // Integer part
    while j < s.endIndex && (s[j].isNumber || s[j] == "_") { j = s.index(after: j) }
    // Fractional part
    if j < s.endIndex && s[j] == "." {
        let dot = s.index(after: j)
        if dot < s.endIndex && s[dot].isNumber {
            j = dot
            while j < s.endIndex && (s[j].isNumber || s[j] == "_") { j = s.index(after: j) }
        }
    }
    // Exponent
    if j < s.endIndex && (s[j] == "e" || s[j] == "E") {
        let e = s.index(after: j)
        var k = e
        if k < s.endIndex && (s[k] == "+" || s[k] == "-") { k = s.index(after: k) }
        if k < s.endIndex && s[k].isNumber {
            j = k
            while j < s.endIndex && s[j].isNumber { j = s.index(after: j) }
        }
    }
    return j > i ? j : nil
}

private func scanIdentifier(_ s: String, at i: String.Index, def: LangDef) -> (String.Index, TokenKind)? {
    let c = s[i]
    guard c.isLetter || c == "_" else { return nil }
    var j = i
    while j < s.endIndex && (s[j].isLetter || s[j].isNumber || s[j] == "_") {
        j = s.index(after: j)
    }
    let word = String(s[i..<j])
    if def.keywords.contains(word) { return (j, .keyword) }
    if def.isType(word)            { return (j, .typeName) }
    return (j, .plain)
}

// MARK: - Utility

private func advance(_ s: String, from i: String.Index, by n: Int) -> String.Index {
    s.index(i, offsetBy: n, limitedBy: s.endIndex) ?? s.endIndex
}
