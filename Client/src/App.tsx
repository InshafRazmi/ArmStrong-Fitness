import { useState } from "react";
import { Modal } from "./components/ui/Modal";
import { useGym } from "./context/GymContext";
import { AppLayout } from "./layout/AppLayout";
import { LoginPage } from "./pages/LoginPage";
import { PageContent } from "./pages/navigation";
import type { Member, Page } from "./types/domain";
import { DesktopScreenNotice } from "./desktop/DesktopScreenNotice";
import { displayDate } from "./utils/format";

export default function App() {
       const { mode, desktop, toasts, notify } = useGym();
       const native = import.meta.env.VITE_DESKTOP_ONLY === 'true' || mode === 'desktop';
       const [authenticated, setAuthenticated] = useState(
              () =>
                     native ||
                     sessionStorage.getItem("armstrong-demo-auth") === "true",
       );
       const [page, setPage] = useState<Page>("Dashboard");
       const [selected, setSelected] = useState<Member | null>(null);
       if (!authenticated || (native && desktop?.authStatus?.requiresLogin && !desktop.authStatus.authenticated))
              return <LoginPage onLogin={() => setAuthenticated(true)} />;
       const content = <PageContent page={page} navigate={setPage} />;
       const logout = () => {
              if (native) {
                     setSelected(null);
                     void desktop!.logout().catch(() => notify("Account locked. Offline credential cleanup needs the OS credential store to be unlocked.", "error"));
                     return;
              }
              sessionStorage.removeItem("armstrong-demo-auth");
              setSelected(null);
              setAuthenticated(false);
       };
       return (
              <>
                     <AppLayout
                            page={page}
                            setPage={setPage}
                            onSelect={setSelected}
                            onLogout={logout}
                     >
                            {desktop && (
                                   <>
                                          <div
                                                 className="foundation-warning"
                                                 role="note"
                                          >
                                                 <b>{desktop.authStatus?.authenticated ? `Signed in as ${desktop.authStatus.userName}` : "Desktop SQLite — local test build"}</b>
                                                 {desktop.authStatus?.authenticated ? `${desktop.authStatus.canWrite ? "Administrator access" : "Read-only computer"}${desktop.authStatus.offline ? " · Offline session" : ""}. ${desktop.snapshot?.memberSync?.available ? "Member sync connected; other modules remain local." : "Connect and sign in online to sync members."}` : "Authentication is not configured. This local test operator has unrestricted access. Use test records only."}
                                          </div>
                                          {desktop.error && (
                                                 <div
                                                        role="alert"
                                                        className="login-error"
                                                 >
                                                        {desktop.error}
                                                 </div>
                                          )}
                                          <p className="foundation-meta">
                                                 {desktop.snapshot
                                                        ? `${desktop.snapshot.pending} pending operations · ${desktop.snapshot.auditCount} audit entries · ${desktop.snapshot.today} (Asia/Colombo)`
                                                        : "Opening SQLite storage…"}{" "}
                                                 <button
                                                        className="secondary compact"
                                                        onClick={() =>
                                                               void desktop
                                                                      .refresh()
                                                                      .catch(
                                                                             () => {},
                                                                      )
                                                        }
                                                 >
                                                        Refresh
                                                 </button>
                                          </p>
                                   </>
                            )}
                            {desktop && !desktop.snapshot ? (
                                   <section className="card form-card">
                                          <h2>
                                                 {desktop.error
                                                        ? "Storage could not be loaded"
                                                        : "Opening local storage…"}
                                          </h2>
                                          <p>
                                                 No demo records will be
                                                 substituted. Use Refresh to
                                                 retry.
                                          </p>
                                   </section>
                            ) : (
                                   <>
                                          <DesktopScreenNotice page={page} />
                                          {content}
                                   </>
                            )}
                     </AppLayout>
                     {selected && (
                            <Modal
                                   title={selected.name}
                                   onClose={() => setSelected(null)}
                            >
                                   <p className="member-id">{selected.id}</p>
                                   <div className="member-summary">
                                          <div>
                                                 <small>Membership</small>
                                                 <b>{selected.plan}</b>
                                          </div>
                                          <div>
                                                 <small>Expiry</small>
                                                 <b>
                                                        {displayDate(
                                                               selected.expiry,
                                                        )}
                                                 </b>
                                          </div>
                                          <div>
                                                 <small>Phone</small>
                                                 <b>{selected.phone}</b>
                                          </div>
                                          <div>
                                                 <small>NFC card</small>
                                                 <b>
                                                        {selected.nfcId ||
                                                               "Not linked"}
                                                 </b>
                                          </div>
                                   </div>
                                   <button
                                          className="primary"
                                          onClick={() => {
                                                 setPage("Members");
                                                 setSelected(null);
                                          }}
                                   >
                                          Open member list
                                   </button>
                            </Modal>
                     )}
                     <div className="toast-stack">
                            {toasts.map((x) => (
                                   <div
                                          className={`toast ${x.tone}`}
                                          key={x.id}
                                   >
                                          {x.message}
                                   </div>
                            ))}
                     </div>
              </>
       );
}
