#![allow(dead_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::wildcard_enum_match_arm)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::wildcard_enum_match_arm,
        deprecated
    )
)]

use std::fs;
use std::io::Write;
use std::path::Path;

use clap::{Parser, Subcommand};

// Include generated modules from multiple packages
#[allow(clippy::too_many_arguments)]
mod demo {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/demo/common/mod.rs"));
    }
    pub mod users {
        include!(concat!(env!("OUT_DIR"), "/demo/users/mod.rs"));
    }
    pub mod orders {
        include!(concat!(env!("OUT_DIR"), "/demo/orders/mod.rs"));
    }
    pub mod notifications {
        include!(concat!(env!("OUT_DIR"), "/demo/notifications/mod.rs"));
    }
}

// Re-export commonly used types
use demo::common::{Address, ApiResponse, Pagination};
use demo::notifications::{
    AlertSeverity, DeliveryStatus, Message, OrderNotification, QueuedNotification, SystemAlert,
    UserNotification,
};
use demo::orders::{
    Order, OrderCancellation, OrderEvent, OrderItem, OrderStatus, OrderStatusChange,
};
use demo::users::{Gender, User, UserEvent, UserStatus, UserStatusChange};

#[derive(Parser)]
#[command(name = "demo")]
#[command(about = "Fluorite demo with multi-package cross-imports")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Write sample data to JSON files
    Write {
        /// Output directory for JSON files
        #[arg(short, long, default_value = "./fixtures")]
        output: String,
    },
    /// Read and validate JSON files
    Read {
        /// Input directory containing JSON files
        #[arg(short, long, default_value = "./fixtures")]
        input: String,
    },
}

// =============================================================================
// Sample data creators - Common
// =============================================================================

fn create_sample_address() -> Address {
    Address {
        street1: "123 Main St".to_string(),
        street2: Some("Apt 4B".to_string()),
        city: "New York".to_string(),
        state: "NY".to_string(),
        postal_code: "10001".to_string(),
        country: "US".to_string(),
    }
}

fn create_sample_api_response_success() -> ApiResponse {
    ApiResponse {
        success: true,
        data: Some(serde_json::json!({"users": 42, "orders": 15})),
        error_message: None,
        error_code: None,
        request_id: "req-12345".to_string(),
    }
}

fn create_sample_api_response_error() -> ApiResponse {
    ApiResponse {
        success: false,
        data: None,
        error_message: Some("User not found".to_string()),
        error_code: Some("USER_NOT_FOUND".to_string()),
        request_id: "req-12346".to_string(),
    }
}

fn create_sample_pagination() -> Pagination {
    Pagination {
        page: 1,
        per_page: 20,
        total_items: 156,
        total_pages: 8,
    }
}

// =============================================================================
// Sample data creators - Users
// =============================================================================

fn create_sample_user() -> User {
    User {
        id: "user-001".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        age: Some(30),
        status: UserStatus::Active,
        gender: Gender::Male,
        active: true,
        home_address: Some(create_sample_address()),
        created_at: "2024-01-15T10:30:00Z".to_string(),
        info: Some(serde_json::json!({
            "hobbies": ["reading", "coding"],
            "score": 95.5
        })),
    }
}

fn create_sample_user_minimal() -> User {
    User {
        id: "user-002".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        email: "jane.smith@example.com".to_string(),
        age: None,
        status: UserStatus::Pending,
        gender: Gender::Female,
        active: false,
        home_address: None,
        created_at: "2024-02-20T14:00:00Z".to_string(),
        info: None,
    }
}

fn create_sample_user_event_created() -> UserEvent {
    UserEvent::Created(create_sample_user())
}

fn create_sample_user_event_status_changed() -> UserEvent {
    UserEvent::StatusChanged(UserStatusChange {
        user_id: "user-001".to_string(),
        old_status: UserStatus::Pending,
        new_status: UserStatus::Active,
        changed_at: "2024-01-16T08:00:00Z".to_string(),
    })
}

// =============================================================================
// Sample data creators - Orders
// =============================================================================

fn create_sample_order() -> Order {
    Order {
        id: "order-001".to_string(),
        user_id: "user-001".to_string(),
        user: Some(create_sample_user()),
        items: vec![
            OrderItem {
                product_id: "prod-001".to_string(),
                name: "Laptop".to_string(),
                quantity: 1,
                unit_price: "999.99".to_string(),
            },
            OrderItem {
                product_id: "prod-002".to_string(),
                name: "Mouse".to_string(),
                quantity: 2,
                unit_price: "29.99".to_string(),
            },
        ],
        total: "1059.97".to_string(),
        status: OrderStatus::Confirmed,
        shipping_address: create_sample_address(),
        billing_address: None,
        created_at: "2024-01-20T09:00:00Z".to_string(),
        tracking_number: Some("1Z999AA10123456784".to_string()),
    }
}

fn create_sample_order_event_created() -> OrderEvent {
    OrderEvent::Created(create_sample_order())
}

fn create_sample_order_event_cancelled() -> OrderEvent {
    OrderEvent::Cancelled(OrderCancellation {
        order_id: "order-001".to_string(),
        reason: "Customer requested cancellation".to_string(),
        refund_amount: Some("1059.97".to_string()),
        cancelled_at: "2024-01-21T15:30:00Z".to_string(),
    })
}

fn create_sample_order_status_change() -> OrderEvent {
    OrderEvent::StatusChanged(OrderStatusChange {
        order_id: "order-001".to_string(),
        old_status: OrderStatus::Pending,
        new_status: OrderStatus::Confirmed,
        changed_at: "2024-01-20T10:00:00Z".to_string(),
    })
}

// =============================================================================
// Sample data creators - Notifications
// =============================================================================

fn create_sample_message_plain() -> Message {
    Message::PlainText("Hello, this is a plain text message!".to_string())
}

fn create_sample_message_user_notification() -> Message {
    Message::UserNotification(UserNotification::new(
        "Welcome!".to_string(),
        "Thank you for signing up.".to_string(),
        "user-001".to_string(),
        Some("https://example.com/welcome".to_string()),
    ))
}

fn create_sample_message_order_notification() -> Message {
    Message::OrderNotification(OrderNotification::new(
        "Order Shipped!".to_string(),
        "Your order is on its way.".to_string(),
        "order-001".to_string(),
        Some("https://example.com/track/order-001".to_string()),
    ))
}

fn create_sample_message_system_alert() -> Message {
    Message::SystemAlert(SystemAlert::new(
        "Scheduled Maintenance".to_string(),
        "The system will be down for maintenance on Sunday.".to_string(),
        AlertSeverity::Warning,
        Some("2024-01-28T00:00:00Z".to_string()),
    ))
}

fn create_sample_queued_notification() -> QueuedNotification {
    QueuedNotification {
        id: "notif-001".to_string(),
        message: Message::UserNotification(UserNotification::new(
            "Welcome!".to_string(),
            "Thank you for signing up.".to_string(),
            "user-001".to_string(),
            None,
        )),
        recipient_id: "user-001".to_string(),
        status: DeliveryStatus::Delivered,
        created_at: "2024-01-15T10:31:00Z".to_string(),
        sent_at: Some("2024-01-15T10:31:05Z".to_string()),
        delivered_at: Some("2024-01-15T10:31:10Z".to_string()),
        read_at: None,
    }
}

// =============================================================================
// Write/Read operations
// =============================================================================

fn write_sample_data(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    // Helper to write JSON
    fn write_json<T: serde::Serialize>(
        path: &Path,
        filename: &str,
        data: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(data)?;
        let mut file = fs::File::create(path.join(filename))?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    // Common types
    write_json(path, "address.json", &create_sample_address())?;
    write_json(
        path,
        "api_response_success.json",
        &create_sample_api_response_success(),
    )?;
    write_json(
        path,
        "api_response_error.json",
        &create_sample_api_response_error(),
    )?;
    write_json(path, "pagination.json", &create_sample_pagination())?;

    // User types
    write_json(path, "user.json", &create_sample_user())?;
    write_json(path, "user_minimal.json", &create_sample_user_minimal())?;
    write_json(
        path,
        "user_event_created.json",
        &create_sample_user_event_created(),
    )?;
    write_json(
        path,
        "user_event_status_changed.json",
        &create_sample_user_event_status_changed(),
    )?;

    // Order types
    write_json(path, "order.json", &create_sample_order())?;
    write_json(
        path,
        "order_event_created.json",
        &create_sample_order_event_created(),
    )?;
    write_json(
        path,
        "order_event_cancelled.json",
        &create_sample_order_event_cancelled(),
    )?;
    write_json(
        path,
        "order_event_status_changed.json",
        &create_sample_order_status_change(),
    )?;

    // Notification types
    write_json(path, "message_plain.json", &create_sample_message_plain())?;
    write_json(
        path,
        "message_user_notification.json",
        &create_sample_message_user_notification(),
    )?;
    write_json(
        path,
        "message_order_notification.json",
        &create_sample_message_order_notification(),
    )?;
    write_json(
        path,
        "message_system_alert.json",
        &create_sample_message_system_alert(),
    )?;
    write_json(
        path,
        "queued_notification.json",
        &create_sample_queued_notification(),
    )?;

    println!("Sample data written to {}", output_dir);
    Ok(())
}

fn read_and_validate(input_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(input_dir);

    let files: &[(&str, &str)] = &[
        ("address.json", "Address"),
        ("api_response_success.json", "ApiResponse"),
        ("api_response_error.json", "ApiResponse"),
        ("pagination.json", "Pagination"),
        ("user.json", "User"),
        ("user_minimal.json", "User"),
        ("user_event_created.json", "UserEvent"),
        ("user_event_status_changed.json", "UserEvent"),
        ("order.json", "Order"),
        ("order_event_created.json", "OrderEvent"),
        ("order_event_cancelled.json", "OrderEvent"),
        ("order_event_status_changed.json", "OrderEvent"),
        ("message_plain.json", "Message"),
        ("message_user_notification.json", "Message"),
        ("message_order_notification.json", "Message"),
        ("message_system_alert.json", "Message"),
        ("queued_notification.json", "QueuedNotification"),
    ];

    for (filename, type_name) in files.iter() {
        let file_path = path.join(filename);
        if !file_path.exists() {
            eprintln!("File not found: {}", file_path.display());
            continue;
        }

        let content = fs::read_to_string(&file_path)?;

        match *type_name {
            "Address" => {
                let addr: Address = serde_json::from_str(&content)?;
                println!(
                    "Validated Address: {}, {}, {}",
                    addr.city, addr.state, addr.country
                );
            }
            "ApiResponse" => {
                let resp: ApiResponse = serde_json::from_str(&content)?;
                println!(
                    "Validated ApiResponse: success={}, request_id={}",
                    resp.success, resp.request_id
                );
            }
            "Pagination" => {
                let page: Pagination = serde_json::from_str(&content)?;
                println!(
                    "Validated Pagination: page {}/{}",
                    page.page, page.total_pages
                );
            }
            "User" => {
                let user: User = serde_json::from_str(&content)?;
                println!(
                    "Validated User: {} {} ({})",
                    user.first_name, user.last_name, user.email
                );
            }
            "UserEvent" => {
                let event: UserEvent = serde_json::from_str(&content)?;
                match event {
                    UserEvent::Created(ref user) => {
                        println!("Validated UserEvent::Created for: {}", user.email);
                    }
                    UserEvent::Updated(ref user) => {
                        println!("Validated UserEvent::Updated for: {}", user.email);
                    }
                    UserEvent::Deleted => {
                        println!("Validated UserEvent::Deleted");
                    }
                    UserEvent::StatusChanged(ref change) => {
                        println!(
                            "Validated UserEvent::StatusChanged: {:?} -> {:?}",
                            change.old_status, change.new_status
                        );
                    }
                }
            }
            "Order" => {
                let order: Order = serde_json::from_str(&content)?;
                println!(
                    "Validated Order: {} ({} items, total: {})",
                    order.id,
                    order.items.len(),
                    order.total
                );
            }
            "OrderEvent" => {
                let event: OrderEvent = serde_json::from_str(&content)?;
                match event {
                    OrderEvent::Created(ref order) => {
                        println!("Validated OrderEvent::Created for: {}", order.id);
                    }
                    OrderEvent::Updated(ref order) => {
                        println!("Validated OrderEvent::Updated for: {}", order.id);
                    }
                    OrderEvent::StatusChanged(ref change) => {
                        println!(
                            "Validated OrderEvent::StatusChanged: {:?} -> {:?}",
                            change.old_status, change.new_status
                        );
                    }
                    OrderEvent::Cancelled(ref cancel) => {
                        println!(
                            "Validated OrderEvent::Cancelled for: {} - {}",
                            cancel.order_id, cancel.reason
                        );
                    }
                }
            }
            "Message" => {
                let msg: Message = serde_json::from_str(&content)?;
                match msg {
                    Message::PlainText(ref text) => {
                        println!("Validated Message::PlainText: {}", text);
                    }
                    Message::UserNotification(ref notif) => {
                        println!("Validated Message::UserNotification: {}", notif.title);
                    }
                    Message::OrderNotification(ref notif) => {
                        println!("Validated Message::OrderNotification: {}", notif.title);
                    }
                    Message::SystemAlert(ref alert) => {
                        println!(
                            "Validated Message::SystemAlert: {} ({:?})",
                            alert.title, alert.severity
                        );
                    }
                }
            }
            "QueuedNotification" => {
                let notif: QueuedNotification = serde_json::from_str(&content)?;
                println!(
                    "Validated QueuedNotification: {} (status: {:?})",
                    notif.id, notif.status
                );
            }
            _ => {}
        }
    }

    Ok(())
}

fn run_default_demo() {
    println!("=== Fluorite Multi-Package Demo ===\n");

    // Demo cross-package types
    println!("--- Common Types ---");
    let addr = create_sample_address();
    println!("Address: {}, {}, {}\n", addr.city, addr.state, addr.country);

    // Demo User with Address from common
    println!("--- User Package (imports common.Address) ---");
    let user = create_sample_user();
    println!("User: {} {}", user.first_name, user.last_name);
    if let Some(ref home) = user.home_address {
        println!("  Home: {}, {}", home.city, home.country);
    }
    println!();

    // Demo Order with User and Address
    println!("--- Order Package (imports common.Address, users.User) ---");
    let order = create_sample_order();
    println!(
        "Order: {} - {} items, total: {}",
        order.id,
        order.items.len(),
        order.total
    );
    if let Some(ref user) = order.user {
        println!("  Placed by: {} {}", user.first_name, user.last_name);
    }
    println!(
        "  Ship to: {}, {}",
        order.shipping_address.city, order.shipping_address.country
    );
    println!();

    // Demo Notifications with extern union
    println!("--- Notification Package (imports users.User, orders.Order) ---");
    let msg = create_sample_message_system_alert();
    if let Message::SystemAlert(ref alert) = msg {
        println!(
            "System Alert: {} (severity: {:?})",
            alert.title, alert.severity
        );
    }
    println!();

    // Demo JSON serialization
    println!("=== JSON Serialization Examples ===\n");

    println!("User JSON:");
    if let Ok(json) = serde_json::to_string_pretty(&user) {
        println!("{}\n", json);
    }

    println!("UserEvent::StatusChanged JSON:");
    let event = create_sample_user_event_status_changed();
    if let Ok(json) = serde_json::to_string_pretty(&event) {
        println!("{}\n", json);
    }

    println!("Message::SystemAlert JSON:");
    if let Ok(json) = serde_json::to_string_pretty(&msg) {
        println!("{}\n", json);
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Write { output }) => {
            if let Err(e) = write_sample_data(&output) {
                eprintln!("Error writing sample data: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Read { input }) => {
            if let Err(e) = read_and_validate(&input) {
                eprintln!("Error reading sample data: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            run_default_demo();
        }
    }
}
