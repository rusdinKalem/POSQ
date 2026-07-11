# BILLING AND RENEWAL

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan alur billing, renewal, subscription state, payment confirmation, dan hubungan dengan license token.

## 1. Core Principle

Billing adalah sumber status subscription di control plane. Desktop POS tidak memproses keputusan billing secara lokal. Desktop hanya:

- Menampilkan status subscription.
- Membuka renewal/payment screen.
- Meminta refresh license setelah renewal berhasil.
- Mengikuti signed license token terbaru dari server.

Checkout lokal tidak boleh memanggil billing server saat transaksi.

## 2. MVP Billing Strategy

MVP recommendation:

- Manual billing confirmation through admin dashboard.
- Payment can be handled outside app through bank transfer/payment link/manual invoice.
- Admin updates subscription status after payment confirmed.
- Desktop app calls license refresh or heartbeat to receive active token.

Beta recommendation:

- Add payment link provider.
- Add webhook from payment provider to update subscription.
- Keep manual override for support.

Production recommendation:

- Payment gateway integration.
- Invoice and receipt management.
- Auto-renewal if business model supports it.
- Dunning/reminder workflow.

## 3. Subscription States

| State | Meaning | Desktop Runtime |
|---|---|---|
| active | Paid and valid | active |
| grace | Payment overdue or issue, still allowed | grace |
| expired | Grace ended | restricted_expired |
| suspended | Admin/business suspension | restricted_expired or revoked depending policy |
| cancelled | Subscription ended | restricted_expired |

## 4. Billing Flow - Manual MVP

```text
Merchant subscription nearing expiry
  -> Desktop shows reminder
  -> Owner opens renewal screen
  -> Owner pays via manual channel/payment link
  -> Admin confirms payment in dashboard
  -> Control plane updates subscription.paid_until
  -> Control plane issues refreshed license token
  -> Desktop heartbeat/license refresh receives token
  -> App returns to active mode
```

## 5. Renewal Screen Requirements

Desktop renewal screen should show:

- Current plan.
- Current status.
- Paid until.
- Grace until.
- Device count usage.
- Renewal instructions.
- Payment reference if available.
- Refresh license button.
- Contact support button.

Restricted Expired Mode must keep renewal screen available.

## 6. Admin Billing Dashboard

Admin dashboard must support:

- View merchant subscription.
- Change plan.
- Extend paid_until.
- Set grace_until.
- Mark active/grace/expired/suspended.
- Add payment reference.
- Add manual override reason.
- View subscription event history.
- Issue license refresh.

Every billing mutation must create admin audit log.

## 7. Subscription Event Model

Events:

```text
subscription_created
plan_changed
payment_confirmed
paid_until_extended
grace_started
expired
suspended
cancelled
manual_override
license_refreshed
```

Each event should include:

- merchant_id
- subscription_id
- actor_type: system/admin/payment_provider
- actor_id
- old_status
- new_status
- old_plan
- new_plan
- paid_until
- reason
- metadata_json
- created_at

## 8. License Refresh After Payment

After payment confirmed:

1. Server updates subscription state.
2. Server increments license token version or marks refresh available.
3. Desktop calls `/licenses/refresh`.
4. Server returns signed active token.
5. Desktop verifies signature.
6. Desktop exits Restricted Expired Mode.

No reinstall required.

## 9. Grace and Reminder Policy

Recommended MVP:

- Reminder starts 7 days before paid_until.
- Grace period: 7 days after paid_until.
- Token lease: 7 days.
- UI warning during reminder and grace.

Reminder levels:

| Time | UI |
|---|---|
| paid_until - 7 days | subtle reminder |
| paid_until - 3 days | stronger banner |
| grace active | persistent warning |
| grace ended | Restricted Expired Mode |

## 10. Payment Provider Future

When adding payment provider:

- Add new ADR.
- Add webhook signature verification.
- Add idempotency for payment events.
- Add invoice table.
- Add failed payment handling.
- Add reconciliation dashboard.

Payment webhook must not directly unlock desktop without issuing signed license token.

## 11. Error Handling

| Scenario | Behavior |
|---|---|
| Payment made but license still expired | User clicks refresh license; if still failed, show support path |
| Admin sets wrong status | Admin audit log allows review; support can correct |
| Payment webhook duplicate | Idempotency prevents double extension |
| Server offline | Desktop remains in current local license mode |
| User paid during Restricted Expired Mode | Renewal remains accessible; refresh restores active token |

## 12. Acceptance Criteria

- Admin can manually extend subscription.
- Manual extension creates subscription event and audit log.
- Desktop can refresh license after payment.
- Renewal works from Restricted Expired Mode.
- Checkout is not coupled to billing server.
- Duplicate payment/renewal event does not double-extend subscription.
- Subscription expiry never deletes local merchant data.
