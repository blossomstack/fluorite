
import Foundation
import FluoriteRuntime
/// A user in the system
public struct User: Codable, Equatable, Sendable {
    /// Unique identifier for the user
    public let id: String
    /// User's first name
    public let firstName: String
    /// User's last name
    public let lastName: String
    /// User's email address
    public let email: String
    /// Optional age of the user
    public let age: UInt32?
    /// User's account status
    public let status: UserStatus
    /// User's gender
    public let gender: Gender
    /// Whether the account is active
    public let active: Bool
    /// User's home address (imported from common)
    public let homeAddress: Address?
    /// When the user was created
    public let createdAt: String
    /// Additional metadata (dynamic JSON)
    public let info: AnyCodable?
}