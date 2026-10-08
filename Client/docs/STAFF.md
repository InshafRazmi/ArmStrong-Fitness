# Staff and personal training

Add a staff member from **Staff → Add staff** with their full name, mobile,
NIC, fixed monthly salary and personal-training charge per member per month.
NIC is required and unique. The list masks it; editing shows the saved value.
Amounts use exact LKR minor units. Changing a rate affects future invoices and
payouts; earlier financial amounts remain recorded at their original values.
These profiles do not create application login accounts.

When adding a member, select **Personal trainer → None** or an active staff
member. With a trainer selected, registration saves the member, membership,
assignment and first monthly training invoice together. A paid membership also
gets its membership invoice in that transaction. Training starts on the selected
membership start date, or today when no membership is selected. A zero training
fee creates no training invoice. Selecting None preserves the existing member
registration and billing workflow.

Edit a member to change or remove their trainer. Existing invoices keep the
trainer and rate that applied when they were issued. Inactive staff cannot receive
new assignments or training invoices; existing earnings remain payable.

For another month or a member assigned after registration, use **Payments →
Training invoice**. Review the start date and rate before saving. It bills one
calendar month, inclusive of the last day. The suggested date follows the last
training invoice; without prior training, it uses the current or next membership
start, or today. Overlapping training dates cannot be billed twice. Monthly
invoices are created explicitly; no background billing is scheduled.

**Receive payment** selects outstanding membership and training invoices and
shows their total. The gym receives one payment and produces one saved receipt.
The selected invoices are settled in the order shown, membership first, then
training. Partial payment pays only that amount; excess stays as member credit.
Credit can later be allocated to a training invoice without recording cash again.

Use **Staff → Pay staff** after paying the person. Review the salary month,
fixed salary, collected unpaid training fees and total. All fees collected and
allocated to that trainer's invoices since the previous payout are included;
unpaid member invoices are excluded. Fixed salary can be paid once per calendar
month. Later fee collections can be paid as a training-only top-up. The payout
and matching Salary expense are recorded together and appear in expense reports.
No commission, deductions or salary proration are applied.

Correct an erroneous payout by voiding its linked Salary expense through
Expenses with Administrator authorization. History remains visible. A member
payment whose training earnings have already been paid cannot be reversed until
the associated payout is voided. Voiding restores the unpaid earnings; reversing
the collection removes those earnings again and reopens its invoices.

Staff NFC attendance is separate from member attendance. Assign an optional
unique card from **Staff → Add staff / Edit staff → NFC attendance card**.
**Scan card** focuses the keyboard/HID reader input; entering the UID works too.
An active card cannot be shared with another member or staff person. Clearing
or changing it revokes the old assignment while retaining its attendance history.

Open **NFC Attendance** to record a card. The card owner determines whether a
member or staff event is recorded, and the activity view switches to that group.
The **Staff** tab also offers manual attendance for active staff. Events alternate
Check-in / Check-out each Asia/Colombo business day. Repeated NFC scans within
two seconds are ignored. Retrying the same operation never creates a second
attendance event, including after card reassignment. Staff cards do not grant
application sign-in access. Physical reader acceptance remains manual.

The dashboard displays separate member and staff activity panels. Daily totals
count each person once after an effective check-in, including repeat visits.
Member Male / Female totals exclude unspecified profiles; an Unspecified count
keeps earlier members visible until their gender is recorded through Edit.
Add/Edit member asks for Male or Female. Existing financial/history rows keep
their identities when profile details are changed.

**Staff → Delete staff** requires an authenticated Administrator and the checked
confirmation. The action is available for active and inactive staff. It removes
the profile from both lists, revokes its active NFC
card and clears current member/trainer assignments in one transaction. Existing
training invoices, salary payments, receipts, attendance and unpaid earnings
retain the staff identity. **Show deleted staff** opens those retained profiles
for history and final earnings payments. Deleted profiles cannot be edited or
reactivated. Use **Edit → Inactive** when a reversible status change is wanted.
Deletion does not erase
salary or invoice evidence. An expired session, stale profile or failed audit/
queue write refuses the operation without changing records.

**Members → Delete permanently** is available to an authenticated Administrator
for active and archived members. It removes the member from both operational
lists, archives the historical identity and revokes its active NFC assignment.
Payments, receipts, membership periods, training charges, attendance and audit
remain for reports and shared history. It does not erase financial evidence.

The NFC reader panel uses a larger amber card and scan controls. Scanning still
uses the existing keyboard/HID input and native owner detection.

Native schema 10 includes staff removal markers in validated backups, restore and transactional
sync. Before using Staff across computers, apply server migrations 6–8 and deploy
the updated API; upgrade all gym desktops. Older clients refuse unknown Staff
rows while retaining their queues. Production deployment is separate from the
isolated tests completed for this update. See [delivery](DELIVERY.md).
