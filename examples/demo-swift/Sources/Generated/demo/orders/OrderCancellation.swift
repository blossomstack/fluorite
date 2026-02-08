
import Foundation
/// Order cancellation details
public struct OrderCancellation: Codable, Equatable, Sendable {
    public let orderId: String
    public let reason: String
    public let refundAmount: String?
    public let cancelledAt: String
}