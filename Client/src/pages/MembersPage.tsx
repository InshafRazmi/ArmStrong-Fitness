import { useMemo, useRef, useState } from "react";
import { Icon } from "../components/ui/Icon";
import { Modal, PageHeader } from "../components/ui/Modal";
import { useGym } from "../context/GymContext";
import type { Member } from "../types/domain";
import { desktopMembers } from '../desktop/adapter';
import { MemberRemovalDialog } from '../desktop/MemberRemovalDialog';
import { MembershipDatesModal } from '../desktop/MembershipDatesModal';
import { displayDate } from "../utils/format";

const blank = {
       name: "",
       phone: "",
       email: "",
       plan: "Gold",
       expiry: "2027-01-01",
       nfcId: "",
};

export function MembersPage() {
       const { data, addMember, updateMember, notify, desktop } = useGym();
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
              const clean = { ...form, nfcId: card };
              setBusy(true); setError("");
              try {
                     if (editing) await updateMember({ ...editing, ...clean });
                     else await addMember(clean);
                     setAdding(false); setEditing(null); setForm(blank); setScanning(false); setScanComplete(false);
              } catch (error) { setError(error instanceof Error ? error.message : String(error)); }
              finally { setBusy(false); }
       };
       const openEdit = (member: Member) => {
              setError("");
              setEditing(member);
              setForm({
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
                            onAction={() => { setError(""); setForm(blank); setAdding(true); }}
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
                     <div className="card table-card">
                            <table>
                                   <thead>
                                          <tr>
                                                 <th>Member</th>
                                                 <th>Phone</th>
                                                 <th>NFC card</th>
                                                 <th>Membership</th>
                                                 <th>Expiry</th>
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
                                                        <td>
                                                               <span
                                                                      className={
                                                                             m.status ===
                                                                             "Active"
                                                                                    ? "tag green"
                                                                                    : "tag amber"
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
                                          {!desktop && <label>
                                                 <span>Expiry</span>
                                                 <input
                                                        type="date"
                                                        value={form.expiry}
                                                        onChange={(e) =>
                                                               setForm({
                                                                      ...form,
                                                                      expiry: e
                                                                             .target
                                                                             .value,
                                                               })
                                                        }
                                                 />
                                          </label>}
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
                                          {!desktop && <label>
                                                 <span>Plan</span>
                                                 <select
                                                        value={form.plan}
                                                        onChange={(e) =>
                                                               setForm({
                                                                      ...form,
                                                                      plan: e
                                                                             .target
                                                                             .value,
                                                               })
                                                        }
                                                 >
                                                        {data.plans
                                                               .filter(
                                                                      (p) =>
                                                                             p.status ===
                                                                             "Active",
                                                               )
                                                               .map((p) => (
                                                                      <option
                                                                             key={
                                                                                    p.id
                                                                             }
                                                                      >
                                                                             {
                                                                                    p.name
                                                                             }
                                                                      </option>
                                                               ))}
                                                 </select>
                                          </label>}
                                          {desktop && <p className="form-note">This saves member details only. Use Membership dates to record a plan and explicit start/end dates.</p>}
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
