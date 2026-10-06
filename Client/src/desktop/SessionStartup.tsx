export function SessionStartup({ error, onRetry }: { error?: string; onRetry?: () => void }) {
  return <div className="login-page session-startup">
    <div className="login-shade"/>
    <div className="login-brand">
      <div className="brand-mark login-logo">A</div>
      <div><span>ARMSTRONG</span><b>FITNESS</b></div>
    </div>
    <section className="login-card" aria-busy={!error}>
      <div className="login-kicker"><span/>MANAGEMENT SYSTEM</div>
      <h1>{error ? 'Unable to check session.' : 'Checking your session…'}</h1>
      {error ? <>
        <div className="login-error" role="alert">{error}</div>
        {onRetry && <button type="button" className="login-submit" onClick={onRetry}>Try again</button>}
      </> : <p role="status">Please wait while we check your access.</p>}
    </section>
  </div>
}
