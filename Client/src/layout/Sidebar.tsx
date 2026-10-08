import { useGym } from '../context/GymContext';import { Icon } from '../components/ui/Icon';import type { Page } from '../types/domain'
import { pages } from '../pages/navigation'
import armLogo from '../img/ArmLogo.png'
export function Sidebar({page,setPage}:{page:Page;setPage:(p:Page)=>void}){const{mode}=useGym();return <aside className="sidebar"><img className="brand-mark" src={armLogo} alt="Armstrong Fitness Gym"/><nav>{pages.map(x=><button key={x.name} className={page===x.name?'active':''} onClick={()=>setPage(x.name)}><Icon name={x.icon}/><span>{x.name}</span></button>)}</nav><div className="sidebar-foot"><span className="status-dot"/> {mode==='desktop'?'SQLite · Local':'Offline-first'}</div></aside>}
