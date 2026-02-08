
import Foundation
/// A queued notification with delivery tracking
public struct QueuedNotification: Codable, Equatable, Sendable {
    public let id: String
    public let message: Message
    public let recipientId: String
    public let status: DeliveryStatus
    public let createdAt: String
    public let sentAt: String?
    public let deliveredAt: String?
    public let readAt: String?
}