# DATA MODEL

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan model data lokal dan server control plane agar Antigravity tidak menebak skema utama saat implementasi.

## 1. Data Ownership

| Data Category | Source of Truth | Stored on Server by Default |
|---|---|---|
| Orders | Local PostgreSQL | No |
| Payments | Local PostgreSQL | No |
| Inventory movements | Local PostgreSQL | No |
| Local users/roles | Local PostgreSQL | No, except account owner credential in control plane |
| Reports | Computed from local DB | No |
| License/subscription | Control plane server | Yes |
| Device registration | Control plane server | Yes |
| Backup metadata | Control plane server | Yes |
| Backup payload | Local/object storage encrypted | No plaintext |
| App update metadata | Control plane server | Yes |

## 2. Naming Conventions

- Table names: plural snake_case.
- Primary key: `id uuid primary key`.
- Timestamps: `created_at`, `updated_at`.
- Soft delete where needed: `deleted_at`.
- Monetary values: integer minor unit in rupiah or decimal with explicit precision. MVP recommendation: integer rupiah.
- IDs should be UUID.
- Local generated business numbers should use outlet/device prefix.

Migration naming:

```text
YYYYMMDDHHMM_description.sql
```

Example:

```text
202607050001_create_core_pos_tables.sql
```

## 3. Local PostgreSQL Schema

Local DB stores operational POS data.

### 3.1 merchants

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| name | text | required |
| tax_number | text | optional |
| address | text | optional |
| phone | text | optional |
| created_at | timestamptz | required |
| updated_at | timestamptz | required |

### 3.2 outlets

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk merchants.id |
| name | text | required |
| code | text | required, unique per merchant |
| address | text | optional |
| timezone | text | default Asia/Jakarta |
| created_at | timestamptz | required |
| updated_at | timestamptz | required |

### 3.3 users

Local app users.

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | nullable |
| name | text | required |
| email | text | optional |
| pin_hash | text | optional |
| password_hash | text | optional |
| status | text | active/inactive |
| created_at | timestamptz | required |
| updated_at | timestamptz | required |

### 3.4 roles

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| name | text | owner, manager, cashier, inventory, finance, kitchen_waiter |
| system_role | boolean | default false |
| created_at | timestamptz | required |

### 3.5 permissions

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| key | text | e.g. checkout:create |
| description | text | optional |

### 3.6 role_permissions

| Column | Type | Notes |
|---|---|---|
| role_id | uuid | fk |
| permission_id | uuid | fk |

Unique:

```text
(role_id, permission_id)
```

### 3.7 user_roles

| Column | Type | Notes |
|---|---|---|
| user_id | uuid | fk |
| role_id | uuid | fk |

Unique:

```text
(user_id, role_id)
```

### 3.8 categories

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| name | text | required |
| sort_order | integer | default 0 |
| active | boolean | default true |

### 3.9 products

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| category_id | uuid | nullable |
| sku | text | required, unique per merchant |
| barcode | text | optional |
| name | text | required |
| price | integer | rupiah |
| cost | integer | optional |
| track_stock | boolean | default true |
| active | boolean | default true |
| created_at | timestamptz | required |
| updated_at | timestamptz | required |

Indexes:

```text
products(merchant_id, sku)
products(merchant_id, barcode)
products(merchant_id, name)
```

### 3.10 inventory_items

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | fk |
| product_id | uuid | fk products.id |
| qty_on_hand | numeric(18,3) | required |
| min_qty | numeric(18,3) | default 0 |
| updated_at | timestamptz | required |

Unique:

```text
(outlet_id, product_id)
```

### 3.11 stock_movements

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | fk |
| product_id | uuid | fk |
| movement_type | text | sale, refund, stock_in, adjustment, transfer_out, transfer_in, opname |
| qty_delta | numeric(18,3) | positive/negative |
| reason | text | required for adjustment |
| reference_type | text | order, refund, transfer, opname, manual |
| reference_id | uuid | nullable |
| created_by | uuid | user id |
| created_at | timestamptz | required |

Indexes:

```text
stock_movements(outlet_id, product_id, created_at)
stock_movements(reference_type, reference_id)
```

### 3.12 shifts

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | fk |
| opened_by | uuid | user id |
| closed_by | uuid | nullable |
| status | text | open/closed |
| starting_cash | integer | rupiah |
| expected_cash | integer | nullable |
| counted_cash | integer | nullable |
| opened_at | timestamptz | required |
| closed_at | timestamptz | nullable |

Constraint:

- Only one open shift per outlet/device policy for MVP.

### 3.13 orders

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | fk |
| shift_id | uuid | fk shifts.id |
| order_number | text | unique per outlet |
| status | text | draft, held, paid, voided, refunded |
| subtotal | integer | rupiah |
| discount_total | integer | rupiah |
| tax_total | integer | rupiah |
| service_total | integer | rupiah |
| grand_total | integer | rupiah |
| paid_total | integer | rupiah |
| change_total | integer | rupiah |
| created_by | uuid | user id |
| created_at | timestamptz | required |
| updated_at | timestamptz | required |

Indexes:

```text
orders(outlet_id, created_at)
orders(shift_id)
orders(order_number)
```

### 3.14 order_items

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| order_id | uuid | fk orders.id |
| product_id | uuid | nullable if custom item |
| sku | text | snapshot |
| name | text | snapshot |
| qty | numeric(18,3) | required |
| unit_price | integer | snapshot |
| discount_total | integer | rupiah |
| line_total | integer | rupiah |
| notes | text | optional |

### 3.15 payments

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | fk |
| order_id | uuid | fk orders.id |
| method | text | cash, qris_manual, debit, transfer, other |
| status | text | pending, paid, failed, refunded |
| amount | integer | rupiah |
| reference | text | optional |
| paid_at | timestamptz | nullable |
| created_at | timestamptz | required |

### 3.16 refunds

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| order_id | uuid | fk |
| amount | integer | rupiah |
| reason | text | required |
| approved_by | uuid | user id |
| created_by | uuid | user id |
| created_at | timestamptz | required |

### 3.17 audit_logs

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | nullable |
| actor_user_id | uuid | nullable |
| action | text | required |
| target_type | text | required |
| target_id | uuid | nullable |
| reason | text | optional |
| metadata_json | jsonb | redacted metadata |
| created_at | timestamptz | required |

Rules:

- Append-only from app perspective.
- Sensitive actions must always create audit log.

### 3.18 job_outbox

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| device_id | text | required |
| job_type | text | heartbeat, license_refresh, backup_metadata, update_check |
| payload_json | jsonb | required |
| idempotency_key | text | required |
| status | text | pending, sending, sent, failed, rejected, dead_letter |
| retry_count | integer | default 0 |
| last_error | text | nullable |
| created_at | timestamptz | required |
| sent_at | timestamptz | nullable |

Unique:

```text
(job_type, idempotency_key)
```

### 3.19 backup_jobs

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| device_id | text | required |
| destination_type | text | local, managed_cloud, byos_s3 |
| status | text | pending, running, uploaded, failed, retained, deleted |
| file_path | text | local path or logical ref |
| size_bytes | bigint | nullable |
| checksum_sha256 | text | nullable |
| encrypted | boolean | required |
| created_at | timestamptz | required |
| completed_at | timestamptz | nullable |

### 3.20 backup_metadata

Local cache of metadata sent to control plane.

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| backup_job_id | uuid | fk |
| server_recorded | boolean | default false |
| server_recorded_at | timestamptz | nullable |
| restore_compatibility_version | text | required |
| manifest_json | jsonb | required |

### 3.21 device_licenses

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| device_id | text | required |
| merchant_id | uuid | required |
| outlet_id | uuid | required |
| license_token | text | signed token |
| token_version | integer | required |
| license_status | text | active, grace, restricted_expired, revoked, suspicious_time |
| valid_until | timestamptz | required |
| grace_until | timestamptz | nullable |
| last_server_time | timestamptz | nullable |
| last_seen_local_time | timestamptz | nullable |
| tamper_seal | text | required |
| updated_at | timestamptz | required |

## 4. Server Control Plane Schema

Server DB stores SaaS control plane only.

### 4.1 merchants

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| name | text | required |
| status | text | active, suspended, closed |
| created_at | timestamptz | required |
| updated_at | timestamptz | required |

### 4.2 merchant_users

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| email | text | unique |
| password_hash | text | required |
| role | text | owner, admin, billing, support |
| mfa_enabled | boolean | default false |
| status | text | active, disabled |
| created_at | timestamptz | required |

### 4.3 devices

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| outlet_id | uuid | nullable |
| device_fingerprint | text | required |
| device_name | text | required |
| status | text | active, revoked |
| app_version | text | optional |
| last_heartbeat_at | timestamptz | nullable |
| created_at | timestamptz | required |
| revoked_at | timestamptz | nullable |
| revoked_reason | text | nullable |

Unique:

```text
(merchant_id, device_fingerprint)
```

### 4.4 subscriptions

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| plan | text | starter, growth, pro, business, enterprise |
| status | text | active, grace, expired, suspended |
| paid_until | timestamptz | required |
| grace_until | timestamptz | nullable |
| billing_reference | text | optional |
| created_at | timestamptz | required |
| updated_at | timestamptz | required |

### 4.5 entitlements

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| plan | text | required |
| key | text | e.g. managed_cloud_backup |
| value_json | jsonb | required |

### 4.6 device_licenses

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| merchant_id | uuid | fk |
| device_id | uuid | fk devices.id |
| token_version | integer | required |
| license_status | text | active, grace, restricted_expired, revoked |
| issued_at | timestamptz | required |
| valid_until | timestamptz | required |
| grace_until | timestamptz | nullable |
| public_key_id | text | signing key id |
| created_at | timestamptz | required |

### 4.7 backup_metadata

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| backup_id | uuid | unique |
| merchant_id | uuid | fk |
| device_id | uuid | fk devices.id |
| destination_type | text | local, managed_cloud, byos_s3 |
| storage_ref | text | logical ref only |
| size_bytes | bigint | required |
| checksum_sha256 | text | required |
| encrypted | boolean | required |
| encryption_alg | text | required |
| app_version | text | required |
| db_schema_version | text | required |
| restore_compatibility_version | text | required |
| status | text | available, failed, deleted |
| created_at | timestamptz | required |

Rules:

- No plaintext backup content.
- No full DB payload.

### 4.8 app_versions

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| version | text | required |
| os | text | windows, macos, linux |
| channel | text | stable, beta |
| critical | boolean | default false |
| minimum_supported_version | text | required |
| download_url | text | required |
| signature | text | required |
| sha256 | text | required |
| release_notes | text | optional |
| created_at | timestamptz | required |

### 4.9 admin_audit_logs

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| actor_admin_id | uuid | required |
| action | text | required |
| target_type | text | required |
| target_id | uuid | nullable |
| merchant_id | uuid | nullable |
| reason | text | optional |
| metadata_json | jsonb | redacted |
| created_at | timestamptz | required |

### 4.10 idempotency_keys

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| key | text | required |
| scope | text | endpoint/merchant/device scope |
| request_hash | text | required |
| response_hash | text | required |
| response_json | jsonb | optional |
| created_at | timestamptz | required |
| expires_at | timestamptz | required |

Unique:

```text
(key, scope)
```

## 5. Enums

Recommended enum values:

```text
order_status: draft, held, paid, voided, refunded
payment_status: pending, paid, failed, refunded
shift_status: open, closed
movement_type: sale, refund, stock_in, adjustment, transfer_out, transfer_in, opname
job_status: pending, sending, sent, failed, rejected, dead_letter
backup_status: pending, running, uploaded, failed, retained, deleted, available
license_status: active, grace, restricted_expired, revoked, suspicious_time
subscription_status: active, grace, expired, suspended
runtime_mode: active, grace, restricted_expired, revoked, suspicious_time
```

## 6. Transaction Rules

Checkout transaction must write atomically:

1. orders
2. order_items
3. payments
4. stock_movements
5. inventory_items qty update
6. audit_logs

If any write fails, all must rollback.

Stock adjustment must write atomically:

1. inventory_items update
2. stock_movements
3. audit_logs

Backup metadata local transaction:

1. backup_jobs update
2. backup_metadata insert/update
3. job_outbox insert for server metadata upload

## 7. Server Schema Guardrail

Default server migrations must not create:

```text
orders
order_items
payments
stock_movements
inventory_items with live operational stock
customers with purchase history
```

If a future enterprise module needs cloud operational sync, create a new ADR and separate schema namespace.

## 8. Acceptance Criteria

- Local migration creates all MVP local tables.
- Server migration creates only control plane tables.
- Checkout writes order/payment/stock/audit atomically.
- Server schema test fails if operational transaction tables are added by default.
- License state supports `restricted_expired`.
- Backup metadata exists both locally and on server without plaintext backup content.
- Audit logs exist for checkout, refund, void, stock adjustment, role change, license activation, migration, restore.
