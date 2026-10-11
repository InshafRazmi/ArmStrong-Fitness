import { useEffect, useMemo, useRef, useState } from "react";
import { Icon } from "../components/ui/Icon";
import { Modal, PageHeader } from "../components/ui/Modal";
import { useGym } from "../context/GymContext";
import type { Member } from "../types/domain";
import { desktopMembers } from '../desktop/adapter';
import { MemberRemovalDialog } from '../desktop/MemberRemovalDialog';
import { MembershipDatesModal } from '../desktop/MembershipDatesModal';
import { DesktopPaymentsPage } from "../desktop/DesktopPaymentsPage";
import { displayDate, money } from "../utils/format";
import { colomboToday, membershipDaysRemaining, membershipEndDate } from '../utils/membership';

const blank = {
       name: "",
       gender: "" as "" | "Male" | "Female",
       phone: "",
       email: "",
       plan: "No membership",
       expiry: "",
       planId: "",
       planVersion: null as number | null,
       durationMonths: 0,
       startsOn: "",
       nfcId: "",
       trainerId: "",
       trainerVersion: null as number | null,
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
       const [receiveFor, setReceiveFor] = useState<string | null>(null);
       const [receiveNow, setReceiveNow] = useState(false);
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
       const [showHistory, setShowHistory] = useState(false);
       const [removal, setRemoval] = useState<Member | null>(null);
       const memberRows = desktop && showHistory ? desktopMembers(desktop.snapshot!, true).filter(member => !member.active) : data.members;
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
              setError(''); setReceiveNow(false); setEditing(null); setScanning(false); setScanComplete(false);
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
              if (!form.gender) { setError('Choose Male or Female.'); return; }
              const clean = { name: form.name, gender: form.gender, phone: form.phone, email: form.email, plan: form.plan, expiry, nfcId: card, trainerId: form.trainerId || null, trainerVersion: form.trainerId ? form.trainerVersion : null };
              setBusy(true); setError("");
              try {
                     if (editing) await updateMember({ ...editing, ...clean });
                     else { const memberId = await addMember({ ...clean, expectedAdmissionMinor: desktop?.snapshot?.profile.admissionMinor ?? 0, requestId: requestId.current, planId: form.planId || null,
                            planVersion: form.planId ? form.planVersion : null, startsOn: form.planId ? form.startsOn : null });
                            if (desktop && receiveNow && memberId) setReceiveFor(memberId);
                     }
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
                     gender: member.gender ?? '',
                     phone: member.phone,
                     email: member.email,
                     plan: member.plan,
                     expiry: member.expiry,
                     nfcId: member.nfcId,
                     trainerId: member.trainerId ?? '',
                     trainerVersion: member.trainerVersion ?? null,
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
                     {desktop && receiveFor && <DesktopPaymentsPage receiveFor={receiveFor} onClose={() => setReceiveFor(null)}/>}
                     <PageHeader
                            title="Members"
                            subtitle="Register, search and manage member records"
                            action="Add member"
                            onAction={openAdd}
                     />
                     {desktop && !desktop.snapshot!.removalAuthorization.allowed && <p className="form-note">{desktop.snapshot!.removalAuthorization.reason}</p>}
                     <div className="toolbar">
                            {desktop && desktop.snapshot!.members.some(member => !member.active) && <button className="secondary" onClick={() => setShowHistory(!showHistory)}>{showHistory ? 'Show current members' : 'Show past members'}</button>}
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
                                                 <th>Gender</th>
                                                 <th>NFC card</th>
                                                 <th>Membership</th>
                                                 <th>Expiry</th>
                                                 <th>Remaining days</th>
                                                 <th>Status</th>
                                                 {desktop && <th>Due to pay</th>}
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
                                                        <td>{m.gender ?? 'Unspecified'}</td>
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
                                                        {desktop && <td><span className={(desktop.snapshot!.financialAccounts.find(a => a.memberId === m.id)?.outstandingMinor ?? 0) > 0 ? 'tag amber' : 'tag green'}>{money((desktop.snapshot!.financialAccounts.find(a => a.memberId === m.id)?.outstandingMinor ?? 0) / 100)}</span></td>}
                                                        <td>
                                                               <div className="member-actions">
                                                               {desktop && m.active !== false && <button className="primary compact" aria-label="Receive payment" title="Receive payment" onClick={() => setReceiveFor(m.id)}><Icon name="money" size={12}/>Receive</button>}
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
                                                               <button className="secondary compact danger-action" onClick={() => setRemoval(m)}>Delete permanently</button>
                                                               </>}
                                                               </div>
                                                        </td>
                                                 </tr>
                                          ))}
                                   </tbody>
                            </table>
                            {!rows.length && <p className="foundation-empty">{q ? "No matching members." : showHistory ? "No past members." : "No members recorded. Add your first member."}</p>}
                     </div>
                     {removal && <MemberRemovalDialog member={removal} onClose={() => setRemoval(null)}/>}
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
                                          <label><span>Gender</span><select required value={form.gender} onChange={event => setForm({...form, gender: event.target.value as typeof form.gender})}><option value="" disabled>Select gender</option><option value="Male">Male</option><option value="Female">Female</option></select></label>
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
                                          {desktop && <label><span>Personal trainer</span><select value={form.trainerId} onChange={event => {
                                                 const trainer = desktop.snapshot!.trainers.find(t => t.id === event.target.value);
                                                 setForm({ ...form, trainerId: trainer?.id ?? '', trainerVersion: trainer?.version ?? null });
                                          }}><option value="">None — no personal training</option>{desktop.snapshot!.trainers.filter(t => t.active || t.id === editing?.trainerId).map(t => <option key={t.id} value={t.id} disabled={!t.active}>{t.name} · {money(t.trainingFeeMinor / 100)} / month{!t.active ? ' (Inactive — select another staff or None)' : ''}</option>)}</select></label>}
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
                                                        <label><span>Expiry (automatic)</span><input type="date" value={expiry} readOnly/></label>
                                                 </> : !plans.length && <p className="form-note">No active membership packages.</p>}
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
                                                 {(scanning || scanComplete) && <small
                                                        className={
                                                               scanComplete
                                                                      ? "scan-message success"
                                                                      : "scan-message"
                                                        }
                                                 >
                                                        {scanning
                                                               ? "Tap the card on the reader."
                                                               : "✓ Card captured"}
                                                 </small>}
                                          </label>
                                          {editing && !desktop && <label><span>Plan</span><select value={form.plan} onChange={event => setForm({ ...form, plan: event.target.value })}>{plans.map(plan => <option key={plan.id}>{plan.name}</option>)}</select></label>}
                                          {desktop && !editing && <><p className="form-note">Membership: {money((plans.find(p => p.id === form.planId)?.price ?? 0))} · Admission: {money((desktop.snapshot!.profile.admissionMinor ?? 0) / 100)}</p><label><span>Payment after registration</span><select value={receiveNow ? 'receive' : 'unpaid'} onChange={event => setReceiveNow(event.target.value === 'receive')}><option value="unpaid">Save as unpaid</option><option value="receive">Save and receive payment</option></select></label></>}
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
