
import Foundation
/// Status change details
public struct UserStatusChange: Codable, Equatable, Sendable {
    public let userId: String
    public let oldStatus: UserStatus
    public let newStatus: UserStatus
    public let changedAt: String
}