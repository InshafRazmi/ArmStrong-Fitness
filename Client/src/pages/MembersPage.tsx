import { useEffect, useMemo, useRef, useState } from "react";
import { Icon } from "../components/ui/Icon";
import { Modal, PageHeader } from "../components/ui/Modal";
import { useGym } from "../context/GymContext";
import type { Member } from "../types/domain";
import { desktopMembers } from '../desktop/adapter';
import { MemberRemovalDialog } from '../desktop/MemberRemovalDialog';
import { MembershipDatesModal } from '../desktop/MembershipDatesModal';
import { displayDate, money } from "../utils/format";
import { colomboToday, membershipDaysRemaining, membershipEndDate } from '../utils/membership';

const blank = {
       name: "",
       phone: "",
       email: "",
       plan: "No membership",
       expiry: "",
       planId: "",
       planVersion: null as number | null,
       durationMonths: 0,
       startsOn: "",
       nfcId: "",
};

function RemainingDays({ member, today }: { member: Member; today: string }) {
       const days = member.active === false ? null : membershipDaysRemaining(member.expiry, today, member.membershipStartsOn);
       return <div className={`membership-days ${days === null ? 'unassigned' : days === 0 ? 'expired' : member.status === 'Scheduled' ? 'scheduled' : days <= 8 ? 'expiring' : ''}`}>
              <b>{days === null ? '—' : `${days} day${days === 1 ? '' : 's'}`}</b>
              {member.status === 'Scheduled' && member.membershipStartsOn
                     ? <small>Starts {displayDate(member.membershipStartsOn)}</small>
                     : days === 1 && member.expiry === today ? <small>Expires today</small> : null}
       </div>;
}

export function MembersPage() {
       const { data, addMember, updateMember, notify, desktop } = useGym();
       const [browserToday, setBrowserToday] = useState(colomboToday);
       const today = desktop?.snapshot?.today ?? browserToday;
       useEffect(() => {
              if (desktop) return;
              const timer = setInterval(() => setBrowserToday(colomboToday()), 60_000);
              return () => clearInterval(timer);
       }, [Boolean(desktop)]);
       const plans = data.plans.filter(plan => plan.status === 'Active');
       const requestId = useRef('');
       const [showArchived, setShowArchived] = useState(false);
       const [removal, setRemoval] = useState<{ member: Member; kind: 'archive' | 'delete' } | null>(null);
       const memberRows = desktop && showArchived ? desktopMembers(desktop.snapshot!, true).filter(member => !member.active) : data.members;
       const [periodMember, setPeriodMember] = useState<Member | null>(null);
       const [busy, setBusy] = useState(false);
       const [error, setError] = useState("");
       const [q, setQ] = useState("");
       const [editing, setEditing] = useState<Member | null>(null);
       const [adding, setAdding] = useState(false);
       const [form, setForm] = useState(blank);
       const expiry = editing ? form.expiry : form.planId ? membershipEndDate(form.startsOn, form.durationMonths) : '';
       const openAdd = () => {
              const plan = plans[0];
              requestId.current = crypto.randomUUID();
              setError(''); setEditing(null); setScanning(false); setScanComplete(false);
              setForm({ ...blank, planId: plan?.id ?? '', planVersion: plan?.version ?? null,
                     plan: plan?.name ?? 'No membership', durationMonths: plan?.durationMonths ?? 0, startsOn: today });
              setAdding(true);
       };
       const [scanning, setScanning] = useState(false);
       const [scanComplete, setScanComplete] = useState(false);
       const nfcRef = useRef<HTMLInputElement>(null);
       const rows = useMemo(
              () =>
                     memberRows.filter((m) =>
                            (m.name + m.id + m.phone + m.plan)
                                   .toLowerCase()
                                   .includes(q.toLowerCase()),
                     ),
              [q, memberRows],
       );
       const close = () => {
              if (busy) return;
              setError("");
              setAdding(false);
              setEditing(null);
              setForm(blank);
              setScanning(false);
              setScanComplete(false);
       };
       const nfcIsDuplicate = (value: string) =>
              data.members.some(
                     (member) =>
                            member.nfcId &&
                            member.nfcId.toLowerCase() ===
                                   value.trim().toLowerCase() &&
                            member.id !== editing?.id,
              );
       const submit = async (event: React.FormEvent) => {
              event.preventDefault();
              if (busy) return;
              const card = form.nfcId.trim();
              if (card && nfcIsDuplicate(card)) {
                     setError("This NFC card is already assigned to another member");
                     nfcRef.current?.focus();
                     return;
              }
              if (!editing && form.planId && !expiry) {
                     setError('Choose a valid start date within the supported membership date range.');
                     return;
              }
              const clean = { name: form.name, phone: form.phone, email: form.email, plan: form.plan, expiry, nfcId: card };
              setBusy(true); setError("");
              try {
                     if (editing) await updateMember({ ...editing, ...clean });
                     else await addMember({ ...clean, requestId: requestId.current, planId: form.planId || null,
                            planVersion: form.planId ? form.planVersion : null, startsOn: form.planId ? form.startsOn : null });
                     setAdding(false); setEditing(null); setForm(blank); setScanning(false); setScanComplete(false);
              } catch (error) { setError(error instanceof Error ? error.message : String(error)); }
              finally { setBusy(false); }
       };
       const openEdit = (member: Member) => {
              setError("");
              setEditing(member);
              setForm({
                     ...blank,
                     name: member.name,
                     phone: member.phone,
                     email: member.email,
                     plan: member.plan,
                     expiry: member.expiry,
                     nfcId: member.nfcId,
              });
              setScanComplete(false);
       };
       const startScan = () => {
              setScanning(true);
              setScanComplete(false);
              setForm({ ...form, nfcId: "" });
              requestAnimationFrame(() => nfcRef.current?.focus());
       };
       const finishScan = () => {
              const card = form.nfcId.trim();
              if (!card) return;
              if (nfcIsDuplicate(card)) {
                     setError("This NFC card is already assigned to another member");
                     setForm({ ...form, nfcId: "" });
                     return;
              }
              setScanning(false);
              setScanComplete(true);
              notify(`NFC card ${card} captured`);
       };

       return (
              <>
                     <PageHeader
                            title="Members"
                            subtitle="Register, search and manage member records"
                            action="Add member"
                            onAction={openAdd}
                     />
                     {desktop && <p className="form-note">{desktop.snapshot!.removalAuthorization.allowed ? 'Removal is restricted to the authenticated Administrator.' : desktop.snapshot!.removalAuthorization.reason}</p>}
                     <div className="toolbar">
                            {desktop && <button className="secondary" onClick={() => setShowArchived(!showArchived)}>{showArchived ? 'Show active members' : 'Show archived members'}</button>}
                            <label className="search grow">
                                   <Icon name="search" size={17} />
                                   <input
                                          value={q}
                                          onChange={(e) => setQ(e.target.value)}
                                          placeholder="Search name, ID, phone or plan"
                                   />
                            </label>
                     </div>
                     <div className="card table-card members-table">
                            <table>
                                   <thead>
                                          <tr>
                                                 <th>Member</th>
                                                 <th>Phone</th>
                                                 <th>NFC card</th>
                                                 <th>Membership</th>
                                                 <th>Expiry</th>
                                                 <th>Remaining days</th>
                                                 <th>Status</th>
                                                 <th></th>
                                          </tr>
                                   </thead>
                                   <tbody>
                                          {rows.map((m) => (
                                                 <tr key={m.id}>
                                                        <td>
                                                               <div className="person">
                                                                      <span className="avatar tiny">
                                                                             {
                                                                                    m.initials
                                                                             }
                                                                      </span>
                                                                      <span>
                                                                             <b>
                                                                                    {
                                                                                           m.name
                                                                                    }
                                                                             </b>
                                                                             <small>
                                                                                    {
                                                                                           m.id
                                                                                    }
                                                                             </small>
                                                                      </span>
                                                               </div>
                                                        </td>
                                                        <td>{m.phone}</td>
                                                        <td>
                                                               {m.nfcId ||
                                                                      "Not linked"}
                                                        </td>
                                                        <td>{m.plan}</td>
                                                        <td>
                                                               {displayDate(
                                                                      m.expiry,
                                                               )}
                                                        </td>
                                                        <td><RemainingDays member={m} today={today}/></td>
                                                        <td>
                                                               <span
                                                                      className={
                                                                             m.status ===
                                                                             "Active"
                                                                                    ? "tag green"
                                                                                    : m.status === "Expired" ? "tag red" : "tag amber"
                                                                      }
                                                               >
                                                                      {m.status}
                                                               </span>
                                                        </td>
                                                        <td>
                                                               <button
                                                                      className="secondary compact"
                                                                      disabled={m.active === false}
                                                                      onClick={() =>
                                                                             openEdit(
                                                                                    m,
                                                                             )
                                                                      }
                                                               >
                                                                      Edit
                                                               </button>
                                                               {desktop && <><button className="secondary compact" onClick={() => setPeriodMember(m)}>{m.active === false ? 'Membership history' : 'Membership dates'}</button>
                                                               {m.active !== false && <button className="secondary compact" onClick={() => setRemoval({ member: m, kind: 'archive' })}>Archive / Deactivate</button>}
                                                               {m.canDelete && <button className="secondary compact" onClick={() => setRemoval({ member: m, kind: 'delete' })}>Delete permanently</button>}
                                                               </>}
                                                        </td>
                                                 </tr>
                                          ))}
                                   </tbody>
                            </table>
                            {!rows.length && <p className="foundation-empty">{q ? "No matching members." : showArchived ? "No archived members." : "No active members recorded. Add your first member."}</p>}
                     </div>
                     {removal && <MemberRemovalDialog member={removal.member} kind={removal.kind} onClose={() => setRemoval(null)}/>}
                     {(adding || editing) && (
                            <Modal
                                   title={
                                          editing
                                                 ? "Edit member"
                                                 : "Register new member"
                                   }
                                   onClose={close}
                            >
                                   <form
                                          className="modal-form"
                                          onSubmit={event => void submit(event)}
                                   ><fieldset className="foundation-fields" disabled={busy}>
                                          <label>
                                                 <span>Full name</span>
                                                 <input
                                                        required
                                                        value={form.name}
                                                        onChange={(e) =>
                                                               setForm({
                                                                      ...form,
                                                                      name: e
                                                                             .target
                                                                             .value,
                                                               })
                                                        }
                                                 />
                                          </label>
                                          <label>
                                                 <span>Phone</span>
                                                 <input
                                                        required
                                                        value={form.phone}
                                                        onChange={(e) =>
                                                               setForm({
                                                                      ...form,
                                                                      phone: e
                                                                             .target
                                                                             .value,
                                                               })
                                                        }
                                                 />
                                          </label>
                                          <label>
                                                 <span>Email</span>
                                                 <input
                                                        type="email"
                                                        value={form.email}
                                                        onChange={(e) =>
                                                               setForm({
                                                                      ...form,
                                                                      email: e
                                                                             .target
                                                                             .value,
                                                               })
                                                        }
                                                 />
                                          </label>
                                          {adding && <>
                                                 <label className="membership-package-field">
                                                        <span>Membership package</span>
                                                        <select value={form.planId} onChange={event => {
                                                               const plan = plans.find(value => value.id === event.target.value);
                                                               setForm({ ...form, planId: plan?.id ?? '', planVersion: plan?.version ?? null,
                                                                      plan: plan?.name ?? 'No membership', durationMonths: plan?.durationMonths ?? 0 });
                                                        }}>
                                                               <option value="">No membership — details only</option>
                                                               {plans.map(plan => <option key={plan.id} value={plan.id}>{plan.name} · {plan.durationMonths} month{plan.durationMonths === 1 ? '' : 's'} · {money(plan.price)}</option>)}
                                                        </select>
                                                 </label>
                                                 {form.planId ? <>
                                                        <label><span>Start date</span><input required type="date" min="1900-01-01" max="2200-12-31" value={form.startsOn} onChange={event => setForm({ ...form, startsOn: event.target.value })}/></label>
                                                        <label><span>Expiry (automatic)</span><input type="date" value={expiry} readOnly aria-describedby="membership-date-note"/></label>
                                                        <p className="form-note" id="membership-date-note">{form.durationMonths} month{form.durationMonths === 1 ? '' : 's'} from the start date. Membership is valid through the expiry date.</p>
                                                 </> : <p className="form-note">{plans.length ? 'Select a package to set membership dates automatically.' : 'Add an active package in Memberships to register with membership dates.'}</p>}
                                          </>}
                                          {editing && !desktop && <label><span>Expiry</span><input type="date" value={form.expiry} onChange={event => setForm({ ...form, expiry: event.target.value })}/></label>}
                                          <label className="nfc-form-field">
                                                 <span>
                                                        NFC card ID — scan or
                                                        enter manually
                                                 </span>
                                                 <div
                                                        className={`nfc-input-row ${scanning ? "scanning" : ""}`}
                                                 >
                                                        <input
                                                               ref={nfcRef}
                                                               value={
                                                                      form.nfcId
                                                               }
                                                               onChange={(
                                                                      e,
                                                               ) => {
                                                                      setForm({
                                                                             ...form,
                                                                             nfcId: e
                                                                                    .target
                                                                                    .value,
                                                                      });
                                                                      setScanComplete(
                                                                             false,
                                                                      );
                                                               }}
                                                               onKeyDown={(
                                                                      e,
                                                               ) => {
                                                                      if (
                                                                             e.key ===
                                                                                    "Enter" &&
                                                                             scanning
                                                                      ) {
                                                                             e.preventDefault();
                                                                             e.stopPropagation();
                                                                             finishScan();
                                                                      }
                                                               }}
                                                               placeholder={
                                                                      scanning
                                                                             ? "Waiting for NFC card scan…"
                                                                             : "Enter card ID manually"
                                                               }
                                                        />
                                                        <button
                                                               type="button"
                                                               className="secondary scan-button"
                                                               onClick={
                                                                      startScan
                                                               }
                                                        >
                                                               <Icon
                                                                      name="signal"
                                                                      size={16}
                                                               />
                                                               {scanning
                                                                      ? "Waiting…"
                                                                      : "Scan card"}
                                                        </button>
                                                 </div>
                                                 <small
                                                        className={
                                                               scanComplete
                                                                      ? "scan-message success"
                                                                      : "scan-message"
                                                        }
                                                 >
                                                        {scanning
                                                               ? "Tap the card on the reader. The UID will be captured automatically."
                                                               : scanComplete
                                                                 ? "✓ Card captured successfully"
                                                                 : "Manual entry remains available."}
                                                 </small>
                                          </label>
                                          {editing && !desktop && <label><span>Plan</span><select value={form.plan} onChange={event => setForm({ ...form, plan: event.target.value })}>{plans.map(plan => <option key={plan.id}>{plan.name}</option>)}</select></label>}
                                          {desktop && <p className="form-note">{editing ? 'Editing member details keeps the recorded membership dates. Use Membership dates or Renew membership to change the membership.' : 'The member and selected membership are saved together. Record invoices and payments in Payments.'}</p>}
                                          {error && <div className="login-error" role="alert">{error}</div>}
                                          <button
                                                 className="primary"
                                                 type="submit"
                                          >
                                                 {busy ? "Saving…" : "Save member"}
                                          </button>
                                   </fieldset></form>
                            </Modal>
                     )}
                     {periodMember && desktop?.snapshot && <MembershipDatesModal member={periodMember} onClose={() => setPeriodMember(null)} />}
              </>
       );
}
