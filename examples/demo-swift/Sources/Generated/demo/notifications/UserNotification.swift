
import Foundation
/// User-related notification
public struct UserNotification: Codable, Equatable, Sendable {
    public let title: String
    public let body: String
    public let userId: String
    public let actionUrl: String?
}