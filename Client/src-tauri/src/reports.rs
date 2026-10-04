use super::*;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReportRange {
    pub from_on: Option<String>,
    pub to_on: Option<String>,
}
impl ReportRange {
    fn check(&self) -> Result<()> {
        for day in [&self.from_on, &self.to_on].into_iter().flatten() {
            date(day)?;
        }
        if self
            .from_on
            .as_ref()
            .zip(self.to_on.as_ref())
            .is_some_and(|(from, to)| from > to)
        {
            return Err("Report start date must be on or before its end date".into());
        }
        Ok(())
    }
    fn includes(&self, day: &str) -> bool {
        self.from_on
            .as_ref()
            .is_none_or(|from| day >= from.as_str())
            && self.to_on.as_ref().is_none_or(|to| day <= to.as_str())
    }
    fn overlaps(&self, starts: &str, ends: &str) -> bool {
        self.from_on
            .as_ref()
            .is_none_or(|from| ends >= from.as_str())
            && self.to_on.as_ref().is_none_or(|to| starts <= to.as_str())
    }
}
// A consistent native snapshot supplies both the summaries and export rows.
// Financial reversals stay on their own posted business date. Memberships use
// inclusive interval overlap; inventory is explicitly the current stock snapshot.
pub(super) fn filter_snapshot(snapshot: &mut Value, range: &ReportRange) -> Result<()> {
    range.check()?;
    if range.from_on.is_none() && range.to_on.is_none() {
        return Ok(());
    }
    for table in ["attendance", "payments", "sales", "expenses"] {
        snapshot[table]
            .as_array_mut()
            .ok_or("Invalid report snapshot")?
            .retain(|row| range.includes(row["businessOn"].as_str().unwrap_or("")));
    }
    snapshot["periods"]
        .as_array_mut()
        .ok_or("Invalid report snapshot")?
        .retain(|row| {
            range.overlaps(
                row["startsOn"].as_str().unwrap_or(""),
                row["endsOn"].as_str().unwrap_or(""),
            )
        });
    let mut filtered_audit = vec![];
    for row in snapshot["audit"]
        .as_array()
        .ok_or("Invalid audit report snapshot")?
    {
        let instant = DateTime::parse_from_rfc3339(row["timestamp"].as_str().unwrap_or(""))
            .map_err(|_| "Invalid saved audit timestamp; keep records for recovery")?;
        if range.includes(&business_date(instant.with_timezone(&Utc))) {
            filtered_audit.push(row.clone());
        }
    }
    snapshot["audit"] = json!(filtered_audit);
    Ok(())
}
const MAX_SAFE_MINOR: i64 = 9_007_199_254_740_991;
fn add(sum: i64, value: i64) -> Result<i64> {
    let total = sum
        .checked_add(value)
        .ok_or("Report amount exceeds the supported exact range")?;
    if !(-MAX_SAFE_MINOR..=MAX_SAFE_MINOR).contains(&total) {
        return Err("Report amount exceeds the supported exact range".into());
    }
    Ok(total)
}
fn amount(row: &Value, field: &str) -> Result<i64> {
    row[field]
        .as_i64()
        .ok_or_else(|| "Invalid saved report amount; keep records for recovery".into())
}
fn sum(snapshot: &Value, table: &str, field: &str) -> Result<i64> {
    snapshot[table]
        .as_array()
        .ok_or("Invalid report snapshot")?
        .iter()
        .try_fold(0, |total, row| add(total, amount(row, field)?))
}
impl Store {
    pub fn report_summary(&self, range: ReportRange) -> Result<Value> {
        let mut snapshot = self.snapshot()?;
        filter_snapshot(&mut snapshot, &range)?;
        let income = add(
            sum(&snapshot, "payments", "netAmountMinor")?,
            sum(&snapshot, "sales", "totalMinor")?,
        )?;
        let expenses = sum(&snapshot, "expenses", "effectiveAmountMinor")?;
        let products = snapshot["products"]
            .as_array()
            .ok_or("Invalid report snapshot")?;
        let inventory = products.iter().try_fold(0, |total, row| {
            let value = amount(row, "costMinor")?
                .checked_mul(amount(row, "stock")?)
                .ok_or("Inventory value exceeds the supported exact range")?;
            add(total, value)
        })?;
        Ok(
            json!({"fromOn":range.from_on,"toOn":range.to_on,"incomeMinor":income,"expenseMinor":expenses,
            "netMinor":add(income,-expenses)?,"inventoryValueMinor":inventory,
            "attendanceCount":snapshot["attendance"].as_array().unwrap().len(),
            "membershipPeriodCount":snapshot["periods"].as_array().unwrap().len(),
            "auditCount":snapshot["audit"].as_array().unwrap().len(),
            "lowStockCount":products.iter().filter(|row| row["stock"].as_i64() <= row["reorderLevel"].as_i64()).count()}),
        )
    }
}
