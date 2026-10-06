import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react'
import { loadData, saveData } from '../services/storage'
import { pushOperations } from '../services/sync'
import { initials, uid } from '../utils/format'
import { colomboToday, membershipDaysRemaining, membershipEndDate } from '../utils/membership'
import type { Attendance, Expense, GymData, Member, MembershipPlan, Payment, Product, Sale, ToastMessage } from '../types/domain'

import { GymContext } from './GymContext'
import type { NewMember } from './GymContext'

export function GymProvider({children}:{children:ReactNode}){
 const [data,setData]=useState(loadData); const [online,setOnline]=useState(navigator.onLine); const [syncing,setSyncing]=useState(false); const [toasts,setToasts]=useState<ToastMessage[]>([])
 useEffect(()=>saveData(data),[data])
 useEffect(()=>{const on=()=>setOnline(true),off=()=>setOnline(false);addEventListener('online',on);addEventListener('offline',off);return()=>{removeEventListener('online',on);removeEventListener('offline',off)}},[])
 const notify=useCallback((message:string,tone:ToastMessage['tone']='success')=>{const id=uid('TOAST');setToasts(v=>[...v,{id,message,tone}]);setTimeout(()=>setToasts(v=>v.filter(x=>x.id!==id)),3000)},[])
 const mutate=useCallback((entity:string,action:'create'|'update'|'delete',payload:unknown,apply:(d:GymData)=>GymData)=>setData(old=>{const changed=apply(old);const op={id:uid('SYNC'),entity,action,payload,createdAt:new Date().toISOString(),attempts:0};return{...changed,queue:[...changed.queue,op],audit:[{id:uid('AUD'),action:`${action} ${entity}`,entity,entityId:(payload as {id?:string}).id||'',user:'Prinzz',timestamp:new Date().toISOString()},...changed.audit]}}),[])
 const addMember=(v:NewMember)=>{
  const today=colomboToday();const plan=v.planId?data.plans.find(p=>p.id===v.planId&&p.status==='Active'):undefined
  if(v.planId&&!plan)throw new Error('Select an active membership package')
  const expiry=plan?membershipEndDate(v.startsOn??'',plan.durationMonths):''
  if(plan&&!expiry)throw new Error('Choose a valid membership start date')
  const days=membershipDaysRemaining(expiry,today,v.startsOn??undefined)
  const member:Member={name:v.name,phone:v.phone,email:v.email,nfcId:v.nfcId,plan:plan?.name??'No membership',expiry,membershipStartsOn:v.startsOn??undefined,
   id:`MF-${String(Math.floor(10000+Math.random()*89999))}`,status:!plan?'No membership':(v.startsOn??today)>today?'Scheduled':days===0?'Expired':days!==null&&days<=8?'Expiring':'Active',initials:initials(v.name),joinedAt:today}
  mutate('member','create',member,d=>({...d,members:[member,...d.members]}));notify('Member saved in browser demo')
 }
 const updateMember=(member:Member)=>{mutate('member','update',member,d=>({...d,members:d.members.map(x=>x.id===member.id?member:x)}));notify('Member updated')}
 const updatePlan=(plan:MembershipPlan)=>{mutate('membership plan','update',plan,d=>({...d,plans:d.plans.map(x=>x.id===plan.id?plan:x)}));notify(`${plan.name} package updated`)}
 const recordAttendance=(memberId:string,source:'NFC'|'Manual')=>{const member=data.members.find(x=>x.id===memberId||x.nfcId===memberId);if(!member){notify('Card is not linked to a member','error');return}const last=data.attendance.find(x=>x.memberId===member.id);const row:Attendance={id:uid('AT'),memberId:member.id,name:member.name,date:new Date().toISOString().slice(0,10),time:new Date().toLocaleTimeString([],{hour:'2-digit',minute:'2-digit'}),type:last?.date===new Date().toISOString().slice(0,10)&&last.type==='Check-in'?'Check-out':'Check-in',source,syncState:online?'synced':'pending'};mutate('attendance','create',row,d=>({...d,attendance:[row,...d.attendance]}));notify(`${member.name}: ${row.type} successful`)}
 const addPayment=(v:Omit<Payment,'id'|'syncState'>)=>{const row={...v,id:uid('INV'),syncState:online?'synced':'pending'} as Payment;mutate('payment','create',row,d=>({...d,payments:[row,...d.payments]}));notify('Payment recorded and receipt is ready')}
 const addExpense=(v:Omit<Expense,'id'|'syncState'>)=>{const row={...v,id:uid('EX'),syncState:online?'synced':'pending'} as Expense;mutate('expense','create',row,d=>({...d,expenses:[row,...d.expenses]}));notify('Expense recorded')}
 const adjustStock=(id:string,amount:number)=>{const product=data.products.find(x=>x.id===id);if(!product)return;const changed={...product,stock:Math.max(0,product.stock+amount)};mutate('product','update',changed,d=>({...d,products:d.products.map(x=>x.id===id?changed:x)}));notify('Stock updated')}
 const completeSale=(productId:string,quantity:number,method:'Cash'|'Card'|'Transfer')=>{const product=data.products.find(x=>x.id===productId);if(!product||quantity<1||product.stock<quantity){notify('Not enough stock for this sale','error');return}const sale:Sale={id:uid('SALE'),date:new Date().toISOString(),items:[{productId,name:product.name,quantity,price:product.price}],total:product.price*quantity,method,syncState:online?'synced':'pending'};mutate('sale','create',sale,d=>({...d,sales:[sale,...d.sales],products:d.products.map((x:Product)=>x.id===productId?{...x,stock:x.stock-quantity}:x)}));notify('Sale completed and stock deducted')}
 const syncNow=useCallback(async()=>{if(!online){notify('You are offline. Changes remain safely stored.','info');return}if(!data.queue.length){notify('Everything is already synced','info');return}setSyncing(true);try{await pushOperations(data.queue);setData(d=>({...d,queue:[],attendance:d.attendance.map(x=>({...x,syncState:'synced'})),payments:d.payments.map(x=>({...x,syncState:'synced'})),sales:d.sales.map(x=>({...x,syncState:'synced'})),expenses:d.expenses.map(x=>({...x,syncState:'synced'}))}));notify('Server synchronization complete')}catch{notify('Sync failed. Data remains stored locally.','error')}finally{setSyncing(false)}},[data.queue,notify,online])
 useEffect(()=>{if(online&&data.queue.length)void syncNow()},[online])
 const value=useMemo(()=>({mode:'browser' as const,data,online,syncing,toasts,addMember,updateMember,updatePlan,recordAttendance,addPayment,addExpense,adjustStock,completeSale,syncNow,restore:setData,notify}),[data,online,syncing,toasts,syncNow,notify])
 return <GymContext.Provider value={value}>{children}</GymContext.Provider>
}
