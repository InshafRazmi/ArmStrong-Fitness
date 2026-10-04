// Render the real React routes without a display server. Native click/IPC acceptance is separate.
import assert from 'node:assert/strict'
import { nativeSnapshot } from './native-snapshot.mjs'
import { createElement as h } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { createServer } from 'vite'

const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, optimizeDeps: { noDiscovery: true, include: [] } })
try {
  const { GymContext } = await server.ssrLoadModule('/src/context/GymContext.tsx')
  const { pages, PageContent } = await server.ssrLoadModule('/src/pages/navigation.tsx')
  const { AppLayout } = await server.ssrLoadModule('/src/layout/AppLayout.tsx')
  const { DesktopScreenNotice } = await server.ssrLoadModule('/src/desktop/DesktopScreenNotice.tsx')
  const { DesktopSettingsPanel } = await server.ssrLoadModule('/src/desktop/DesktopSettingsPanel.tsx')
  const { desktopData, desktopMembers } = await server.ssrLoadModule('/src/desktop/adapter.ts')
  const { DesktopGymProvider } = await server.ssrLoadModule('/src/desktop/DesktopGymProvider.tsx')
  const { default: App } = await server.ssrLoadModule('/src/App.tsx')
  const { isDesktop } = await server.ssrLoadModule('/src/desktop/api.ts')
  const native = nativeSnapshot()
  const unexpected = () => { throw new Error('Render triggered a write or browser storage access') }
  const value = {
    mode: 'desktop', desktop: { archiveMember: unexpected, deleteMember: unexpected, voidExpense: unexpected, snapshot: native, error: '', refresh: unexpected, createInvoice: unexpected, allocatePayment: unexpected, receivePayment: unexpected, renewMembership: unexpected, reversePayment: unexpected, paymentReceipt: unexpected, savePlan: unexpected, addPeriod: unexpected, saveProfile: unexpected, saveProduct: unexpected, exportBackup: unexpected, previewRestore: unexpected, restoreBackup: unexpected, exportReport: unexpected },
    data: desktopData(native), online: false, syncing: false, toasts: [], notify: unexpected,
    addMember: unexpected, updateMember: unexpected, updatePlan: unexpected,
    recordAttendance: unexpected, addPayment: unexpected, addExpense: unexpected,
    adjustStock: unexpected, completeSale: unexpected, restore: unexpected, syncNow: unexpected,
  }
  globalThis.localStorage = { getItem: unexpected, setItem: unexpected }
  globalThis.sessionStorage = { getItem: unexpected, setItem: unexpected }
  const names = ['Dashboard', 'Members', 'NFC Attendance', 'Memberships', 'Payments', 'Sales & Inventory', 'Expenses', 'Reports', 'Settings']
  const components = ['DashboardPage', 'MembersPage', 'AttendancePage', 'MembershipsPage', 'PaymentsPage', 'InventoryPage', 'ExpensesPage', 'ReportsPage', 'SettingsPage']
  assert.deepEqual(pages.map(page => page.name), names)
  for (const [index, page] of pages.entries()) {
    assert.equal(page.Screen.name, components[index], `${page.name} maps to the intended component`)
    const markup = renderToStaticMarkup(h(GymContext.Provider, { value }, h(AppLayout, { page: page.name, setPage: unexpected, onSelect: unexpected, onLogout: unexpected }, h(DesktopScreenNotice, { page: page.name }), h(PageContent, { page: page.name, navigate: unexpected }))))
    const escapedName = page.name.replaceAll('&', '&amp;')
    assert.ok(markup.includes(`<h1>${escapedName}</h1>`), `${page.name} header`)
    assert.equal((markup.match(/<nav>/g) ?? []).length, 1)
    for (const name of names) assert.ok(markup.includes(`<span>${name.replaceAll('&', '&amp;')}</span>`), `${name} navigation exists in ${page.name}`)
    assert.ok(markup.includes(`class="active"`), `${page.name} active navigation`)
    if (page.name !== 'Dashboard') assert.ok(markup.includes(`<h2>${escapedName}</h2>`), `${page.name} correct screen heading`)
    assert.ok(markup.includes('8 PENDING · NO SERVER'), 'native pending counts never labeled synced')
    assert.ok(markup.includes('Local operator') && !markup.includes('Prinzz'), 'no demo administrator in native shell')
    assert.ok(!markup.includes('storage unavailable') && !markup.includes('storage not implemented'), `${page.name} reads implemented storage`)
    if (page.name === 'Dashboard') {
      assert.ok(markup.includes('Total Members') && markup.includes('Sign in online to synchronize members.'))
      assert.ok(!markup.includes('Auto-sync on reconnection') && markup.includes('last 28 business days') && markup.includes('Payments + retail sales'))
      assert.ok(markup.includes('524.45'), 'today received amounts include native payments and sales')
    }
    if (page.name === 'Members') assert.ok(markup.includes('SQLite member') && markup.includes('Historical plan') && markup.includes('Membership dates') && markup.includes('Archive / Deactivate') && markup.includes('Show archived members') && markup.includes('authenticated Administrator'))
    if (page.name === 'Payments') assert.ok(markup.includes('Recorded') && markup.includes('123.45') && markup.includes('Stored membership invoice') && markup.includes('New invoice') && markup.includes('Renew membership') && markup.includes('Reverse') && markup.includes('Receipt') && markup.includes('876.55'))
    if (page.name === 'Expenses') assert.ok(markup.includes('Stored electricity') && markup.includes('23.45') && markup.includes('Void / Reverse') && markup.includes('Effective expenses'))
    if (page.name === 'Sales & Inventory') assert.ok(markup.includes('Stored bottle') && markup.includes('Add product'))
    if (page.name === 'NFC Attendance') assert.ok(markup.includes('00:05') && markup.includes('SQLite member'))
    if (page.name === 'Reports') assert.ok([...markup.matchAll(/<button class="card report-card"([^>]*)>/g)].every(match=>match[1].includes('disabled')) && markup.includes('Loading report totals') && markup.includes('From date') && markup.includes('To date') && markup.includes('Asia/Colombo') && markup.includes('current stock'), 'report export waits for native totals and exposes date filters')
    if (page.name === 'Memberships') assert.ok(markup.includes('SQLite plan') && markup.includes('6,000.5') && markup.includes('Add plan'))
    console.log(`PASS rendered navigation route: ${page.name}`)
  }
  for (const tab of ['Gym profile', 'Users & roles', 'NFC reader', 'Receipt printing', 'Backup & restore', 'Server synchronization', 'Application updates']) {
    const markup = renderToStaticMarkup(h(GymContext.Provider, { value }, h(DesktopSettingsPanel, { tab })))
    assert.ok(!markup.includes('>Ready<') && !markup.includes('>Active<'), tab+' no fake capability labels')
    if (tab === 'Gym profile') {
      for (const field of ['Gym name','Location','Phone','Email','Stored gym','gym@example.test']) assert.ok(markup.includes(field), field+' from native profile')
      assert.ok(markup.includes('Save changes') && !markup.includes('disabled=""'))
    } else if (tab === 'Backup & restore') {
      assert.ok(markup.includes('type="file"') && markup.includes('.armstrong-backup.json') && !markup.includes('disabled=""'))
    } else if (tab === 'NFC reader') assert.ok(markup.includes('Unverified'))
    else if (tab === 'Users & roles') assert.ok(markup.includes('Authentication is not configured') && markup.includes('Use test records only'))
    else if (tab === 'Server synchronization') assert.ok(markup.includes('Prepare this computer') && markup.includes('Account sign-in prepares this computer automatically') && markup.includes('Sync unavailable'))
    else if (tab === 'Application updates') assert.ok(markup.includes('Manual updates') && markup.includes('Gym records are stored separately') && !markup.includes('Check for updates'))
    else assert.ok(markup.includes('Unverified') || markup.includes('No authenticated backend'), tab+' accurately reports its capability')
  }
  console.log('PASS all seven desktop settings panels')
  const restoredMarkup = renderToStaticMarkup(h(GymContext.Provider,{value:{...value,desktop:{...value.desktop,snapshot:{...native,restoreRequiresReconciliation:true}}}},h(DesktopSettingsPanel,{tab:'Server synchronization'})))
  assert.ok(/<button class="secondary" disabled="">Prepare this computer<\/button>/.test(restoredMarkup),'restored database cannot prepare a substitute credential')
  const conflictValue = { ...value, desktop: { ...value.desktop, snapshot: { ...native, memberSync: {available:false,reason:'Enrollment required',cursor:1,acknowledged:2,conflicts:[{id:'conflict-1',memberId:'sqlite-member',operationId:'op-1',reason:'Remote change overlaps pending local member edits',remote:{name:'Server & member',nfcId:'CARD-2',archivedAt:'2026-10-03T10:00:00Z'},createdAt:'2026-10-03T10:00:00Z'}]} } } }
  const conflictMarkup = renderToStaticMarkup(h(GymContext.Provider, { value: conflictValue }, h(DesktopSettingsPanel, {tab:'Server synchronization'})))
  for (const text of ['SQLite member / CARD-1','Server &amp; member / CARD-2','Archived','Local member records and history have been retained','An enrolled Administrator can review','Confirmed member operations','Sync unavailable']) assert.ok(conflictMarkup.includes(text), text)
  assert.ok(conflictMarkup.includes('disabled=""'), 'unconnected member sync remains disabled')
  const failedSyncValue = { ...conflictValue, desktop: { ...conflictValue.desktop, snapshot: { ...conflictValue.desktop.snapshot, memberSync: { ...conflictValue.desktop.snapshot.memberSync, lastError:'Server request failed; local operations are retained',retryOn:'2026-10-04T01:00:00Z',lastSuccessOn:null } } } }
  const failedSyncMarkup = renderToStaticMarkup(h(GymContext.Provider,{value:failedSyncValue},h(DesktopSettingsPanel,{tab:'Server synchronization'})))
  assert.ok(failedSyncMarkup.includes('Server request failed; local operations are retained') && failedSyncMarkup.includes('Sync unavailable') && failedSyncMarkup.includes('disabled=""'),'retained retry failure does not enable live sync')
  const allowedConflictValue = {...conflictValue,desktop:{...conflictValue.desktop,snapshot:{...conflictValue.desktop.snapshot,memberSync:{...conflictValue.desktop.snapshot.memberSync,reviewAuthorization:{allowed:true,reason:''},resolved:2,superseded:3}}}}
  const allowedConflictMarkup = renderToStaticMarkup(h(GymContext.Provider,{value:allowedConflictValue},h(DesktopSettingsPanel,{tab:'Server synchronization'})))
  assert.ok(/<button class="secondary">Review conflict<\/button>/.test(allowedConflictMarkup),'enrolled Administrator can request a native review')
  assert.ok(allowedConflictMarkup.includes('discarded or superseded edits') && allowedConflictMarkup.includes('separate from confirmed server operations'))
  const { MemberConflictDialog } = await server.ssrLoadModule('/src/desktop/MemberConflictDialog.tsx')
  const review = {conflictId:'conflict-1',memberId:'sqlite-member',fingerprint:'test-native-review',local:{name:'Current local',phone:'0771234567',email:'',nfcId:'CARD-1',joinedOn:'2026-10-01',archivedAt:null,version:3},remote:{name:'Recorded server',phone:'0777654321',email:'server@example.test',nfcId:'CARD-2',joinedOn:'2026-10-01',archivedAt:null,revision:5},conflicts:[{id:'conflict-1',reason:'Revision changed'}],operations:[{id:'op-1',action:'update'}],useServer:{allowed:true,reason:''},keepLocal:{allowed:true,reason:''}}
  const reviewMarkup = renderToStaticMarkup(h(GymContext.Provider,{value:allowedConflictValue},h(MemberConflictDialog,{preview:review,onClose:unexpected})))
  for(const text of ['Current local version 3','Recorded server revision 5','Current local','Recorded server','does not contact the server','Discard the reviewed pending member edits','queue a fresh retry','Review reason','I confirm the reviewed member conflict choice','Confirm member review']) assert.ok(reviewMarkup.includes(text),text)
  const blockedReview = {...review,useServer:{allowed:false,reason:'Retry the unconfirmed request'},keepLocal:{allowed:false,reason:'Retry the unconfirmed request'}}
  const blockedMarkup = renderToStaticMarkup(h(GymContext.Provider,{value:allowedConflictValue},h(MemberConflictDialog,{preview:blockedReview,onClose:unexpected})))
  assert.ok(blockedMarkup.includes('Retry the unconfirmed request') && /<button class="primary" disabled="">Confirm member review<\/button>/.test(blockedMarkup))
  console.log('PASS member conflict comparison, Administrator gate and blocked review without enabling sync')
  const failedValue = { ...value, desktop: { ...value.desktop, snapshot: null, error: 'SQLite storage failure' } }
  const failedMarkup = renderToStaticMarkup(h(GymContext.Provider, { value: failedValue }, h(App)))
  assert.ok(failedMarkup.includes('SQLite storage failure') && failedMarkup.includes('No demo records will be substituted'))
  assert.ok(!failedMarkup.includes('SQLite member') && !failedMarkup.includes('Register new member'))
  const loadingMarkup = renderToStaticMarkup(h(DesktopGymProvider, null, h(App)))
  assert.ok(loadingMarkup.includes('Opening local storage') && !loadingMarkup.includes('login-page'))
  globalThis.window = { __TAURI_INTERNALS__: {} }
  assert.equal(isDesktop(), true, 'Tauri cannot silently fall back to demo if the global invoke bridge is unavailable')
  console.log('PASS desktop loading/error states and fail-closed runtime detection')
  const lockedAuth = {requiresLogin:true,configured:true,authenticated:false,canWrite:false,userName:null,role:null,expiresAt:null,reason:'Approved Administrator sign-in required'}
  const lockedValue = {...value,desktop:{...value.desktop,snapshot:null,authStatus:lockedAuth,login:unexpected,logout:unexpected}}
  const nativeLogin = renderToStaticMarkup(h(GymContext.Provider,{value:lockedValue},h(App)))
  assert.ok(nativeLogin.includes('login-page') && nativeLogin.includes('Administrator email') && nativeLogin.includes('Internet access is required') && nativeLogin.includes('Computer registration'))
  assert.ok(!nativeLogin.includes('SQLite member') && !nativeLogin.includes('OFFLINE-READY ACCESS'))
  assert.ok(!nativeLogin.includes('Continue offline'), 'fresh computers cannot invent offline authorization')
  const offlineLogin = renderToStaticMarkup(h(GymContext.Provider,{value:{...lockedValue,desktop:{...lockedValue.desktop,authStatus:{...lockedAuth,offlineUntil:'2026-10-12T10:00:00Z'}}}},h(App)))
  assert.ok(offlineLogin.includes('Continue offline') && offlineLogin.includes('OS account') && !offlineLogin.includes('SQLite member'))
  const configuredMissing = renderToStaticMarkup(h(GymContext.Provider,{value:{...lockedValue,desktop:{...lockedValue.desktop,authStatus:{...lockedAuth,configured:false}}}},h(App)))
  assert.ok(/class="login-submit" disabled=""/.test(configuredMissing),'missing native configuration cannot submit a demo login')
  const signedIn = renderToStaticMarkup(h(GymContext.Provider,{value:{...value,desktop:{...value.desktop,authStatus:{...lockedAuth,authenticated:true,canWrite:true,userName:'Verified Administrator',role:'Administrator',expiresAt:'2026-10-04T12:00:00Z'}}}},h(App)))
  assert.ok(signedIn.includes('Signed in as Verified Administrator') && signedIn.includes('Administrator · Sign out') && !signedIn.includes('local test build'))
  console.log('PASS native sign-in gate, missing configuration and verified user shell')


  const { ReceiptView } = await server.ssrLoadModule('/src/desktop/ReceiptView.tsx')
  const savedReceipt = { number:'AF-R-stable',currentStatus:'Reversed',reversalReceiptNumber:'AF-R-reversal',snapshot:{
    formatVersion:1,number:'AF-R-stable',payment:{...native.payments[0],memberName:'<script>untrusted</script>'},
    gym:{name:'Historical gym',location:'Matale',phone:'0661234567',email:'old@example.lk'},
    allocations:[{invoiceNumber:'AF-I-stable',description:'Saved plan',amountMinor:1200,outstandingMinor:8800,released:0}],
    unallocatedAtIssueMinor:11145,originalReceiptNumber:null,reason:null,issuedAt:'2026-10-03T01:00:00Z',legacy:true
  }}
  const preview = renderToStaticMarkup(h(ReceiptView,{document:savedReceipt}))
  for (const text of ['AF-R-stable','AF-R-reversal','REVERSED','Legacy received-amount record','Historical gym','Saved plan','12','88','111.45','Cash','local-operator','Asia/Colombo']) assert.ok(preview.includes(text),text+' saved receipt content')
  assert.ok(preview.includes('&lt;script&gt;') && !preview.includes('<script>'), 'receipt content is escaped')
  assert.ok(!preview.includes('Stored gym') && !preview.includes('SQLite member'), 'reprint reads historical snapshot rather than live profile/member')
  const reversalPreview = renderToStaticMarkup(h(ReceiptView,{document:{...savedReceipt,currentStatus:'Reversal',snapshot:{...savedReceipt.snapshot,legacy:false,payment:{...savedReceipt.snapshot.payment,reversesId:'original'},reason:'Wrong received amount',originalReceiptNumber:'AF-R-original',allocations:[{...savedReceipt.snapshot.allocations[0],released:1}]}}}))
  assert.ok(reversalPreview.includes('Payment reversal') && reversalPreview.includes('Released:') && reversalPreview.includes('AF-R-original') && reversalPreview.includes('Wrong received amount') && reversalPreview.includes('does not confirm a bank or card refund'))
  console.log('PASS saved receipt preview, legacy/reversed/reversal labels, historical fields and escaping')
  const reversalSnapshot = {...native,payments:[...native.payments,{...native.payments[0],id:'reversal',netAmountMinor:-12345,status:'Reversal',reversesId:'payment-1'}]}
  const reversedDashboard = renderToStaticMarkup(h(GymContext.Provider,{value:{...value,desktop:{...value.desktop,snapshot:reversalSnapshot},data:desktopData(reversalSnapshot)}},h(PageContent,{page:'Dashboard',navigate:unexpected})))
  assert.ok(reversedDashboard.includes('Rs. 401'), 'dashboard subtracts reversal rather than doubling received cash')
  console.log('PASS dashboard cash effect of payment reversal')


  const { MemberRemovalDialog } = await server.ssrLoadModule('/src/desktop/MemberRemovalDialog.tsx')
  const { ExpenseVoidDialog } = await server.ssrLoadModule('/src/desktop/ExpenseVoidDialog.tsx')
  const member=desktopMembers(native,true)[0]
  for (const kind of ['archive','delete']) {
    const markup=renderToStaticMarkup(h(GymContext.Provider,{value},h(MemberRemovalDialog,{member:{...member,canDelete:true},kind,onClose:unexpected})))
    assert.ok(markup.includes('Confirmation') && markup.includes('type="checkbox"') && markup.includes('required=""'))
    assert.ok(markup.includes('authenticated Administrator') && markup.includes('disabled=""'),kind+' locked without authentication')
    assert.ok(markup.includes(kind==='archive'?'Existing dates, NFC assignment, invoices, payments and all history remain':'Audit and pending operation history remain'))
  }
  const voidMarkup=renderToStaticMarkup(h(GymContext.Provider,{value},h(ExpenseVoidDialog,{expense:desktopData(native).expenses[0],onClose:unexpected})))
  assert.ok(voidMarkup.includes('Required void reason') && voidMarkup.includes('maxLength="254"') && voidMarkup.includes('type="checkbox"') && voidMarkup.includes('authenticated Administrator') && voidMarkup.includes('disabled=""'))
  const authorizedValue={...value,desktop:{...value.desktop,snapshot:{...native,removalAuthorization:{allowed:true,userId:'verified',user:'Verified administrator',reason:''}}}}
  const authorizedVoid=renderToStaticMarkup(h(GymContext.Provider,{value:authorizedValue},h(ExpenseVoidDialog,{expense:desktopData(native).expenses[0],onClose:unexpected})))
  assert.ok(!authorizedVoid.includes('disabled=""'), 'verified administrator can confirm after filling reason/confirmation')
  const voidSnapshot={...native,expenses:[{...native.expenses[0],status:'Voided',effectiveAmountMinor:0,voidReason:'Duplicate electricity entry',voidedBy:'Verified administrator',voidedAt:'2026-10-03T02:00:00Z'}]}
  const voidValue={...value,desktop:{...value.desktop,snapshot:voidSnapshot},data:desktopData(voidSnapshot)}
  const historyMarkup=renderToStaticMarkup(h(GymContext.Provider,{value:voidValue},h(PageContent,{page:'Expenses',navigate:unexpected})))
  assert.ok(historyMarkup.includes('Stored electricity') && historyMarkup.includes('23.45') && historyMarkup.includes('Voided') && historyMarkup.includes('Duplicate electricity entry') && historyMarkup.includes('Verified administrator') && !historyMarkup.includes('Void / Reverse'), 'voided expense stays visible and cannot be voided twice')
  const expenseReport=renderToStaticMarkup(h(GymContext.Provider,{value:voidValue},h(PageContent,{page:'Reports',navigate:unexpected})))
  assert.ok(expenseReport.includes('Expense totals exclude voided records') && expenseReport.includes('Loading report totals'), 'report does not display UI-calculated cash totals before native summary arrives')
  console.log('PASS removal confirmations, unauthenticated lock, void history and native report loading')

  // The real browser provider still renders seeded demo screens and its existing login.
  globalThis.localStorage = { getItem: () => null, setItem: unexpected }
  globalThis.sessionStorage = { getItem: () => null, setItem: unexpected }
  const { GymProvider } = await server.ssrLoadModule('/src/context/BrowserGymProvider.tsx')
  const loginMarkup = renderToStaticMarkup(h(GymProvider, null, h(App)))
  assert.ok(loginMarkup.includes('login-page'), 'browser keeps demo login')
  globalThis.sessionStorage = { getItem: () => 'true', setItem: unexpected }
  const browserMarkup = renderToStaticMarkup(h(GymProvider, null, h(App)))
  assert.ok(browserMarkup.includes('Prinzz') && !browserMarkup.includes('Desktop SQLite'), 'demo remains separate')
  for (const name of names) {
    const markup = renderToStaticMarkup(h(GymProvider, null, h(PageContent, { page: name, navigate: unexpected })))
    assert.ok(markup.length > 300, `${name} browser demo screen renders`)
  }
  console.log('PASS separate browser demo login, provider and all nine routes')
} finally {
  await server.close()
}
