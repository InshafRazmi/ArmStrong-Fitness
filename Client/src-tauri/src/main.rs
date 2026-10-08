#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(all(feature = "packaged-auth", feature = "ui-smoke"))]
compile_error!("Installer builds must not include UI smoke commands");
#[cfg(all(windows, not(debug_assertions), not(feature = "packaged-auth")))]
compile_error!("Windows release builds require the packaged-auth feature");
use armstrong_core::{
    AllocationInput, AttendanceInput, BackupEnvelope, BusinessRetryInput, CombinedPaymentInput,
    ExpenseInput, ExpenseVoidInput, InvoiceInput, MemberConflictInput, MemberInput,
    MemberRemovalInput, PaymentInput, PeriodInput, PlanInput, ProductInput, ProfileInput,
    ReceivePaymentInput, RegisterMemberInput, RenewalInput, ReportRange, ReversalInput, SaleInput,
    StaffAttendanceInput, StaffMemberInput, StaffPayoutInput, StaffRegisterInput,
    StaffRemovalInput, StockInput, Store, TrainerInput, TrainingChargeInput,
};
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{Manager, State};

struct Database(Mutex<Store>, Arc<armstrong_core::DesktopAuth>);
impl Database {
    fn access(&self, write: bool) -> Result<MutexGuard<'_, Store>, String> {
        let mut store = self.0.lock().map_err(|_| "Database unavailable")?;
        self.1.authorize(&mut store, write)?;
        Ok(store)
    }
}
#[tauri::command]
fn desktop_auth_status(db: State<Database>) -> Result<armstrong_core::DesktopAuthStatus, String> {
    let store = db.0.lock().map_err(|_| "Database unavailable")?;
    db.1.status(&store)
}
#[tauri::command]
fn desktop_logout(db: State<Database>) -> Result<(), String> {
    let mut store = db.0.lock().map_err(|_| "Database unavailable")?;
    db.1.logout(&mut store)
}
#[tauri::command]
fn desktop_unlock_offline(
    db: State<Database>,
) -> Result<armstrong_core::DesktopAuthStatus, String> {
    let mut store = db.0.lock().map_err(|_| "Database unavailable")?;
    db.1.unlock_offline(&mut store)
}
#[tauri::command]
async fn desktop_login(
    db: State<'_, Database>,
    email: String,
    password: String,
) -> Result<armstrong_core::DesktopAuthStatus, String> {
    let (path, epoch) = {
        let mut store = db.0.lock().map_err(|_| "Database unavailable")?;
        let epoch = db.1.begin(&mut store)?;
        (store.database_path(), epoch)
    };
    let auth = db.1.clone();
    let pending = tauri::async_runtime::spawn_blocking(move || {
        auth.authenticate(&path, epoch, &email, &password)
    })
    .await
    .map_err(|_| "Native sign-in unavailable; values withheld".to_string())??;
    let mut store = db.0.lock().map_err(|_| "Database unavailable")?;
    db.1.finish(&mut store, pending)
}
#[tauri::command]
async fn desktop_renew_session(
    db: State<'_, Database>,
) -> Result<armstrong_core::DesktopAuthStatus, String> {
    let job = {
        let store = db.0.lock().map_err(|_| "Database unavailable")?;
        match db.1.prepare_session_renewal(&store)? {
            Some(job) => job,
            None => return db.1.status(&store),
        }
    };
    let auth = db.1.clone();
    let result = tauri::async_runtime::spawn_blocking(move || auth.run_session_renewal(job))
        .await
        .map_err(|_| "Native session renewal unavailable; values withheld".to_string())??;
    let mut store = db.0.lock().map_err(|_| "Database unavailable")?;
    db.1.finish_session_renewal(&mut store, result)
}
#[tauri::command]
async fn prepare_native_device(
    db: State<'_, Database>,
) -> Result<armstrong_core::DeviceApproval, String> {
    let path =
        db.0.lock()
            .map_err(|_| "Database unavailable")?
            .database_path();
    tauri::async_runtime::spawn_blocking(move || Store::open(&path)?.prepare_native_device())
        .await
        .map_err(|_| "Native device preparation unavailable; values withheld".to_string())?
}
#[tauri::command]
fn foundation_snapshot(db: State<Database>) -> Result<serde_json::Value, String> {
    let store = db.access(false)?;
    let mut snapshot = store.snapshot()?;
    db.1.member_sync_status(&store, &mut snapshot);
    Ok(snapshot)
}
#[tauri::command]
async fn synchronize_members(
    db: State<'_, Database>,
) -> Result<armstrong_core::MemberSyncOutcome, String> {
    let job = {
        let store = db.access(false)?;
        db.1.prepare_member_sync(&store)?
    };
    let auth = db.1.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || auth.run_member_sync(job))
        .await
        .map_err(|_| "Native member synchronization unavailable; values withheld".to_string())??;
    let mut store = db.0.lock().map_err(|_| "Database unavailable")?;
    db.1.finish_member_sync(&mut store, outcome)
}
#[tauri::command]
fn preview_business_retry(
    db: State<Database>,
    batch_id: String,
) -> Result<serde_json::Value, String> {
    let store = db.access(false)?;
    db.1.prepare_member_sync(&store)?;
    store.preview_business_retry(&batch_id)
}
#[tauri::command]
fn retry_business_transaction(
    db: State<Database>,
    input: BusinessRetryInput,
) -> Result<serde_json::Value, String> {
    let mut store = db.access(true)?;
    db.1.prepare_member_sync(&store)?;
    store.retry_business_transaction(input)
}
#[tauri::command]
fn save_plan(db: State<Database>, input: PlanInput) -> Result<(), String> {
    db.access(true)?.save_plan(input)
}
#[tauri::command]
fn save_member(db: State<Database>, input: MemberInput) -> Result<(), String> {
    db.access(true)?.save_member(input)
}
#[tauri::command]
fn save_trainer(db: State<Database>, input: TrainerInput) -> Result<serde_json::Value, String> {
    db.access(true)?.save_trainer(input)
}
#[tauri::command]
fn delete_staff(
    db: State<Database>,
    input: StaffRemovalInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.delete_staff(input)
}
#[tauri::command]
fn register_member_with_trainer(
    db: State<Database>,
    input: StaffRegisterInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.register_member_with_trainer(input)
}
#[tauri::command]
fn save_member_with_trainer(db: State<Database>, input: StaffMemberInput) -> Result<(), String> {
    db.access(true)?.save_member_with_trainer(input)
}
#[tauri::command]
fn create_training_charge(
    db: State<Database>,
    input: TrainingChargeInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.create_training_charge(input)
}
#[tauri::command]
fn receive_combined_payment(
    db: State<Database>,
    input: CombinedPaymentInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.receive_combined_payment(input)
}
#[tauri::command]
fn pay_staff(db: State<Database>, input: StaffPayoutInput) -> Result<serde_json::Value, String> {
    db.access(true)?.pay_staff(input)
}
#[tauri::command]
fn register_member(
    db: State<Database>,
    input: RegisterMemberInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.register_member(input)
}
#[tauri::command]
fn add_membership_period(db: State<Database>, input: PeriodInput) -> Result<(), String> {
    db.access(true)?.add_period(input)
}
#[tauri::command]
fn save_gym_profile(db: State<Database>, input: ProfileInput) -> Result<serde_json::Value, String> {
    db.access(true)?.save_profile(input)
}
#[tauri::command]
fn record_attendance(
    db: State<Database>,
    input: AttendanceInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.record_attendance(input)
}
#[tauri::command]
fn record_payment(db: State<Database>, input: PaymentInput) -> Result<serde_json::Value, String> {
    db.access(true)?.record_payment(input)
}
#[tauri::command]
fn record_expense(db: State<Database>, input: ExpenseInput) -> Result<serde_json::Value, String> {
    db.access(true)?.record_expense(input)
}
#[tauri::command]
fn save_product(db: State<Database>, input: ProductInput) -> Result<serde_json::Value, String> {
    db.access(true)?.save_product(input)
}
#[tauri::command]
fn adjust_stock(db: State<Database>, input: StockInput) -> Result<serde_json::Value, String> {
    db.access(true)?.adjust_stock(input)
}
#[tauri::command]
fn complete_sale(db: State<Database>, input: SaleInput) -> Result<serde_json::Value, String> {
    db.access(true)?.complete_sale(input)
}
#[tauri::command]
fn preview_restore(
    db: State<Database>,
    input: BackupEnvelope,
) -> Result<serde_json::Value, String> {
    db.access(true)?.preview_restore(input)
}
#[tauri::command]
fn export_backup(db: State<Database>) -> Result<serde_json::Value, String> {
    db.access(false)?.export_backup()
}
#[tauri::command]
fn restore_backup(db: State<Database>, token: String) -> Result<serde_json::Value, String> {
    db.access(true)?.restore_backup(token)
}
#[tauri::command]
fn export_report(
    db: State<Database>,
    kind: String,
    range: Option<ReportRange>,
) -> Result<serde_json::Value, String> {
    db.access(false)?
        .export_report_range(kind, range.unwrap_or_default())
}
#[tauri::command]
fn report_summary(db: State<Database>, range: ReportRange) -> Result<serde_json::Value, String> {
    db.access(false)?.report_summary(range)
}
#[tauri::command]
fn create_invoice(db: State<Database>, input: InvoiceInput) -> Result<serde_json::Value, String> {
    db.access(true)?.create_invoice(input)
}
#[tauri::command]
fn allocate_payment(
    db: State<Database>,
    input: AllocationInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.allocate_payment(input)
}
#[tauri::command]
fn receive_payment(
    db: State<Database>,
    input: ReceivePaymentInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.receive_payment(input)
}
#[tauri::command]
fn renew_membership(db: State<Database>, input: RenewalInput) -> Result<serde_json::Value, String> {
    db.access(true)?.renew_membership(input)
}
#[tauri::command]
fn reverse_payment(db: State<Database>, input: ReversalInput) -> Result<serde_json::Value, String> {
    db.access(true)?.reverse_payment(input)
}
#[tauri::command]
fn payment_receipt(db: State<Database>, payment_id: String) -> Result<serde_json::Value, String> {
    db.access(false)?.payment_receipt(payment_id)
}
#[tauri::command]
fn preview_member_conflict(
    db: State<Database>,
    conflict_id: String,
) -> Result<serde_json::Value, String> {
    db.access(true)?.preview_member_conflict(&conflict_id)
}
#[tauri::command]
fn resolve_member_conflict(
    db: State<Database>,
    input: MemberConflictInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.resolve_member_conflict(input)
}
#[tauri::command]
fn archive_member(
    db: State<Database>,
    input: MemberRemovalInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.archive_member(input)
}
#[tauri::command]
fn delete_member(
    db: State<Database>,
    input: MemberRemovalInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.delete_member(input)
}
#[tauri::command]
fn record_staff_attendance(
    db: State<Database>,
    input: StaffAttendanceInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.record_staff_attendance(input)
}
#[tauri::command]
fn record_nfc_attendance(
    db: State<Database>,
    input: AttendanceInput,
) -> Result<serde_json::Value, String> {
    db.access(true)?.record_nfc_attendance(input)
}
#[tauri::command]
fn void_expense(db: State<Database>, input: ExpenseVoidInput) -> Result<serde_json::Value, String> {
    db.access(true)?.void_expense(input)
}
#[cfg(feature = "ui-smoke")]
#[tauri::command]
fn smoke_progress(stage: String) {
    println!("ARMSTRONG_UI_SMOKE stage: {stage}");
}
#[cfg(feature = "ui-smoke")]
#[tauri::command]
fn smoke_finished(app: tauri::AppHandle, error: Option<String>) {
    if let Some(error) = error {
        eprintln!("ARMSTRONG_UI_SMOKE FAILED: {error}");
        app.exit(1);
    } else {
        println!("ARMSTRONG_UI_SMOKE PASSED");
        app.exit(0);
    }
}
fn main() {
    let builder = tauri::Builder::default();
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());
    let builder = builder.setup(|app| {
        #[cfg(not(feature = "ui-smoke"))]
        let dir = app.path().app_data_dir()?;
        #[cfg(feature = "ui-smoke")]
        let dir = std::path::PathBuf::from(
            std::env::var("ARMSTRONG_SMOKE_DIR")
                .expect("UI smoke builds require an isolated ARMSTRONG_SMOKE_DIR"),
        );
        std::fs::create_dir_all(&dir)?;
        let mut store =
            Store::open(&dir.join("armstrong.sqlite3")).map_err(std::io::Error::other)?;
        #[cfg(all(feature = "packaged-auth", not(feature = "ui-smoke")))]
        let auth = Arc::new(armstrong_core::DesktopAuth::load_packaged(&dir));
        #[cfg(any(not(feature = "packaged-auth"), feature = "ui-smoke"))]
        let auth = Arc::new(armstrong_core::DesktopAuth::load(&dir));
        auth.initialize(&mut store).map_err(std::io::Error::other)?;
        app.manage(Database(Mutex::new(store), auth));
        Ok(())
    });
    #[cfg(not(feature = "ui-smoke"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        desktop_auth_status,
        desktop_login,
        desktop_renew_session,
        desktop_logout,
        desktop_unlock_offline,
        prepare_native_device,
        foundation_snapshot,
        synchronize_members,
        preview_business_retry,
        retry_business_transaction,
        save_plan,
        save_member,
        save_trainer,
        delete_staff,
        register_member_with_trainer,
        save_member_with_trainer,
        create_training_charge,
        receive_combined_payment,
        pay_staff,
        register_member,
        add_membership_period,
        save_gym_profile,
        record_attendance,
        record_staff_attendance,
        record_nfc_attendance,
        record_payment,
        create_invoice,
        allocate_payment,
        receive_payment,
        renew_membership,
        reverse_payment,
        payment_receipt,
        record_expense,
        preview_member_conflict,
        resolve_member_conflict,
        archive_member,
        delete_member,
        void_expense,
        save_product,
        adjust_stock,
        complete_sale,
        export_backup,
        preview_restore,
        restore_backup,
        export_report,
        report_summary
    ]);
    #[cfg(feature = "ui-smoke")]
    let builder = builder
        .invoke_handler(tauri::generate_handler![
            desktop_auth_status,
            desktop_login,
            desktop_renew_session,
            desktop_logout,
            desktop_unlock_offline,
            prepare_native_device,
            foundation_snapshot,
            synchronize_members,
            preview_business_retry,
            retry_business_transaction,
            save_plan,
            save_member,
            save_trainer,
            delete_staff,
            register_member_with_trainer,
            save_member_with_trainer,
            create_training_charge,
            receive_combined_payment,
            pay_staff,
            register_member,
            add_membership_period,
            save_gym_profile,
            record_attendance,
            record_staff_attendance,
            record_nfc_attendance,
            record_payment,
            create_invoice,
            allocate_payment,
            receive_payment,
            renew_membership,
            reverse_payment,
            payment_receipt,
            record_expense,
            preview_member_conflict,
            resolve_member_conflict,
            archive_member,
            delete_member,
            void_expense,
            save_product,
            adjust_stock,
            complete_sale,
            export_backup,
            preview_restore,
            restore_backup,
            export_report,
            report_summary,
            smoke_finished,
            smoke_progress
        ])
        .on_page_load(|webview, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                webview
                    .eval(include_str!("../../tests/desktop-smoke.js"))
                    .expect("Could not start UI smoke test");
            }
        });
    builder.run(tauri::generate_context!()).expect(
        "Armstrong could not open its database or desktop window. No recovery data was replaced.",
    );
}
