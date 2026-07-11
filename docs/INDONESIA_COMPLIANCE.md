# INDONESIA COMPLIANCE NOTES

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan checklist kepatuhan produk untuk konteks Indonesia. Dokumen ini bukan nasihat hukum/pajak final. Semua aturan pajak, pembayaran, dan perlindungan data harus divalidasi ulang dengan konsultan pajak/legal sebelum produksi.

## 1. Compliance Principles

- POS harus menghasilkan catatan transaksi yang rapi, konsisten, dan dapat diaudit.
- POS harus mendukung format rupiah dan praktik operasional Indonesia.
- POS harus menjaga data pelanggan dan transaksi.
- POS harus bisa export data agar merchant dapat memenuhi kebutuhan akuntansi/pajak.
- Integrasi pembayaran resmi harus mengikuti aturan penyedia pembayaran dan regulator.
- Jangan mengklaim compliance penuh sebelum validasi legal/pajak.

## 2. Currency and Number Formatting

Requirements:

- Currency default: IDR/Rupiah.
- Decimal currency not required for rupiah display.
- Format display: `Rp10.000`.
- Internal monetary storage: integer rupiah.
- Rounding policy must be explicit.

Rounding policy:

| Use Case | Recommendation |
|---|---|
| Item price | integer rupiah |
| Discount amount | integer rupiah |
| Tax/service calculation | calculate then round explicitly |
| Cash change | integer rupiah |
| Report totals | sum stored transaction rows, not recalculated from UI |

## 3. Receipt Requirements

Receipt should include:

- Merchant name.
- Outlet name/address.
- Transaction number.
- Date/time.
- Cashier name/code.
- Items.
- Quantity.
- Unit price.
- Discount.
- Tax/service if configured.
- Grand total.
- Payment method.
- Paid amount and change for cash.
- Refund/void marker if applicable.
- Optional footer.

Receipt number format recommendation:

```text
OUTLETCODE-YYYYMMDD-DEVICESEQ
```

Example:

```text
JKT01-20260705-K01-000123
```

## 4. Tax and Service Charge

MVP should support configurable tax/service charge, not hardcoded tax assumptions.

Requirements:

- Tax can be enabled/disabled per merchant/outlet.
- Service charge can be enabled/disabled.
- Tax/service calculation must be visible before payment.
- Report should separate subtotal, discount, tax, service, grand total.
- Tax label should be configurable.

Important:

- Do not hardcode a tax rate as legal truth.
- Merchant/admin should configure tax settings.
- Production tax handling must be reviewed by tax consultant.

## 5. Payment Methods

MVP payment methods:

- Cash.
- Manual QRIS record.
- Bank transfer/manual record.
- Debit/EDC manual record.
- Other/manual.

Manual QRIS record means:

- POS records that payment method is QRIS.
- POS may store reference number if cashier enters it.
- POS does not automatically verify settlement unless payment provider integration exists.

Future payment integration:

- Requires ADR.
- Requires provider contract.
- Requires webhook signature verification.
- Requires reconciliation workflow.
- Requires settlement report.

## 6. QRIS Considerations

For MVP:

- Treat QRIS as manual payment record.
- Do not claim real-time QRIS confirmation unless integrated with licensed payment provider.
- Allow cashier to enter reference number.
- Report QRIS manual separately from cash.

For production integration:

- Use official payment gateway/acquirer.
- Verify webhook signatures.
- Handle pending/paid/failed/expired statuses.
- Reconcile settlement.
- Keep audit log for manual override.

## 7. Data Protection

Data categories:

- Transaction data.
- Employee/user data.
- Customer data if CRM is implemented.
- Payment reference data.
- Backup data.
- Diagnostic logs.

Requirements:

- Collect minimum customer data for MVP.
- Do not require customer personal data for checkout.
- Redact personal data in diagnostics.
- Encrypt backup.
- Support export.
- Support deletion/anonymization policy if CRM is implemented later.

## 8. Audit and Fraud Control

Sensitive actions requiring audit:

- Login/logout.
- Open shift.
- Close shift.
- Checkout.
- Refund.
- Void.
- Discount override.
- Stock adjustment.
- Stock opname.
- Role change.
- Backup restore.
- License activation.
- Device revoke.

Audit log should include:

- Actor.
- Action.
- Target.
- Reason.
- Timestamp.
- Device/outlet.
- Redacted metadata.

## 9. Report Exports

Required exports:

- Daily sales summary.
- Payment method breakdown.
- Shift report.
- Product sales.
- Stock movement.
- Refund/void report.
- Audit log export.

Formats:

- CSV.
- XLSX later if spreadsheet package exists.

Export rules:

- Export must work in Restricted Expired Mode.
- Export must not require server.
- Export should include generated timestamp and app version.

## 10. Invoice and Tax Future

Not MVP unless explicitly required.

Future modules:

- Invoice numbering.
- Tax invoice support.
- Accounting export.
- Integration with accounting software.
- E-faktur or tax system integration if legally/technically required.

These require:

- New ADR.
- Tax consultant review.
- Dedicated test cases.

## 11. Operational Compliance Checklist

Before pilot:

- Receipt format approved by merchant.
- Tax/service settings tested.
- Refund/void audit tested.
- Export report tested.
- Backup/restore tested.
- User roles tested.
- Cashier shift report tested.

Before production:

- Legal/tax review completed.
- Payment provider review completed if integrated.
- Data protection policy drafted.
- Terms of service and privacy policy drafted.
- Support procedure drafted.

## 12. Acceptance Criteria

- POS displays and stores rupiah amounts consistently.
- Receipt includes required operational fields.
- Tax/service charge is configurable, not hardcoded as legal assumption.
- Manual QRIS record works as payment method.
- Reports separate payment methods.
- Export works offline and in Restricted Expired Mode.
- Sensitive actions create audit logs.
- Compliance-sensitive features are marked for legal/tax validation before production.
