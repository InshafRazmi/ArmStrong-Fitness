import { useState } from "react";
import { PageHeader } from "../components/ui/Modal";
import { useGym } from "../context/GymContext";
import { DesktopSettingsPanel } from "../desktop/DesktopSettingsPanel";
export function SettingsPage() {
       const { data, online, syncing, syncNow, restore, notify, mode } = useGym();
       const [fileRef, setFileRef] = useState<HTMLInputElement | null>(null);
       const [tab, setTab] = useState("Gym profile");
       const items = [
              "Gym profile",
              "Users & roles",
              "NFC reader",
              "Receipt printing",
              "Backup & restore",
              "Server synchronization",
              "Application updates",
       ];
       return (
              <>
                     <PageHeader
                            title="Settings"
                            subtitle="Security, hardware, backup, synchronization and updates"
                     />
                     <div className="settings-grid">
                            <section className="card settings-nav">
                                   {items.map((x) => (
                                          <button
                                                 className={
                                                        tab === x
                                                               ? "selected"
                                                               : ""
                                                 }
                                                 onClick={() => setTab(x)}
                                                 key={x}
                                          >
                                                 {x}
                                          </button>
                                   ))}
                            </section>
                            <section className="card form-card">
                                   <h2>{tab}</h2>
                                   {mode === "desktop" ? <DesktopSettingsPanel tab={tab}/> : <>
                                   {tab === "Gym profile" && (
                                          <>
                                                 <p>
                                                        Information used on
                                                        receipts and reports.
                                                 </p>
                                                 <div className="form-grid">
                                                        <label>
                                                               <span>
                                                                      Gym name
                                                               </span>
                                                               <input defaultValue="Armstrong Fitness" />
                                                        </label>
                                                        <label>
                                                               <span>
                                                                      Location
                                                               </span>
                                                               <input defaultValue="Matale, Sri Lanka" />
                                                        </label>
                                                        <label>
                                                               <span>
                                                                      Phone
                                                               </span>
                                                               <input placeholder="Enter phone" />
                                                        </label>
                                                        <label>
                                                               <span>
                                                                      Email
                                                               </span>
                                                               <input placeholder="Enter email" />
                                                        </label>
                                                 </div>
                                                 <button
                                                        className="primary"
                                                        onClick={() =>
                                                               notify(
                                                                      "Gym profile saved",
                                                               )
                                                        }
                                                 >
                                                        Save changes
                                                 </button>
                                          </>
                                   )}
                                   {tab === "Users & roles" && (
                                          <>
                                                 <p>
                                                        Role-based access
                                                        protects sensitive
                                                        operations.
                                                 </p>
                                                 <div className="setting-line">
                                                        <div>
                                                               <b>Prinzz</b>
                                                               <small>
                                                                      Administrator
                                                                      · Full
                                                                      access
                                                               </small>
                                                        </div>
                                                        <span className="tag green">
                                                               Active
                                                        </span>
                                                 </div>
                                                 <div className="setting-line">
                                                        <div>
                                                               <b>Reception</b>
                                                               <small>
                                                                      Members,
                                                                      attendance
                                                                      and
                                                                      payments
                                                               </small>
                                                        </div>
                                                        <span className="tag green">
                                                               Active
                                                        </span>
                                                 </div>
                                          </>
                                   )}
                                   {tab === "NFC reader" && (
                                          <>
                                                 <p>
                                                        Use a USB HID NFC reader
                                                        that types the card ID
                                                        and presses Enter.
                                                 </p>
                                                 <div className="setting-line">
                                                        <div>
                                                               <b>
                                                                      Reader
                                                                      mode
                                                               </b>
                                                               <small>
                                                                      Keyboard /
                                                                      HID input
                                                               </small>
                                                        </div>
                                                        <span className="tag green">
                                                               Ready
                                                        </span>
                                                 </div>
                                                 <button
                                                        className="primary"
                                                        onClick={() =>
                                                               notify(
                                                                      "Reader test started. Scan a card on Attendance.",
                                                                      "info",
                                                               )
                                                        }
                                                 >
                                                        Test reader
                                                 </button>
                                          </>
                                   )}
                                   {tab === "Receipt printing" && (
                                          <>
                                                 <p>
                                                        Receipt printing will
                                                        use the Windows default
                                                        thermal printer.
                                                 </p>
                                                 <div className="setting-line">
                                                        <div>
                                                               <b>Paper size</b>
                                                               <small>
                                                                      80 mm
                                                                      thermal
                                                                      receipt
                                                               </small>
                                                        </div>
                                                        <span>Default</span>
                                                 </div>
                                          </>
                                   )}
                                   {tab === "Backup & restore" && (
                                          <>
                                                 <p>
                                                        Export all local records
                                                        or restore a previously
                                                        exported JSON backup.
                                                 </p>
                                                 <div className="settings-actions">
                                                        <button
                                                               className="primary"
                                                               onClick={() =>
                                                                      void import("../services/storage").then(({exportBackup}) => exportBackup(data))
                                                               }
                                                        >
                                                               Export backup
                                                        </button>
                                                        <button
                                                               className="secondary"
                                                               onClick={() =>
                                                                      fileRef?.click()
                                                               }
                                                        >
                                                               Restore backup
                                                        </button>
                                                        <input
                                                               hidden
                                                               ref={setFileRef}
                                                               type="file"
                                                               accept=".json"
                                                               onChange={async (
                                                                      e,
                                                               ) => {
                                                                      const f =
                                                                             e
                                                                                    .target
                                                                                    .files?.[0];
                                                                      if (!f)
                                                                             return;
                                                                      try {
                                                                             const { readBackup } = await import("../services/storage");
                                                                             restore(await readBackup(f));
                                                                             notify("Backup restored successfully");
                                                                      } catch {
                                                                             notify(
                                                                                    "Invalid backup file",
                                                                                    "error",
                                                                             );
                                                                      }
                                                               }}
                                                        />
                                                 </div>
                                          </>
                                   )}
                                   {tab === "Server synchronization" && (
                                          <>
                                                 <p>
                                                        Every change saves
                                                        locally first. Pending
                                                        operations upload when
                                                        internet returns.
                                                 </p>
                                                 <div className="summary-grid">
                                                        <div>
                                                               <small>
                                                                      Connection
                                                               </small>
                                                               <strong>
                                                                      {online
                                                                             ? "Online"
                                                                             : "Offline"}
                                                               </strong>
                                                        </div>
                                                        <div>
                                                               <small>
                                                                      Pending
                                                                      changes
                                                               </small>
                                                               <strong>
                                                                      {
                                                                             data
                                                                                    .queue
                                                                                    .length
                                                                      }
                                                               </strong>
                                                        </div>
                                                 </div>
                                                 <button
                                                        className="primary"
                                                        disabled={syncing}
                                                        onClick={() =>
                                                               void syncNow()
                                                        }
                                                 >
                                                        {syncing
                                                               ? "Synchronizing..."
                                                               : "Sync now"}
                                                 </button>
                                          </>
                                   )}
                                   {tab === "Application updates" && (
                                          <>
                                                 <p>
                                                        Desktop releases can be
                                                        checked against the
                                                        update server.
                                                 </p>
                                                 <div className="setting-line">
                                                        <div>
                                                               <b>
                                                                      Installed
                                                                      version
                                                               </b>
                                                               <small>
                                                                      Frontend
                                                                      architecture
                                                                      v0.4
                                                               </small>
                                                        </div>
                                                        <span className="tag green">
                                                               Current
                                                        </span>
                                                 </div>
                                                 <button
                                                        className="primary"
                                                        onClick={() =>
                                                               notify(
                                                                      "No new update found",
                                                                      "info",
                                                               )
                                                        }
                                                 >
                                                        Check for updates
                                                 </button>
                                          </>
                                   )}
                                   </>}
                            </section>
                     </div>
              </>
       );
}
