
import Foundation
/// Order-related notification
public struct OrderNotification: Codable, Equatable, Sendable {
    public let title: String
    public let body: String
    public let orderId: String
    public let actionUrl: String?
}