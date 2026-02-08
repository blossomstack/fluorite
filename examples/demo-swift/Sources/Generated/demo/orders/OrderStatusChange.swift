
import Foundation
/// Order status change details
public struct OrderStatusChange: Codable, Equatable, Sendable {
    public let orderId: String
    public let oldStatus: OrderStatus
    public let newStatus: OrderStatus
    public let changedAt: String
}