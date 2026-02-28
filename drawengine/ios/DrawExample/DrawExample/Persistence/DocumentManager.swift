//
//  DocumentManager.swift
//  DrawExample
//

import Foundation

/// Manages saving and loading drawing JSON files to the app's Documents directory.
enum DocumentManager {

    private static var documentsDirectory: URL {
        FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    }

    @discardableResult
    static func save(json: String) throws -> URL {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyyMMdd_HHmmss"
        let filename = "drawing_\(formatter.string(from: Date())).json"
        let url = documentsDirectory.appendingPathComponent(filename)
        try json.write(to: url, atomically: true, encoding: .utf8)
        return url
    }

    static func load(url: URL) throws -> String {
        try String(contentsOf: url, encoding: .utf8)
    }

    static func listFiles() -> [URL] {
        let fm = FileManager.default
        guard let files = try? fm.contentsOfDirectory(
            at: documentsDirectory,
            includingPropertiesForKeys: [.creationDateKey],
            options: .skipsHiddenFiles
        ) else {
            return []
        }

        return files
            .filter { $0.pathExtension == "json" && $0.lastPathComponent.hasPrefix("drawing_") }
            .sorted { a, b in
                let dateA = (try? a.resourceValues(forKeys: [.creationDateKey]))?.creationDate ?? .distantPast
                let dateB = (try? b.resourceValues(forKeys: [.creationDateKey]))?.creationDate ?? .distantPast
                return dateA > dateB
            }
    }
}
