
import Foundation
import FluoriteRuntime
/// API response wrapper
public struct ApiResponse: Codable, Equatable, Sendable {
    /// Whether the request was successful
    public let success: Bool
    /// Response data (dynamic JSON)
    public let data: AnyCodable?
    /// Error message (only present on failure)
    public let errorMessage: String?
    /// Error code (only present on failure)
    public let errorCode: String?
    /// Request ID for tracing
    public let requestId: String
}