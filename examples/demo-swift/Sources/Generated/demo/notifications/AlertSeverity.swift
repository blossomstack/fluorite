
/// Alert severity levels
public enum AlertSeverity: String, Codable, Equatable, Sendable {
    case info = "Info"
    case warning = "Warning"
    case error = "Error"
    case critical = "Critical"
}