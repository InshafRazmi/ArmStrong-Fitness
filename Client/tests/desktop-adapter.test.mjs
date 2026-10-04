import assert from 'node:assert/strict'
import test from 'node:test'
import { nativeSnapshot } from './native-snapshot.mjs'
import { minorUnits, requireSnapshot } from '../src/desktop/api.ts'
import { desktopData, desktopMembers, emptyDesktopData } from '../src/desktop/adapter.ts'

test('device preparation accepts no caller identity or credential and propagates vault failure', async () => {
  const api = await import('../src/desktop/api.ts')
  let fail = false
  const calls = []
  const approval = {deviceId:'native-device',sqlitePath:'/native/database.sqlite3',secretSha256:'b'.repeat(64)}
  globalThis.window = {__TAURI__:{core:{invoke:async (command,args)=>{
    calls.push({command,args})
    if (fail) throw new Error('OS credential storage is unavailable')
    return approval
  }}}}
  assert.deepEqual(await api.prepareNativeDevice(),approval)
  assert.deepEqual(calls[0],{command:'prepare_native_device',args:undefined})
  fail = true
  await assert.rejects(api.prepareNativeDevice(),/OS credential storage/)
  delete globalThis.window
})
test('desktop login exposes only email/password IPC and logout/status remain native', async () => {
  const api = await import('../src/desktop/api.ts')
  const calls = []
  globalThis.window = {__TAURI__:{core:{invoke:async (command,args)=>{calls.push({command,args});if(command==='desktop_login') throw new Error('Sign-in refused');return {authenticated:false}}}}}
  await api.desktopAuthStatus()
  await assert.rejects(api.desktopLogin('admin@example.test','private-test-password'),/Sign-in refused/)
  await api.desktopLogout()
  assert.deepEqual(calls,[{command:'desktop_auth_status',args:undefined},{command:'desktop_login',args:{email:'admin@example.test',password:'private-test-password'}},{command:'desktop_logout',args:undefined}])
  delete globalThis.window
})
test('offline unlock, session renewal and member sync expose no client identity, scope, credential or database path',async()=>{
  const api=await import('../src/desktop/api.ts')
  const calls=[]
  globalThis.window={__TAURI__:{core:{invoke:async(command,args)=>{calls.push({command,args});throw new Error('Native authorization refused')}}}}
  await assert.rejects(api.desktopUnlockOffline(),/authorization refused/)
  await assert.rejects(api.desktopRenewSession(),/authorization refused/)
  await assert.rejects(api.synchronizeMembers(),/authorization refused/)
  assert.deepEqual(calls,[{command:'desktop_unlock_offline',args:undefined},{command:'desktop_renew_session',args:undefined},{command:'synchronize_members',args:undefined}])
  delete globalThis.window
})

const member = { id: 'member-1', version: 7, active: true, archivedAt: null, archivedByUserId: null, canDelete: false, name: 'Local Member', phone: '0771234567', email: '', nfcId: 'CARD', joinedOn: '2026-01-01' }
const plan = { id: 'plan-1', version: 3, name: 'Renamed plan', durationMonths: 1, priceMinor: 600050, active: true, activeMembers: 1 }
const period = { id: 'period-1', memberId: member.id, planId: plan.id, planName: 'Original plan', priceMinor: 500025, startsOn: '2026-10-01', endsOn: '2026-10-31', status: 'Active' }
const snapshot = (periods = [period]) => ({ ...nativeSnapshot(), members: [member], plans: [plan], periods, attendance: [], payments: [], products: [], sales: [], expenses: [], audit: [], pending: 12, auditCount: 12 })

test('fresh desktop data never supplies demo records or fake audit/outbox entries', () => {
  const empty = emptyDesktopData()
  assert.ok(Object.values(empty).every(records => records.length === 0))
  const data = desktopData(snapshot())
  for (const key of ['attendance', 'payments', 'products', 'sales', 'expenses', 'queue', 'audit']) assert.deepEqual(data[key], [])
})
test('native prices, versions and historical membership names survive the interface adapter', () => {
  const data = desktopData(snapshot())
  assert.equal(data.plans[0].price, 6000.50)
  assert.equal(data.plans[0].version, 3)
  assert.equal(data.plans[0].activeMembers, 1)
  assert.equal(data.members[0].version, 7)
  assert.equal(data.members[0].plan, 'Original plan')
  assert.equal(data.members[0].expiry, '2026-10-31')
  assert.equal(data.members[0].nfcId, 'CARD')
})
test('current membership takes precedence over a later scheduled period', () => {
  const future = { ...period, id: 'future', startsOn: '2026-11-01', endsOn: '2026-11-30', status: 'Scheduled' }
  assert.equal(desktopData(snapshot([future, period])).members[0].status, 'Active')
  assert.equal(desktopData(snapshot([future, period])).members[0].expiry, '2026-10-31')
})
test('unassigned, scheduled and expired members are never presented as active demo members', () => {
  const unassigned = desktopData(snapshot([])).members[0]
  assert.equal(unassigned.status, 'No membership')
  assert.equal(unassigned.plan, 'No membership')
  assert.equal(unassigned.expiry, '')
  assert.equal(desktopData(snapshot([{ ...period, status: 'Scheduled' }])).members[0].status, 'Scheduled')
  assert.equal(desktopData(snapshot([{ ...period, status: 'Expired' }])).members[0].status, 'Expired')
})

test('operational snapshots preserve money, history, Colombo time and pending status', () => {
  const data = desktopData(nativeSnapshot())
  assert.equal(data.attendance[0].date, '2026-10-03')
  assert.equal(data.attendance[0].time, '00:05')
  assert.equal(data.payments[0].amount, 123.45)
  assert.equal(data.payments[0].status, 'Recorded')
  assert.equal(data.payments[0].syncState, 'pending')
  assert.equal(data.expenses[0].amount, 23.45)
  assert.equal(data.products[0].stock, 3)
  assert.equal(data.products[0].version, 2)
  assert.equal(data.sales[0].items[0].name, 'Historical bottle')
  assert.equal(data.sales[0].total, 401)
  assert.deepEqual(data.audit, nativeSnapshot().audit)
  assert.deepEqual(data.queue, [])
})
test('upcoming display chooses earliest scheduled membership', () => {
  const late = { ...period, id:'late', startsOn:'2026-12-01', endsOn:'2026-12-31', status:'Scheduled' }
  const early = { ...period, id:'early', startsOn:'2026-11-01', endsOn:'2026-11-30', status:'Scheduled' }
  assert.equal(desktopData(snapshot([late,early])).members[0].expiry,'2026-11-30')
})
test('incompatible native snapshots fail closed and currency never silently loses fractions', () => {
  assert.equal(requireSnapshot(nativeSnapshot()).pending,8)
  assert.throws(()=>requireSnapshot({members:[]}),/no demo data/)
  assert.throws(()=>requireSnapshot({...nativeSnapshot(),payments:undefined}),/incompatible/)
  assert.equal(minorUnits(6000.50),600050)
  assert.equal(minorUnits(0.29),29)
  for (const amount of [NaN,Infinity,-1,0.001,1_000_000_001]) assert.throws(()=>minorUnits(amount),/valid LKR/)
})

test('payment reversals have negative cash effect without altering original received amounts', () => {
  const native = nativeSnapshot()
  native.payments.push({...native.payments[0],id:'reversal',netAmountMinor:-12345,status:'Reversal',reversesId:'payment-1',unallocatedMinor:0})
  const data = desktopData(native)
  assert.equal(data.payments[0].amount,123.45)
  assert.equal(data.payments[1].amount,-123.45)
  assert.equal(data.payments[1].status,'Reversal')
  for (const field of ['invoices','allocations','financialAccounts']) assert.throws(()=>requireSnapshot({...native,[field]:undefined}),/incompatible/)
})
test('finance storage methods use exact native commands and payloads and propagate failures', async () => {
  const api = await import('../src/desktop/api.ts')
  const calls = []
  globalThis.window = {__TAURI__:{core:{invoke:async (command,args)=>{calls.push({command,args}); if(command==='reverse_payment') throw new Error('SQLite rollback'); return {id:'committed'} }}}}
  const input = {requestId:'operation',memberId:'member',amountMinor:50}
  await api.createInvoice(input)
  await api.allocatePayment(input)
  await api.receivePayment(input)
  await api.renewMembership(input)
  await assert.rejects(api.reversePayment(input),/SQLite rollback/)
  await api.paymentReceipt('saved-payment')
  assert.deepEqual(calls.map(call=>call.command),['create_invoice','allocate_payment','receive_payment','renew_membership','reverse_payment','payment_receipt'])
  assert.deepEqual(calls[0].args,{input})
  assert.deepEqual(calls[5].args,{paymentId:'saved-payment'})
  globalThis.window = {}
  await assert.rejects(api.receivePayment(input),/desktop application/)
  delete globalThis.window
})

test('archived members are hidden from normal data and remain in explicit history without hiding debts', () => {
  const native = nativeSnapshot()
  native.members[0] = {...native.members[0],active:false,archivedAt:'2026-10-03T02:00:00Z',archivedByUserId:'administrator'}
  const data = desktopData(native)
  assert.deepEqual(data.members,[])
  assert.equal(desktopMembers(native,true)[0].status,'Archived')
  assert.equal(desktopMembers(native,true)[0].plan,'Historical plan')
  assert.equal(data.payments[0].memberName,'SQLite member')
  assert.equal(native.financialAccounts[0].outstandingMinor,100000)
  assert.throws(()=>requireSnapshot({...native,members:[{...native.members[0],active:undefined}]}),/incompatible/)
  assert.throws(()=>requireSnapshot({...native,removalAuthorization:undefined}),/incompatible/)
})
test('voided expense history retains received fields and uses zero effective expense', () => {
  const native=nativeSnapshot()
  native.expenses[0]={...native.expenses[0],status:'Voided',effectiveAmountMinor:0,voidReason:'Duplicate',voidedAt:'2026-10-03T02:00:00Z',voidedBy:'Verified administrator'}
  const expense=desktopData(native).expenses[0]
  assert.equal(expense.amount,23.45)
  assert.equal(expense.effectiveAmount,0)
  assert.equal(expense.status,'Voided')
  assert.equal(expense.voidReason,'Duplicate')
  assert.equal(expense.recordedBy,'local-operator (unauthenticated)')
  assert.equal(expense.voidedBy,'Verified administrator')
})
test('removal adapter forwards committed native writes and propagates authorization denial', async () => {
  const api=await import('../src/desktop/api.ts')
  const calls=[]
  const input={requestId:'request',memberId:'member',version:7}
  globalThis.window={__TAURI__:{core:{invoke:async(command,args)=>{calls.push({command,args});throw new Error('Authenticated Administrator required')}}}}
  await assert.rejects(api.archiveMember(input),/Administrator/)
  await assert.rejects(api.deleteMember(input),/Administrator/)
  const expense={requestId:'request',expenseId:'expense',reason:'Duplicate'}
  await assert.rejects(api.voidExpense(expense),/Administrator/)
  assert.deepEqual(calls,[{command:'archive_member',args:{input}},{command:'delete_member',args:{input}},{command:'void_expense',args:{input:expense}}])
  delete globalThis.window
})

test('native date-filtered reports forward bounds and preserve summary/export failures', async () => {
  const api = await import('../src/desktop/api.ts')
  const calls = []
  const range = {fromOn:'2026-10-01',toOn:'2026-10-03'}
  globalThis.window = {__TAURI__:{core:{invoke:async(command,args)=>{
    calls.push({command,args})
    if (command === 'export_report') throw new Error('Invalid report dates')
    return {fromOn:range.fromOn,toOn:range.toOn,incomeMinor:12345,expenseMinor:2345,netMinor:10000}
  }}}}
  assert.equal((await api.reportSummary(range)).netMinor,10000)
  await assert.rejects(api.exportReport('Income report',range),/Invalid report dates/)
  assert.deepEqual(calls,[{command:'report_summary',args:{range}},{command:'export_report',args:{kind:'Income report',range}}])
  delete globalThis.window
})

test('member conflict IPC sends a native preview fingerprint and propagates stale review failure', async () => {
  const api = await import('../src/desktop/api.ts')
  const calls = []
  globalThis.window = {__TAURI__:{core:{invoke:async(command,args)=>{
    calls.push({command,args})
    if(command==='resolve_member_conflict') throw new Error('Member or conflict changed')
    return {fingerprint:'native-fingerprint'}
  }}}}
  assert.deepEqual(await api.previewMemberConflict('conflict-1'),{fingerprint:'native-fingerprint'})
  const input = {requestId:'review-request',conflictId:'conflict-1',fingerprint:'native-fingerprint',choice:'keep_local',reason:'Reviewed current edits'}
  await assert.rejects(api.resolveMemberConflict(input),/changed/)
  assert.deepEqual(calls,[{command:'preview_member_conflict',args:{conflictId:'conflict-1'}},{command:'resolve_member_conflict',args:{input}}])
  assert.equal('remote' in input,false)
  assert.equal('actorUserId' in input,false)
  delete globalThis.window
})
