export const money=(value:number)=>`Rs. ${value.toLocaleString('en-LK')}`
export const displayDate=(value:string)=>value ? new Intl.DateTimeFormat('en-GB',{day:'2-digit',month:'short',year:'numeric'}).format(new Date(value+'T00:00:00')) : '—'
export const uid=(prefix:string)=>`${prefix}-${Date.now()}-${Math.random().toString(36).slice(2,6)}`
export const initials=(name:string)=>name.split(/\s+/).slice(0,2).map(part=>part[0]).join('').toUpperCase()
