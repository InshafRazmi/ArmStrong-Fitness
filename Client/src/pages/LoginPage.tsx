import { useState } from "react";
import { Icon } from "../components/ui/Icon";
import { useGym } from "../context/GymContext";
import { DevicePreparation } from "../desktop/DesktopSettingsPanel";
import { errorText } from "../desktop/DesktopGymProvider";

export function LoginPage({ onLogin }: { onLogin: () => void }) {
       const { mode, desktop } = useGym();
       const native = mode === "desktop";
       const [username, setUsername] = useState("");
       const [password, setPassword] = useState("");
       const [show, setShow] = useState(false);
       const [error, setError] = useState("");
       const [loading, setLoading] = useState(false);
       const submit = async (event: React.FormEvent) => {
              event.preventDefault();
              if (loading) return;
              setError("");
              if (native) {
                     setLoading(true);
                     try { await desktop!.login(username, password); setPassword(""); onLogin(); }
                     catch (error) { setPassword(""); setError(errorText(error)); }
                     finally { setLoading(false); }
                     return;
              }
              if (username !== "Admin" || password !== "arm123") {
                     setError(
                            "Incorrect username or password. Please try again.",
                     );
                     return;
              }
              setLoading(true);
              setTimeout(() => {
                     sessionStorage.setItem("armstrong-demo-auth", "true");
                     onLogin();
              }, 450);
       };
       return (
              <div className="login-page">
                     <div className="login-shade" />
                     <div className="login-brand">
                            <div className="brand-mark login-logo">A</div>
                            <div>
                                   <span>ARMSTRONG</span>
                                   <b>FITNESS</b>
                            </div>
                     </div>
                     <section className="login-card">
                            <div className="login-kicker">
                                   <span />
                                   MANAGEMENT SYSTEM
                            </div>
                            <h1>Welcome back.</h1>
                            <p>
                                   Sign in to manage members, attendance and
                                   daily operations.
                            </p>
                            {native && <p role="status">{desktop?.authStatus?.reason}</p>}
                            <form onSubmit={event => void submit(event)}>
                                   <label>
                                          <span>{native ? "Email" : "Username"}</span>
                                          <div className="login-input">
                                                 <Icon name="users" size={17} />
                                                 <input
                                                        autoFocus
                                                        autoComplete="username"
                                                        type={native ? "email" : "text"}
                                                        required
                                                        disabled={loading}
                                                        maxLength={254}
                                                        value={username}
                                                        onChange={(e) => {
                                                               setUsername(
                                                                      e.target
                                                                             .value,
                                                               );
                                                               setError("");
                                                        }}
                                                        placeholder={native ? "Administrator email" : "Enter username"}
                                                 />
                                          </div>
                                   </label>
                                   <label>
                                          <span>Password</span>
                                          <div className="login-input">
                                                 <Icon
                                                        name="settings"
                                                        size={17}
                                                 />
                                                 <input
                                                        type={
                                                               show
                                                                      ? "text"
                                                                      : "password"
                                                        }
                                                        autoComplete="current-password"
                                                        required
                                                        disabled={loading}
                                                        maxLength={4096}
                                                        value={password}
                                                        onChange={(e) => {
                                                               setPassword(
                                                                      e.target
                                                                             .value,
                                                               );
                                                               setError("");
                                                        }}
                                                        placeholder="Enter password"
                                                 />
                                                 <button
                                                        type="button"
                                                        onClick={() =>
                                                               setShow(!show)
                                                        }
                                                 >
                                                        {show ? "Hide" : "Show"}
                                                 </button>
                                          </div>
                                   </label>
                                   {error && (
                                          <div className="login-error">
                                                 ! <span>{error}</span>
                                          </div>
                                   )}
                                   <button
                                          className="login-submit"
                                          disabled={loading || (native && !desktop?.authStatus?.configured)}
                                   >
                                          {loading ? "Signing in…" : "Sign in"}
                                          <i>→</i>
                                   </button>
                            </form>
                            <div className="login-security">
                                   <span>●</span>
                                   <div>
                                          <b>{native ? "VERIFIED ACCOUNT ACCESS" : "OFFLINE-READY ACCESS"}</b>
                                          <small>
                                                 {native ? "Internet access is required to sign in. Signing out locks local records on this device." : "Your local gym data remains available securely on this device."}
                                          </small>
                                   </div>
                            </div>
                            {native && <details><summary>Computer registration</summary><DevicePreparation/></details>}
                     </section>
                     <footer className="login-footer">
                            ARMSTRONG FITNESS · MATALE <span>•</span> DESKTOP
                            MANAGEMENT v0.6
                     </footer>
              </div>
       );
}
