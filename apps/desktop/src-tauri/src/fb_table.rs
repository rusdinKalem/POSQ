use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tauri::State;
use chrono::Utc;

// --- DOMAIN MODELS ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableStatus {
    pub id: String,
    pub name: String,
    pub status: String, // "available", "reserved", "occupied", "dirty", "disabled"
    pub version: i32,
    pub active_session: Option<ActiveSessionInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActiveSessionInfo {
    pub session_id: String,
    pub session_status: String,
    pub tables: Vec<String>, // List of joined table names
    pub bills: Vec<BillSummary>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillSummary {
    pub id: String,
    pub bill_number: String,
    pub status: String, // "open", "merged", "paid", "closed", "void", "refunded"
    pub subtotal: i32,
    pub discount_total: i32,
    pub tax_total: i32,
    pub service_total: i32,
    pub rounding_total: i32,
    pub grand_total: i32,
    pub paid_total: i32,
    pub balance_amount: i32,
    pub version: i32,
    pub reference_id: Option<String>,
    pub reference_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillWithItems {
    pub id: String,
    pub dining_session_id: String,
    pub status: String,
    pub bill_number: String,
    pub subtotal: i32,
    pub discount_total: i32,
    pub tax_total: i32,
    pub service_total: i32,
    pub rounding_total: i32,
    pub grand_total: i32,
    pub paid_total: i32,
    pub balance_amount: i32,
    pub version: i32,
    pub reference_id: Option<String>,
    pub reference_type: Option<String>,
    pub items: Vec<BillItemInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillItemInfo {
    pub id: String,
    pub product_id: Option<String>,
    pub product_name_snapshot: String,
    pub order_item_id: Option<String>,
    pub parent_bill_item_id: Option<String>,
    pub quantity: f64,
    pub unit_price: i32,
    pub gross_amount: i32,
    pub item_discount_amount: i32,
    pub taxable_amount: i32,
    pub tax_amount: i32,
    pub service_charge_amount: i32,
    pub net_amount: i32,
}

// --- PAYLOADS ---

#[derive(Deserialize, Debug, Clone)]
pub struct SplitBillItemPayload {
    pub id: String, // original bill_item_id
    pub quantity_to_move: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SplitTargetPayload {
    pub target_bill_index: usize, // 0-based index of target bill
    pub items: Vec<SplitBillItemPayload>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SplitBillRequest {
    pub bill_id: String,
    pub num_splits: usize,
    pub strategy: String, // "item", "qty", "equally"
    pub targets: Vec<SplitTargetPayload>,
    pub idempotency_key: String,
    pub authorized_by: Option<String>,
    pub reason: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SplitBillPreviewResponse {
    pub original_bill_id: String,
    pub operation_token: String,
    pub bills: Vec<BillWithItems>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SplitBillCommitResponse {
    pub success: bool,
    pub transaction_id: String,
    pub bills: Vec<BillSummary>,
}

// --- TAURI COMMANDS ---

#[tauri::command]
pub async fn get_all_tables_status(pool: State<'_, SqlitePool>) -> Result<Vec<TableStatus>, String> {
    let tables_records = sqlx::query(
        "SELECT id, name, status, version FROM tables ORDER BY name ASC"
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for r in tables_records {
        let table_id: String = r.get("id");
        let table_name: String = r.get("name");
        let table_status: String = r.get("status");
        let table_version: i32 = r.get("version");

        // Find active session for this table
        let session_record = sqlx::query(
            r#"
            SELECT ds.id, ds.status, ds.version
            FROM dining_sessions ds
            JOIN session_tables st ON ds.id = st.dining_session_id
            WHERE st.table_id = ? AND ds.status = 'active'
            LIMIT 1
            "#
        )
        .bind(&table_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

        let active_session = if let Some(sr) = session_record {
            let session_id: String = sr.get("id");
            let session_status: String = sr.get("status");

            // Get all tables in this session (joined tables)
            let joined_tables_records = sqlx::query(
                r#"
                SELECT t.name
                FROM session_tables st
                JOIN tables t ON st.table_id = t.id
                WHERE st.dining_session_id = ?
                "#
            )
            .bind(&session_id)
            .fetch_all(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

            let tables: Vec<String> = joined_tables_records.into_iter().map(|tr| tr.get::<String, _>("name")).collect();

            // Get bills for this session
            let bills_records = sqlx::query(
                r#"
                SELECT id, bill_number, status, subtotal, discount_total, tax_total, service_total, rounding_total, grand_total, paid_total, balance_amount, version, reference_id, reference_type
                FROM bills
                WHERE dining_session_id = ? AND status IN ('open', 'paid')
                "#
            )
            .bind(&session_id)
            .fetch_all(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

            let bills = bills_records.into_iter().map(|br| {
                BillSummary {
                    id: br.get("id"),
                    bill_number: br.get("bill_number"),
                    status: br.get("status"),
                    subtotal: br.get("subtotal"),
                    discount_total: br.get("discount_total"),
                    tax_total: br.get("tax_total"),
                    service_total: br.get("service_total"),
                    rounding_total: br.get("rounding_total"),
                    grand_total: br.get("grand_total"),
                    paid_total: br.get("paid_total"),
                    balance_amount: br.get("balance_amount"),
                    version: br.get("version"),
                    reference_id: br.get("reference_id"),
                    reference_type: br.get("reference_type"),
                }
            }).collect();

            Some(ActiveSessionInfo {
                session_id,
                session_status,
                tables,
                bills,
            })
        } else {
            None
        };

        result.push(TableStatus {
            id: table_id,
            name: table_name,
            status: table_status,
            version: table_version,
            active_session,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_bills_by_session(session_id: String, pool: State<'_, SqlitePool>) -> Result<Vec<BillWithItems>, String> {
    let bills_records = sqlx::query(
        r#"
        SELECT id, dining_session_id, status, bill_number, subtotal, discount_total, tax_total, service_total, rounding_total, grand_total, paid_total, balance_amount, version, reference_id, reference_type
        FROM bills
        WHERE dining_session_id = ? AND status IN ('open', 'paid')
        "#
    )
    .bind(&session_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut bills = Vec::new();

    for br in bills_records {
        let bill_id: String = br.get("id");

        let items_records = sqlx::query(
            r#"
            SELECT id, product_id, product_name_snapshot, order_item_id, parent_bill_item_id, quantity, unit_price, gross_amount, item_discount_amount, taxable_amount, tax_amount, service_charge_amount, net_amount
            FROM bill_items
            WHERE bill_id = ?
            "#
        )
        .bind(&bill_id)
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

        let items = items_records.into_iter().map(|ir| {
            BillItemInfo {
                id: ir.get("id"),
                product_id: ir.get("product_id"),
                product_name_snapshot: ir.get("product_name_snapshot"),
                order_item_id: ir.get("order_item_id"),
                parent_bill_item_id: ir.get("parent_bill_item_id"),
                quantity: ir.get("quantity"),
                unit_price: ir.get("unit_price"),
                gross_amount: ir.get("gross_amount"),
                item_discount_amount: ir.get("item_discount_amount"),
                taxable_amount: ir.get("taxable_amount"),
                tax_amount: ir.get("tax_amount"),
                service_charge_amount: ir.get("service_charge_amount"),
                net_amount: ir.get("net_amount"),
            }
        }).collect();

        bills.push(BillWithItems {
            id: bill_id,
            dining_session_id: br.get("dining_session_id"),
            status: br.get("status"),
            bill_number: br.get("bill_number"),
            subtotal: br.get("subtotal"),
            discount_total: br.get("discount_total"),
            tax_total: br.get("tax_total"),
            service_total: br.get("service_total"),
            rounding_total: br.get("rounding_total"),
            grand_total: br.get("grand_total"),
            paid_total: br.get("paid_total"),
            balance_amount: br.get("balance_amount"),
            version: br.get("version"),
            reference_id: br.get("reference_id"),
            reference_type: br.get("reference_type"),
            items,
        });
    }

    Ok(bills)
}


#[tauri::command]
pub async fn preview_split_bill(
    payload: SplitBillRequest,
    pool: State<'_, SqlitePool>,
) -> Result<SplitBillPreviewResponse, String> {
    preview_split_bill_inner(payload, pool.inner()).await
}

pub async fn preview_split_bill_inner(
    payload: SplitBillRequest,
    pool: &SqlitePool,
) -> Result<SplitBillPreviewResponse, String> {
    // 1. Fetch original bill & items
    let bill_record = sqlx::query(
        "SELECT id, dining_session_id, bill_number, subtotal, discount_total, tax_total, service_total, rounding_total, grand_total, paid_total, balance_amount, version FROM bills WHERE id = ?"
    )
    .bind(&payload.bill_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Bill not found".to_string());

    let original_bill = bill_record?;
    let orig_subtotal: i32 = original_bill.get("subtotal");
    let orig_discount: i32 = original_bill.get("discount_total");
    let orig_tax: i32 = original_bill.get("tax_total");
    let orig_service: i32 = original_bill.get("service_total");
    let orig_rounding: i32 = original_bill.get("rounding_total");
    let orig_grand: i32 = original_bill.get("grand_total");

    let items_records = sqlx::query(
        "SELECT id, product_id, product_name_snapshot, order_item_id, parent_bill_item_id, quantity, unit_price FROM bill_items WHERE bill_id = ?"
    )
    .bind(&payload.bill_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut original_items = Vec::new();
    for ir in items_records {
        original_items.push(BillItemInfo {
            id: ir.get("id"),
            product_id: ir.get("product_id"),
            product_name_snapshot: ir.get("product_name_snapshot"),
            order_item_id: ir.get("order_item_id"),
            parent_bill_item_id: ir.get("parent_bill_item_id"),
            quantity: ir.get("quantity"),
            unit_price: ir.get("unit_price"),
            gross_amount: 0,
            item_discount_amount: 0,
            taxable_amount: 0,
            tax_amount: 0,
            service_charge_amount: 0,
            net_amount: 0,
        });
    }

    // 2. Perform splitting calculations
    let mut split_bills = Vec::new();

    // Create target bill templates
    for i in 0..payload.num_splits {
        split_bills.push(BillWithItems {
            id: Uuid::new_v4().to_string(),
            dining_session_id: original_bill.get("dining_session_id"),
            status: "open".to_string(),
            bill_number: format!("{}-S{}", original_bill.get::<String, _>("bill_number"), i + 1),
            subtotal: 0,
            discount_total: 0,
            tax_total: 0,
            service_total: 0,
            rounding_total: 0,
            grand_total: 0,
            paid_total: 0,
            balance_amount: 0,
            version: 1,
            reference_id: None,
            reference_type: None,
            items: Vec::new(),
        });
    }

    // Allocate items based on strategy
    if payload.strategy == "equally" {
        // Divide each item quantity by num_splits
        let divisor = payload.num_splits as f64;
        for item in &original_items {
            for b in &mut split_bills {
                let split_qty = item.quantity / divisor;
                let gross = (split_qty * item.unit_price as f64).round() as i32;
                b.items.push(BillItemInfo {
                    id: Uuid::new_v4().to_string(),
                    product_id: item.product_id.clone(),
                    product_name_snapshot: item.product_name_snapshot.clone(),
                    order_item_id: item.order_item_id.clone(),
                    parent_bill_item_id: item.parent_bill_item_id.clone(),
                    quantity: split_qty,
                    unit_price: item.unit_price,
                    gross_amount: gross,
                    item_discount_amount: 0,
                    taxable_amount: gross,
                    tax_amount: 0,
                    service_charge_amount: 0,
                    net_amount: gross,
                });
            }
        }
    } else {
        // Custom item/qty allocations
        for target in &payload.targets {
            if target.target_bill_index >= split_bills.len() {
                continue;
            }
            let b = &mut split_bills[target.target_bill_index];
            for t_item in &target.items {
                if let Some(orig_item) = original_items.iter().find(|i| i.id == t_item.id) {
                    let gross = (t_item.quantity_to_move * orig_item.unit_price as f64).round() as i32;
                    b.items.push(BillItemInfo {
                        id: Uuid::new_v4().to_string(),
                        product_id: orig_item.product_id.clone(),
                        product_name_snapshot: orig_item.product_name_snapshot.clone(),
                        order_item_id: orig_item.order_item_id.clone(),
                        parent_bill_item_id: orig_item.parent_bill_item_id.clone(),
                        quantity: t_item.quantity_to_move,
                        unit_price: orig_item.unit_price,
                        gross_amount: gross,
                        item_discount_amount: 0,
                        taxable_amount: gross,
                        tax_amount: 0,
                        service_charge_amount: 0,
                        net_amount: gross,
                    });
                }
            }
        }
    }

    // 3. Proportional tax/service/discount distribution & Rounding Invariant Enforcement
    let mut sum_subtotal = 0;
    let mut sum_discount = 0;
    let mut sum_tax = 0;
    let mut sum_service = 0;
    let mut sum_rounding = 0;
    let mut sum_grand = 0;

    let len = split_bills.len();
    for (idx, b) in split_bills.iter_mut().enumerate() {
        let sub: i32 = b.items.iter().map(|i| i.gross_amount).sum();
        b.subtotal = sub;

        if orig_subtotal > 0 {
            let ratio = sub as f64 / orig_subtotal as f64;
            b.discount_total = (orig_discount as f64 * ratio).round() as i32;
            b.service_total = (orig_service as f64 * ratio).round() as i32;
            b.tax_total = (orig_tax as f64 * ratio).round() as i32;
            b.rounding_total = (orig_rounding as f64 * ratio).round() as i32;
            b.grand_total = b.subtotal - b.discount_total + b.service_total + b.tax_total + b.rounding_total;
            b.balance_amount = b.grand_total;
        }

        sum_subtotal += b.subtotal;
        sum_discount += b.discount_total;
        sum_tax += b.tax_total;
        sum_service += b.service_total;
        sum_rounding += b.rounding_total;
        sum_grand += b.grand_total;

        // Last bill adjustment for precision (Largest Remainder)
        if idx == len - 1 {
            let diff_sub = orig_subtotal - sum_subtotal;
            let diff_disc = orig_discount - sum_discount;
            let diff_tax = orig_tax - sum_tax;
            let diff_serv = orig_service - sum_service;
            let diff_round = orig_rounding - sum_rounding;
            let diff_grand = orig_grand - sum_grand;

            b.subtotal += diff_sub;
            b.discount_total += diff_disc;
            b.tax_total += diff_tax;
            b.service_total += diff_serv;
            b.rounding_total += diff_round;
            b.grand_total += diff_grand;
            b.balance_amount = b.grand_total;
        }
    }

    Ok(SplitBillPreviewResponse {
        original_bill_id: payload.bill_id,
        operation_token: Uuid::new_v4().to_string(),
        bills: split_bills,
    })
}

#[tauri::command]
pub async fn commit_split_bill(
    payload: SplitBillRequest,
    pool: State<'_, SqlitePool>,
) -> Result<SplitBillCommitResponse, String> {
    crate::license::enforce_active_license().await?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Check idempotency
    let audit_exists: Option<String> = sqlx::query_scalar(
        "SELECT target_entity_ids FROM operation_audit_logs WHERE idempotency_key = ?"
    )
    .bind(&payload.idempotency_key)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(target_ids_json) = audit_exists {
        // Return already split bills
        let bill_ids: Vec<String> = serde_json::from_str(&target_ids_json).unwrap_or_default();
        let mut summaries = Vec::new();
        for b_id in bill_ids {
            let br = sqlx::query(
                "SELECT id, bill_number, status, subtotal, discount_total, tax_total, service_total, rounding_total, grand_total, paid_total, balance_amount, version, reference_id, reference_type FROM bills WHERE id = ?"
            )
            .bind(&b_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            summaries.push(BillSummary {
                id: br.get("id"),
                bill_number: br.get("bill_number"),
                status: br.get("status"),
                subtotal: br.get("subtotal"),
                discount_total: br.get("discount_total"),
                tax_total: br.get("tax_total"),
                service_total: br.get("service_total"),
                rounding_total: br.get("rounding_total"),
                grand_total: br.get("grand_total"),
                paid_total: br.get("paid_total"),
                balance_amount: br.get("balance_amount"),
                version: br.get("version"),
                reference_id: br.get("reference_id"),
                reference_type: br.get("reference_type"),
            });
        }
        return Ok(SplitBillCommitResponse {
            success: true,
            transaction_id: payload.idempotency_key,
            bills: summaries,
        });
    }

    // Perform preview logic to get exact proportional math
    let preview = preview_split_bill_inner(payload.clone(), pool.inner()).await?;

    // Verify original bill version and permissions
    let _current_version: i32 = sqlx::query_scalar(
        "SELECT version FROM bills WHERE id = ?"
    )
    .bind(&payload.bill_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Check permissions
    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "bill.split").await?;
    if !has_perm {
        return Err("INSUFFICIENT_PERMISSION".to_string());
    }

    // Insert split bills into DB and update original bill (set to merged/closed or update items)
    let mut target_bill_ids = Vec::new();
    let mut summaries = Vec::new();

    // In this model, the original bill is voided/closed, and brand new split bills are created.
    // This is clean and maintains full audit history.
    sqlx::query("UPDATE bills SET status = 'merged', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&payload.bill_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for pb in preview.bills {
        let bill_id = pb.id.clone();
        target_bill_ids.push(bill_id.clone());

        // Insert new bill
        sqlx::query(
            r#"
            INSERT INTO bills (
                id, dining_session_id, status, bill_number, subtotal, discount_total, tax_total,
                service_total, rounding_total, grand_total, paid_total, balance_amount, version, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#
        )
        .bind(&bill_id)
        .bind(&pb.dining_session_id)
        .bind(&pb.status)
        .bind(&pb.bill_number)
        .bind(pb.subtotal)
        .bind(pb.discount_total)
        .bind(pb.tax_total)
        .bind(pb.service_total)
        .bind(pb.rounding_total)
        .bind(pb.grand_total)
        .bind(pb.balance_amount)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Insert bill items
        for item in &pb.items {
            sqlx::query(
                r#"
                INSERT INTO bill_items (
                    id, bill_id, product_id, product_name_snapshot, order_item_id, parent_bill_item_id,
                    quantity, unit_price, gross_amount, item_discount_amount, taxable_amount, net_amount, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                "#
            )
            .bind(&item.id)
            .bind(&bill_id)
            .bind(&item.product_id)
            .bind(&item.product_name_snapshot)
            .bind(&item.order_item_id)
            .bind(&item.parent_bill_item_id)
            .bind(item.quantity)
            .bind(item.unit_price)
            .bind(item.gross_amount)
            .bind(item.item_discount_amount)
            .bind(item.taxable_amount)
            .bind(item.net_amount)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        // Also create a cart_draft for each split bill so POS can checkout/pay them
        let cart_json = serde_json::json!({
            "cart": pb.items.iter().map(|i| {
                serde_json::json!({
                    "id": Uuid::new_v4().to_string(),
                    "product_id": i.product_id,
                    "name": i.product_name_snapshot,
                    "qty": i.quantity,
                    "price": i.unit_price,
                    "discount_total": i.item_discount_amount,
                    "line_total": i.net_amount,
                    "sku": ""
                })
            }).collect::<Vec<_>>(),
            "subtotal": pb.subtotal,
            "tax": pb.tax_total,
            "grandTotal": pb.grand_total,
            "tableNumber": pb.bill_number.clone(),
            "orderType": "dine_in"
        }).to_string();

        sqlx::query("INSERT INTO cart_drafts (id, name, cart_json, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP)")
            .bind(&bill_id)
            .bind(&pb.bill_number)
            .bind(&cart_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        summaries.push(BillSummary {
            id: bill_id,
            bill_number: pb.bill_number,
            status: pb.status,
            subtotal: pb.subtotal,
            discount_total: pb.discount_total,
            tax_total: pb.tax_total,
            service_total: pb.service_total,
            rounding_total: pb.rounding_total,
            grand_total: pb.grand_total,
            paid_total: 0,
            balance_amount: pb.balance_amount,
            version: 1,
            reference_id: Some(pb.id),
            reference_type: Some("draft".to_string()),
        });
    }

    // Write audit log
    let target_ids_str = serde_json::to_string(&target_bill_ids).unwrap();
    sqlx::query(
        r#"
        INSERT INTO operation_audit_logs (id, operation_type, idempotency_key, source_entity_ids, target_entity_ids, performed_by, authorized_by, reason)
        VALUES (?, 'split_bill', ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&payload.idempotency_key)
    .bind(format!("[\"{}\"]", payload.bill_id))
    .bind(&target_ids_str)
    .bind(user_id.to_string())
    .bind(payload.authorized_by)
    .bind(payload.reason)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(SplitBillCommitResponse {
        success: true,
        transaction_id: payload.idempotency_key,
        bills: summaries,
    })
}

// --- JOIN BILLS ---

#[derive(Deserialize, Debug, Clone)]
pub struct JoinBillsRequest {
    pub bill_ids: Vec<String>,
    pub target_table_id: String,
    pub idempotency_key: String,
    pub authorized_by: Option<String>,
    pub reason: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct JoinBillsPreviewResponse {
    pub combined_bill: BillWithItems,
}

#[derive(Serialize, Debug, Clone)]
pub struct JoinBillsCommitResponse {
    pub success: bool,
    pub transaction_id: String,
    pub bill: BillSummary,
}

#[tauri::command]
pub async fn preview_join_bills(
    payload: JoinBillsRequest,
    pool: State<'_, SqlitePool>,
) -> Result<JoinBillsPreviewResponse, String> {
    preview_join_bills_inner(payload, pool.inner()).await
}

pub async fn preview_join_bills_inner(
    payload: JoinBillsRequest,
    pool: &SqlitePool,
) -> Result<JoinBillsPreviewResponse, String> {
    if payload.bill_ids.is_empty() {
        return Err("No bills provided".to_string());
    }

    let mut combined_items: Vec<BillItemInfo> = Vec::new();
    let mut total_sub = 0;
    let mut total_disc = 0;
    let mut total_tax = 0;
    let mut total_serv = 0;
    let mut total_round = 0;
    let mut total_grand = 0;
    let mut first_session_id = String::new();

    for (idx, b_id) in payload.bill_ids.iter().enumerate() {
        let br = sqlx::query(
            "SELECT dining_session_id, subtotal, discount_total, tax_total, service_total, rounding_total, grand_total FROM bills WHERE id = ?"
        )
        .bind(b_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Bill {} not found", b_id))?;

        if idx == 0 {
            first_session_id = br.get("dining_session_id");
        }

        total_sub += br.get::<i32, _>("subtotal");
        total_disc += br.get::<i32, _>("discount_total");
        total_tax += br.get::<i32, _>("tax_total");
        total_serv += br.get::<i32, _>("service_total");
        total_round += br.get::<i32, _>("rounding_total");
        total_grand += br.get::<i32, _>("grand_total");

        let items_records = sqlx::query(
            "SELECT id, product_id, product_name_snapshot, order_item_id, parent_bill_item_id, quantity, unit_price, gross_amount, item_discount_amount, net_amount FROM bill_items WHERE bill_id = ?"
        )
        .bind(b_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for ir in items_records {
            combined_items.push(BillItemInfo {
                id: Uuid::new_v4().to_string(),
                product_id: ir.get("product_id"),
                product_name_snapshot: ir.get("product_name_snapshot"),
                order_item_id: ir.get("order_item_id"),
                parent_bill_item_id: ir.get("parent_bill_item_id"),
                quantity: ir.get("quantity"),
                unit_price: ir.get("unit_price"),
                gross_amount: ir.get("gross_amount"),
                item_discount_amount: ir.get("item_discount_amount"),
                taxable_amount: ir.get("gross_amount"),
                tax_amount: 0,
                service_charge_amount: 0,
                net_amount: ir.get("net_amount"),
            });
        }
    }

    Ok(JoinBillsPreviewResponse {
        combined_bill: BillWithItems {
            id: Uuid::new_v4().to_string(),
            dining_session_id: first_session_id,
            status: "open".to_string(),
            bill_number: format!("BILL-JOIN-{}", Utc::now().format("%Y%m%d%H%M%S")),
            subtotal: total_sub,
            discount_total: total_disc,
            tax_total: total_tax,
            service_total: total_serv,
            rounding_total: total_round,
            grand_total: total_grand,
            paid_total: 0,
            balance_amount: total_grand,
            version: 1,
            reference_id: None,
            reference_type: None,
            items: combined_items,
        }
    })
}

#[tauri::command]
pub async fn commit_join_bills(
    payload: JoinBillsRequest,
    pool: State<'_, SqlitePool>,
) -> Result<JoinBillsCommitResponse, String> {
    crate::license::enforce_active_license().await?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Check permissions
    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "bill.join").await?;
    if !has_perm {
        return Err("INSUFFICIENT_PERMISSION".to_string());
    }

    let preview = preview_join_bills_inner(payload.clone(), pool.inner()).await?;
    let joined_bill = preview.combined_bill;

    // Void the source bills
    for b_id in &payload.bill_ids {
        sqlx::query("UPDATE bills SET status = 'merged', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(b_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        
        // delete from drafts so POS doesn't show old draft
        sqlx::query("DELETE FROM cart_drafts WHERE id = ?")
            .bind(b_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Insert joined bill
    sqlx::query(
        r#"
        INSERT INTO bills (
            id, dining_session_id, status, bill_number, subtotal, discount_total, tax_total,
            service_total, rounding_total, grand_total, paid_total, balance_amount, version, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#
    )
    .bind(&joined_bill.id)
    .bind(&joined_bill.dining_session_id)
    .bind(&joined_bill.status)
    .bind(&joined_bill.bill_number)
    .bind(joined_bill.subtotal)
    .bind(joined_bill.discount_total)
    .bind(joined_bill.tax_total)
    .bind(joined_bill.service_total)
    .bind(joined_bill.rounding_total)
    .bind(joined_bill.grand_total)
    .bind(joined_bill.balance_amount)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    for item in &joined_bill.items {
        sqlx::query(
            r#"
            INSERT INTO bill_items (
                id, bill_id, product_id, product_name_snapshot, order_item_id, parent_bill_item_id,
                quantity, unit_price, gross_amount, item_discount_amount, taxable_amount, net_amount, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#
        )
        .bind(&item.id)
        .bind(&joined_bill.id)
        .bind(&item.product_id)
        .bind(&item.product_name_snapshot)
        .bind(&item.order_item_id)
        .bind(&item.parent_bill_item_id)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(item.gross_amount)
        .bind(item.item_discount_amount)
        .bind(item.taxable_amount)
        .bind(item.net_amount)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Save cart draft for the joined bill
    let cart_json = serde_json::json!({
        "cart": joined_bill.items.iter().map(|i| {
            serde_json::json!({
                "id": Uuid::new_v4().to_string(),
                "product_id": i.product_id,
                "name": i.product_name_snapshot,
                "qty": i.quantity,
                "price": i.unit_price,
                "discount_total": i.item_discount_amount,
                "line_total": i.net_amount,
                "sku": ""
            })
        }).collect::<Vec<_>>(),
        "subtotal": joined_bill.subtotal,
        "tax": joined_bill.tax_total,
        "grandTotal": joined_bill.grand_total,
        "tableNumber": joined_bill.bill_number.clone(),
        "orderType": "dine_in"
    }).to_string();

    sqlx::query("INSERT INTO cart_drafts (id, name, cart_json, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP)")
        .bind(&joined_bill.id)
        .bind(&joined_bill.bill_number)
        .bind(&cart_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Audit log
    let source_ids_str = serde_json::to_string(&payload.bill_ids).unwrap();
    sqlx::query(
        r#"
        INSERT INTO operation_audit_logs (id, operation_type, idempotency_key, source_entity_ids, target_entity_ids, performed_by, authorized_by, reason)
        VALUES (?, 'join_bill', ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&payload.idempotency_key)
    .bind(&source_ids_str)
    .bind(format!("[\"{}\"]", joined_bill.id))
    .bind(user_id.to_string())
    .bind(payload.authorized_by)
    .bind(payload.reason)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(JoinBillsCommitResponse {
        success: true,
        transaction_id: payload.idempotency_key,
        bill: BillSummary {
            id: joined_bill.id,
            bill_number: joined_bill.bill_number,
            status: joined_bill.status,
            subtotal: joined_bill.subtotal,
            discount_total: joined_bill.discount_total,
            tax_total: joined_bill.tax_total,
            service_total: joined_bill.service_total,
            rounding_total: joined_bill.rounding_total,
            grand_total: joined_bill.grand_total,
            paid_total: 0,
            balance_amount: joined_bill.balance_amount,
            version: 1,
            reference_id: None,
            reference_type: None,
        },
    })
}

// --- TABLE TRANSFERS (MOVE, SWAP, JOIN) ---

#[tauri::command]
pub async fn commit_move_table(
    source_table_id: String,
    destination_table_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "table.move").await?;
    if !has_perm {
        return Err("INSUFFICIENT_PERMISSION".to_string());
    }

    // Get active session on source table
    let session_id: Option<String> = sqlx::query_scalar(
        r#"
        SELECT ds.id FROM dining_sessions ds
        JOIN session_tables st ON ds.id = st.dining_session_id
        WHERE st.table_id = ? AND ds.status = 'active'
        LIMIT 1
        "#
    )
    .bind(&source_table_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let s_id = session_id.ok_or_else(|| "No active session on source table".to_string())?;

    // Check that destination table is available
    let dest_status: String = sqlx::query_scalar("SELECT status FROM tables WHERE id = ?")
        .bind(&destination_table_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if dest_status != "available" {
        return Err("Destination table is not available".to_string());
    }

    // Update session_tables to link to destination table instead of source
    sqlx::query("UPDATE session_tables SET table_id = ? WHERE dining_session_id = ? AND table_id = ?")
        .bind(&destination_table_id)
        .bind(&s_id)
        .bind(&source_table_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Update source table to 'available'
    sqlx::query("UPDATE tables SET status = 'available', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&source_table_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Update destination table to 'occupied'
    sqlx::query("UPDATE tables SET status = 'occupied', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&destination_table_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Record transfer history
    sqlx::query(
        "INSERT INTO table_transfer_history (id, dining_session_id, source_table_id, destination_table_id, transfer_type, performed_by) \
         VALUES (?, ?, ?, ?, 'move', ?)"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&s_id)
    .bind(&source_table_id)
    .bind(&destination_table_id)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn commit_swap_tables(
    source_table_id: String,
    destination_table_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "table.swap").await?;
    if !has_perm {
        return Err("INSUFFICIENT_PERMISSION".to_string());
    }

    // Get active sessions
    let source_session_id: Option<String> = sqlx::query_scalar(
        "SELECT dining_session_id FROM session_tables st JOIN dining_sessions ds ON st.dining_session_id = ds.id WHERE st.table_id = ? AND ds.status = 'active' LIMIT 1"
    )
    .bind(&source_table_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let dest_session_id: Option<String> = sqlx::query_scalar(
        "SELECT dining_session_id FROM session_tables st JOIN dining_sessions ds ON st.dining_session_id = ds.id WHERE st.table_id = ? AND ds.status = 'active' LIMIT 1"
    )
    .bind(&destination_table_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let s_sess = source_session_id.ok_or_else(|| "No active session on source table".to_string())?;
    let d_sess = dest_session_id.ok_or_else(|| "No active session on destination table".to_string())?;

    // Swap tables in session_tables
    sqlx::query("UPDATE session_tables SET table_id = 'temp_swap_table' WHERE dining_session_id = ? AND table_id = ?")
        .bind(&s_sess)
        .bind(&source_table_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE session_tables SET table_id = ? WHERE dining_session_id = ? AND table_id = ?")
        .bind(&source_table_id)
        .bind(&d_sess)
        .bind(&destination_table_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE session_tables SET table_id = ? WHERE dining_session_id = ? AND table_id = 'temp_swap_table'")
        .bind(&destination_table_id)
        .bind(&s_sess)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Record histories
    sqlx::query(
        "INSERT INTO table_transfer_history (id, dining_session_id, source_table_id, destination_table_id, transfer_type, performed_by) \
         VALUES (?, ?, ?, ?, 'swap', ?)"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&s_sess)
    .bind(&source_table_id)
    .bind(&destination_table_id)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn commit_join_tables(
    source_table_id: String,
    destination_table_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "table.join").await?;
    if !has_perm {
        return Err("INSUFFICIENT_PERMISSION".to_string());
    }

    // Get active session of destination table (we join source session into destination session, or vice versa.
    // Let's join source table into destination session)
    let dest_session_id: Option<String> = sqlx::query_scalar(
        "SELECT dining_session_id FROM session_tables st JOIN dining_sessions ds ON st.dining_session_id = ds.id WHERE st.table_id = ? AND ds.status = 'active' LIMIT 1"
    )
    .bind(&destination_table_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let d_sess = dest_session_id.ok_or_else(|| "Destination table has no active session".to_string())?;

    // Check if source table has active session. If it has, merge the session.
    let source_session_id: Option<String> = sqlx::query_scalar(
        "SELECT dining_session_id FROM session_tables st JOIN dining_sessions ds ON st.dining_session_id = ds.id WHERE st.table_id = ? AND ds.status = 'active' LIMIT 1"
    )
    .bind(&source_table_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(s_sess) = source_session_id {
        // Move all tables from source session to destination session
        sqlx::query("UPDATE session_tables SET dining_session_id = ? WHERE dining_session_id = ?")
            .bind(&d_sess)
            .bind(&s_sess)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Merge bills
        sqlx::query("UPDATE bills SET dining_session_id = ?, version = version + 1 WHERE dining_session_id = ?")
            .bind(&d_sess)
            .bind(&s_sess)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Close source session
        sqlx::query("UPDATE dining_sessions SET status = 'merged', merged_into_session_id = ?, version = version + 1 WHERE id = ?")
            .bind(&d_sess)
            .bind(&s_sess)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        // Source table has no session, just add it to destination session
        sqlx::query("INSERT OR IGNORE INTO session_tables (id, dining_session_id, table_id) VALUES (?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(&d_sess)
            .bind(&source_table_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Mark source table occupied
        sqlx::query("UPDATE tables SET status = 'occupied', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&source_table_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

// ========================
// POS Table Management Commands
// ========================

/// Add a new physical table to the database for the active outlet.
/// Called from the POS "Tambah Meja Baru" dialog.
#[tauri::command]
pub async fn add_new_table(name: String, pool: State<'_, SqlitePool>) -> Result<TableStatus, String> {
    if name.trim().is_empty() {
        return Err("Nama meja tidak boleh kosong".to_string());
    }

    let table_id = Uuid::new_v4().to_string();

    // Get first outlet
    let outlet = sqlx::query("SELECT id FROM outlets LIMIT 1")
        .fetch_one(pool.inner())
        .await
        .map_err(|e| format!("Outlet tidak ditemukan: {}", e))?;

    let outlet_id: String = outlet.get("id");

    sqlx::query(
        "INSERT INTO tables (id, outlet_id, name, status, version) VALUES (?, ?, ?, 'available', 1)"
    )
    .bind(&table_id)
    .bind(&outlet_id)
    .bind(name.trim())
    .execute(pool.inner())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            format!("Meja dengan nama '{}' sudah ada", name.trim())
        } else {
            e.to_string()
        }
    })?;

    Ok(TableStatus {
        id: table_id,
        name: name.trim().to_string(),
        status: "available".to_string(),
        version: 1,
        active_session: None,
    })
}

/// Release/cancel an active dining session for a table (close session, set table to available).
/// Called from the POS when a cashier wants to manually free a table.
#[tauri::command]
pub async fn release_table_session(table_name: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Get the table record
    let table_rec = sqlx::query("SELECT id FROM tables WHERE name = ? LIMIT 1")
        .bind(&table_name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let table_id = match table_rec {
        Some(r) => r.get::<String, _>("id"),
        None => return Err(format!("Meja '{}' tidak ditemukan", table_name)),
    };

    // Find the active dining session for this table
    let session_rec = sqlx::query(
        "SELECT ds.id FROM dining_sessions ds \
         JOIN session_tables st ON ds.id = st.dining_session_id \
         WHERE st.table_id = ? AND ds.status = 'active' LIMIT 1"
    )
    .bind(&table_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(sr) = session_rec {
        let session_id: String = sr.get("id");

        // Close all open bills in this session
        sqlx::query(
            "UPDATE bills SET status = 'closed', updated_at = CURRENT_TIMESTAMP WHERE dining_session_id = ? AND status = 'open'"
        )
        .bind(&session_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Close the dining session
        sqlx::query("UPDATE dining_sessions SET status = 'closed', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&session_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Update all tables in this session to available
        sqlx::query(
            "UPDATE tables SET status = 'available', version = version + 1, updated_at = CURRENT_TIMESTAMP \
             WHERE id IN (SELECT table_id FROM session_tables WHERE dining_session_id = ?)"
        )
        .bind(&session_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        // No active session, just ensure table is available anyway
        sqlx::query("UPDATE tables SET status = 'available', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&table_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

