import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { DesktopGymProvider } from './desktop/DesktopGymProvider'
import { isDesktop } from './desktop/api'
import './styles.css'
import './search.css'
import './desktop/foundation.css'

async function start() {
  let content
  let nativeRuntime = false, unavailable = false
  try { nativeRuntime = isDesktop() } catch { unavailable = true }
  if (import.meta.env.VITE_DESKTOP_ONLY === 'true' || unavailable) {
    content = nativeRuntime ? <DesktopGymProvider><App /></DesktopGymProvider>
      : <div className="foundation-warning" role="alert">The desktop connection is unavailable. Close and reopen ArmStrong Fitness. Gym records remain locked.</div>
  }
  else if (nativeRuntime) content = <DesktopGymProvider><App /></DesktopGymProvider>
  else {
    // Browser storage/seeds/simulated sync are loaded only by the demo entry path.
    const { GymProvider } = await import('./context/BrowserGymProvider')
    content = <><div className="foundation-warning"><b>Browser prototype — demo data and simulated sync.</b>Desktop data is stored separately in SQLite.</div><GymProvider><App /></GymProvider></>
  }
  ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode>{content}</React.StrictMode>)
}
void start()
