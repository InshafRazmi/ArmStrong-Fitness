use super::*;

struct Fixture {
    store: Store,
    path: std::path::PathBuf,
    staff: String,
    member: String,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("armstrong-attendance-{}.sqlite3", id()));
        let mut store = Store::open(&path).unwrap();
        let staff = store
            .save_trainer(TrainerInput {
                request_id: id(),
                id: None,
                version: None,
                name: "Synthetic staff".into(),
                phone: "0771234567".into(),
                nic: "900000002V".into(),
                salary_minor: 0,
                training_fee_minor: 0,
                active: true,
                nfc_id: Some("STAFF-CARD".into()),
            })
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let member = store
            .register_member_with_trainer(StaffRegisterInput {
                member: RegisterMemberInput {
                    request_id: id(),
                    name: "Synthetic female member".into(),
                    phone: "0771234567".into(),
                    email: "".into(),
                    nfc_id: "MEMBER-CARD".into(),
                    plan_id: None,
                    plan_version: None,
                    starts_on: None,
                },
                trainer_id: None,
                trainer_version: None,
                gender: Some("Female".into()),
            })
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            store,
            path,
            staff,
            member,
        }
    }
    fn scan(&self) -> StaffAttendanceInput {
        StaffAttendanceInput {
            request_id: id(),
            staff_or_card: "staff-card".into(),
            source: "NFC".into(),
        }
    }
    fn now() -> DateTime<Utc> {
        "2026-10-08T18:29:55Z".parse().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(format!("{}-wal", self.path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", self.path.display()));
    }
}

#[test]
fn staff_scans_alternate_suppress_bounce_and_restart_each_colombo_business_day() {
    let mut f = Fixture::new();
    let now = Fixture::now();
    let first = f.store.staff_attendance_at(f.scan(), now).unwrap();
    assert_eq!(first["type"], "Check-in");
    let bounced = f
        .store
        .staff_attendance_at(f.scan(), now + chrono::Duration::milliseconds(1999))
        .unwrap();
    assert_eq!(bounced["duplicate"], true);
    assert_eq!(bounced["id"], first["id"]);
    let second = f
        .store
        .staff_attendance_at(f.scan(), now + chrono::Duration::seconds(2))
        .unwrap();
    assert_eq!(second["type"], "Check-out");
    let next = f
        .store
        .staff_attendance_at(f.scan(), now + chrono::Duration::seconds(5))
        .unwrap();
    assert_eq!(next["type"], "Check-in");
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["staffAttendance"].as_array().unwrap().len(), 3);
    assert_eq!(s["staffAttendance"][0]["businessOn"], "2026-10-09");
    assert_eq!(s["attendance"], json!([]));
    let manual = StaffAttendanceInput {
        request_id: id(),
        staff_or_card: f.staff.clone(),
        source: "Manual".into(),
    };
    let result = f
        .store
        .staff_attendance_at(manual, now + chrono::Duration::seconds(6))
        .unwrap();
    assert_eq!(result["type"], "Check-out");
    assert!(f
        .store
        .conn
        .execute("UPDATE staff_attendance SET staff_name='Changed'", [])
        .is_err());
    assert!(f
        .store
        .conn
        .execute("DELETE FROM staff_attendance", [])
        .is_err());
}

#[test]
fn automatic_nfc_receipts_replay_original_owner_after_restart_and_reassignment() {
    let mut f = Fixture::new();
    let input = AttendanceInput {
        request_id: id(),
        member_or_card: "STAFF-CARD".into(),
        source: "NFC".into(),
    };
    let first = f.store.record_nfc_attendance(input.clone()).unwrap();
    assert_eq!(first["entity"], "Staff");
    {
        let tx = f.store.conn.transaction().unwrap();
        assign_staff_card(&tx, &f.staff, Some("")).unwrap();
        tx.commit().unwrap();
    }
    f.store
        .save_member(MemberInput {
            id: Some(f.member.clone()),
            version: Some(1),
            name: "Synthetic female member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "STAFF-CARD".into(),
        })
        .unwrap();
    let mut reopened = Store::open(&f.path).unwrap();
    assert_eq!(
        reopened.record_nfc_attendance(input.clone()).unwrap(),
        first
    );
    let mut reused = input;
    reused.member_or_card = "MEMBER-CARD".into();
    assert!(reopened
        .record_nfc_attendance(reused)
        .unwrap_err()
        .contains("reused"));
    let fresh = reopened
        .record_nfc_attendance(AttendanceInput {
            request_id: id(),
            member_or_card: "STAFF-CARD".into(),
            source: "NFC".into(),
        })
        .unwrap();
    assert_eq!(fresh["entity"], "Member");
    assert_eq!(
        reopened.snapshot().unwrap()["staffAttendance"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn gender_is_versioned_and_invalid_or_stale_changes_roll_back_entire_member_edit() {
    let mut f = Fixture::new();
    assert_eq!(
        f.store.snapshot().unwrap()["members"][0]["gender"],
        "Female"
    );
    let edit = |gender: &str, version: Option<i64>| StaffMemberInput {
        member: MemberInput {
            id: Some(f.member.clone()),
            version: Some(1),
            name: "Updated member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "MEMBER-CARD".into(),
        },
        trainer_id: None,
        trainer_version: None,
        assignment_version: None,
        gender: Some(gender.into()),
        gender_version: version,
    };
    let invalid = edit("invalid", Some(1));
    let stale = edit("Male", None);
    let valid = edit("Male", Some(1));
    let before = f.store.snapshot().unwrap();
    assert!(f.store.save_member_with_trainer(invalid).is_err());
    assert!(f
        .store
        .save_member_with_trainer(stale)
        .unwrap_err()
        .contains("gender changed"));
    assert_eq!(f.store.snapshot().unwrap(), before);
    f.store.save_member_with_trainer(valid).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["gender"], "Male");
    assert_eq!(s["members"][0]["genderVersion"], 2);
}

#[test]
fn cards_are_unique_across_staff_and_members_and_sql_rejects_forged_staff_card_events() {
    let mut f = Fixture::new();
    let before = f.store.snapshot().unwrap();
    let tx = f.store.conn.transaction().unwrap();
    assert!(assign_staff_card(&tx, &f.staff, Some("MEMBER-CARD"))
        .unwrap_err()
        .contains("already assigned"));
    drop(tx);
    assert!(f
        .store
        .save_member(MemberInput {
            id: None,
            version: None,
            name: "Other member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "STAFF-CARD".into()
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    assert!(f.store.conn.execute("INSERT INTO staff_attendance VALUES(?1,?2,(SELECT id FROM staff_nfc_cards LIMIT 1),'Staff','WRONG','Check-in','NFC','2026-10-08','2026-10-08T00:00:00Z')",params![id(),f.staff]).is_err());
    f.store
        .conn
        .execute(
            "UPDATE trainers SET active=0,version=2 WHERE id=?1",
            [&f.staff],
        )
        .unwrap();
    assert!(f
        .store
        .record_staff_attendance(f.scan())
        .unwrap_err()
        .contains("active staff"));
}

#[test]
fn attendance_envelopes_match_server_and_backup_retains_profiles_cards_and_both_histories() {
    let mut f = Fixture::new();
    f.store
        .staff_attendance_at(f.scan(), Fixture::now())
        .unwrap();
    f.store
        .attendance_at(
            AttendanceInput {
                request_id: id(),
                member_or_card: "MEMBER-CARD".into(),
                source: "NFC".into(),
            },
            Fixture::now(),
        )
        .unwrap();
    let backup = f.store.backup_envelope().unwrap();
    let preview = f.store.preview_restore(backup).unwrap();
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["gender"], "Female");
    assert_eq!(s["trainers"][0]["nfcId"], "STAFF-CARD");
    assert_eq!(s["staffAttendance"].as_array().unwrap().len(), 1);
    assert_eq!(s["attendance"].as_array().unwrap().len(), 1);
    if let Ok(path) = std::env::var("ARMSTRONG_ATTENDANCE_FIXTURE_PATH") {
        let batches = rows(
            &f.store.conn,
            "SELECT json(request_json) FROM business_batches ORDER BY ordinal",
        )
        .unwrap();
        std::fs::write(path, serde_json::to_vec(&batches).unwrap()).unwrap();
    }
}
