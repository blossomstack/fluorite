
/// Possible statuses for a user account
public enum UserStatus: String, Codable, Equatable, Sendable {
    case active = "Active"
    case inactive = "Inactive"
    case suspended = "Suspended"
    case pending = "Pending"
}