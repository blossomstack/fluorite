
import Foundation
/// System alert
public struct SystemAlert: Codable, Equatable, Sendable {
    public let title: String
    public let body: String
    public let severity: AlertSeverity
    public let expiresAt: String?
}