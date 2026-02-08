
import Foundation
/// Pagination metadata
public struct Pagination: Codable, Equatable, Sendable {
    /// Current page number (1-indexed)
    public let page: UInt32
    /// Number of items per page
    public let perPage: UInt32
    /// Total number of items
    public let totalItems: UInt64
    /// Total number of pages
    public let totalPages: UInt32
}