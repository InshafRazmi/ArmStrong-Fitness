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
  if (isDesktop()) content = <DesktopGymProvider><App /></DesktopGymProvider>
  else {
    // Browser storage/seeds/simulated sync are loaded only by the demo entry path.
    const { GymProvider } = await import('./context/BrowserGymProvider')
    content = <><div className="foundation-warning"><b>Browser prototype — demo data and simulated sync.</b>Desktop data is stored separately in SQLite.</div><GymProvider><App /></GymProvider></>
  }
  ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode>{content}</React.StrictMode>)
}
void start()
