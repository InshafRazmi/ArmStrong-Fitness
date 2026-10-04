import { seedData } from '../data/seed'
import type { GymData } from '../types/domain'

const KEY='armstrong-gym-data-v1'
export function loadData():GymData{
  try { const value=localStorage.getItem(KEY); return value?{...seedData,...JSON.parse(value)}:structuredClone(seedData) }
  catch { return structuredClone(seedData) }
}
export function saveData(data:GymData){ localStorage.setItem(KEY,JSON.stringify(data)) }
export function exportBackup(data:GymData){
  const blob=new Blob([JSON.stringify(data,null,2)],{type:'application/json'})
  const url=URL.createObjectURL(blob); const a=document.createElement('a'); a.href=url; a.download=`armstrong-backup-${new Date().toISOString().slice(0,10)}.json`; a.click(); URL.revokeObjectURL(url)
}
export async function readBackup(file:File):Promise<GymData>{ return JSON.parse(await file.text()) as GymData }
