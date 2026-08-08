// Demo for Fluorite Go code generation. Mirrors examples/demo-ts/src/index.ts:
// writes sample JSON for other languages to read, and reads theirs back.
//
// Every .fl package generates into one flat Go package, so the four demo
// packages — common, users, orders, notifications — are all reachable here
// through a single import.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	g "fluorite-demo-go/generated"
)

func ptr[T any](v T) *T { return &v }

// --- Common ---

func sampleAddress() g.Address {
	return g.Address{
		Street1:    "123 Main St",
		Street2:    ptr("Apt 4B"),
		City:       "New York",
		State:      "NY",
		PostalCode: "10001",
		Country:    "US",
	}
}

func sampleApiResponseSuccess() g.ApiResponse {
	return g.ApiResponse{
		Success:   true,
		Data:      ptr(any(map[string]any{"users": 42, "orders": 15})),
		RequestID: "req-12345",
	}
}

func sampleApiResponseError() g.ApiResponse {
	return g.ApiResponse{
		Success:      false,
		ErrorMessage: ptr("User not found"),
		ErrorCode:    ptr("USER_NOT_FOUND"),
		RequestID:    "req-12346",
	}
}

func samplePagination() g.Pagination {
	return g.Pagination{
		Page:       1,
		PerPage:    20,
		TotalItems: 156,
		TotalPages: 8,
	}
}

// --- Users ---

func sampleUser() g.User {
	return g.User{
		ID:          "user-001",
		FirstName:   "John",
		LastName:    "Doe",
		Email:       "john.doe@example.com",
		Age:         ptr(uint32(30)),
		Status:      g.UserStatusActive,
		Gender:      g.GenderMale,
		Active:      true,
		HomeAddress: ptr(sampleAddress()),
		CreatedAt:   "2024-01-15T10:30:00Z",
		Info: ptr(any(map[string]any{
			"hobbies": []any{"reading", "coding"},
			"score":   95.5,
		})),
	}
}

func sampleUserMinimal() g.User {
	return g.User{
		ID:        "user-002",
		FirstName: "Jane",
		LastName:  "Smith",
		Email:     "jane.smith@example.com",
		Status:    g.UserStatusPending,
		Gender:    g.GenderFemale,
		Active:    false,
		CreatedAt: "2024-02-20T14:00:00Z",
	}
}

func sampleUserEventCreated() g.UserEvent {
	return g.UserEvent{Variant: g.UserEventCreated{Value: sampleUser()}}
}

func sampleUserEventStatusChanged() g.UserEvent {
	return g.UserEvent{Variant: g.UserEventStatusChanged{Value: g.UserStatusChange{
		UserID:    "user-001",
		OldStatus: g.UserStatusPending,
		NewStatus: g.UserStatusActive,
		ChangedAt: "2024-01-16T08:00:00Z",
	}}}
}

// --- Orders ---

func sampleOrder() g.Order {
	return g.Order{
		ID:     "order-001",
		UserID: "user-001",
		User:   ptr(sampleUser()),
		Items: []g.OrderItem{
			{ProductID: "prod-001", Name: "Laptop", Quantity: 1, UnitPrice: "999.99"},
			{ProductID: "prod-002", Name: "Mouse", Quantity: 2, UnitPrice: "29.99"},
		},
		Total:           "1059.97",
		Status:          g.OrderStatusConfirmed,
		ShippingAddress: sampleAddress(),
		CreatedAt:       "2024-01-20T09:00:00Z",
		TrackingNumber:  ptr("1Z999AA10123456784"),
	}
}

func sampleOrderEventCreated() g.OrderEvent {
	return g.OrderEvent{Variant: g.OrderEventCreated{Value: sampleOrder()}}
}

func sampleOrderEventCancelled() g.OrderEvent {
	return g.OrderEvent{Variant: g.OrderEventCancelled{Value: g.OrderCancellation{
		OrderID:      "order-001",
		Reason:       "Customer requested cancellation",
		RefundAmount: ptr("1059.97"),
		CancelledAt:  "2024-01-21T15:30:00Z",
	}}}
}

func sampleOrderEventStatusChanged() g.OrderEvent {
	return g.OrderEvent{Variant: g.OrderEventStatusChanged{Value: g.OrderStatusChange{
		OrderID:   "order-001",
		OldStatus: g.OrderStatusPending,
		NewStatus: g.OrderStatusConfirmed,
		ChangedAt: "2024-01-20T10:00:00Z",
	}}}
}

// --- Notifications ---

func sampleMessagePlainText() g.Message {
	return g.Message{Variant: g.MessagePlainText{Value: "Hello, this is a plain text message!"}}
}

func sampleMessageUserNotification() g.Message {
	return g.Message{Variant: g.MessageUserNotification{Value: g.UserNotification{
		Title:     "Welcome!",
		Body:      "Thank you for signing up.",
		UserID:    "user-001",
		ActionURL: ptr("https://example.com/welcome"),
	}}}
}

func sampleMessageOrderNotification() g.Message {
	return g.Message{Variant: g.MessageOrderNotification{Value: g.OrderNotification{
		Title:     "Order Shipped!",
		Body:      "Your order is on its way.",
		OrderID:   "order-001",
		ActionURL: ptr("https://example.com/track/order-001"),
	}}}
}

func sampleMessageSystemAlert() g.Message {
	return g.Message{Variant: g.MessageSystemAlert{Value: g.SystemAlert{
		Title:     "Scheduled Maintenance",
		Body:      "The system will be down for maintenance on Sunday.",
		Severity:  g.AlertSeverityWarning,
		ExpiresAt: ptr("2024-01-28T00:00:00Z"),
	}}}
}

func sampleQueuedNotification() g.QueuedNotification {
	return g.QueuedNotification{
		ID:          "notif-001",
		Message:     sampleMessageUserNotification(),
		RecipientID: "user-001",
		Status:      g.DeliveryStatusDelivered,
		CreatedAt:   "2024-01-15T10:31:00Z",
		SentAt:      ptr("2024-01-15T10:31:05Z"),
		DeliveredAt: ptr("2024-01-15T10:31:10Z"),
	}
}

// --- Fixture I/O ---

func writeJSON(dir, name string, v any) error {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return fmt.Errorf("%s: %w", name, err)
	}
	return os.WriteFile(filepath.Join(dir, name), data, 0o644)
}

func writeAll(dir string) error {
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	files := []struct {
		name  string
		value any
	}{
		{"address.json", sampleAddress()},
		{"api_response_success.json", sampleApiResponseSuccess()},
		{"api_response_error.json", sampleApiResponseError()},
		{"pagination.json", samplePagination()},
		{"user.json", sampleUser()},
		{"user_minimal.json", sampleUserMinimal()},
		{"user_event_created.json", sampleUserEventCreated()},
		{"user_event_status_changed.json", sampleUserEventStatusChanged()},
		{"order.json", sampleOrder()},
		{"order_event_created.json", sampleOrderEventCreated()},
		{"order_event_cancelled.json", sampleOrderEventCancelled()},
		{"order_event_status_changed.json", sampleOrderEventStatusChanged()},
		{"message_plain.json", sampleMessagePlainText()},
		{"message_user_notification.json", sampleMessageUserNotification()},
		{"message_order_notification.json", sampleMessageOrderNotification()},
		{"message_system_alert.json", sampleMessageSystemAlert()},
		{"queued_notification.json", sampleQueuedNotification()},
	}
	for _, f := range files {
		if err := writeJSON(dir, f.name, f.value); err != nil {
			return err
		}
	}
	fmt.Printf("Sample data written to %s\n", dir)
	return nil
}

// readAll decodes every fixture into its declared type. A decode failure is a
// wire incompatibility, which is the whole point of the interop suite.
func readAll(dir string) error {
	cases := []struct {
		name   string
		target func() any
	}{
		{"address.json", func() any { return new(g.Address) }},
		{"api_response_success.json", func() any { return new(g.ApiResponse) }},
		{"api_response_error.json", func() any { return new(g.ApiResponse) }},
		{"pagination.json", func() any { return new(g.Pagination) }},
		{"user.json", func() any { return new(g.User) }},
		{"user_minimal.json", func() any { return new(g.User) }},
		{"user_event_created.json", func() any { return new(g.UserEvent) }},
		{"user_event_status_changed.json", func() any { return new(g.UserEvent) }},
		{"order.json", func() any { return new(g.Order) }},
		{"order_event_created.json", func() any { return new(g.OrderEvent) }},
		{"order_event_cancelled.json", func() any { return new(g.OrderEvent) }},
		{"order_event_status_changed.json", func() any { return new(g.OrderEvent) }},
		{"message_plain.json", func() any { return new(g.Message) }},
		{"message_user_notification.json", func() any { return new(g.Message) }},
		{"message_order_notification.json", func() any { return new(g.Message) }},
		{"message_system_alert.json", func() any { return new(g.Message) }},
		{"queued_notification.json", func() any { return new(g.QueuedNotification) }},
	}
	for _, c := range cases {
		data, err := os.ReadFile(filepath.Join(dir, c.name))
		if err != nil {
			return fmt.Errorf("%s: %w", c.name, err)
		}
		target := c.target()
		if err := json.Unmarshal(data, target); err != nil {
			return fmt.Errorf("%s: %w", c.name, err)
		}
		fmt.Printf("  ok %s\n", c.name)
	}
	return nil
}

func main() {
	args := os.Args[1:]
	if len(args) != 2 {
		fmt.Fprintln(os.Stderr, "usage: demo-go (--write|--read) <dir>")
		os.Exit(2)
	}
	var err error
	switch args[0] {
	case "--write":
		err = writeAll(args[1])
	case "--read":
		err = readAll(args[1])
	default:
		fmt.Fprintf(os.Stderr, "unknown flag %q\n", args[0])
		os.Exit(2)
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
