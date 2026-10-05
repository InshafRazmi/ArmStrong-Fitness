// Runs through the shared React interface, native IPC and an isolated real SQLite database.
(async () => {
  const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
  async function until(check, label) {
    for (let i = 0; i < 100; i++) { if (check()) return; await sleep(100); }
    throw new Error(`Timed out: ${label}. Page: ${document.body.innerText.slice(-2400)}`);
  }
  const button = text => [...document.querySelectorAll('button')].find(button => button.textContent.trim() === text);
  async function click(text) { await until(() => button(text) && !button(text).disabled, text); button(text).click(); await sleep(80); }
  async function field(label, value) {
    const element = [...document.querySelectorAll('.modal-form label, .form-card label')].find(labelElement => labelElement.textContent.includes(label))?.querySelector('input,select');
    if (!element) throw new Error(`Missing field: ${label}`);
    const prototype = element.tagName === 'SELECT' ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
    Object.getOwnPropertyDescriptor(prototype, 'value').set.call(element, value);
    element.dispatchEvent(new Event(element.tagName === 'SELECT' ? 'change' : 'input', { bubbles: true }));
    await sleep(40);
  }
  const state = () => window.__TAURI__.core.invoke('foundation_snapshot');
  const assert = (condition, label) => { if (!condition) throw new Error(label); };
  const closed = () => !document.querySelector('.modal-form');
  const closeModal = async () => { document.querySelector('.modal-close').click(); await sleep(80); };
  const errors = [];
  addEventListener('error', event => errors.push(String(event.error || event.message)));
  addEventListener('unhandledrejection', event => errors.push(String(event.reason)));
  // Any demo storage access during navigation or saves is an acceptance failure.
  const originalGet = Storage.prototype.getItem;
  const originalSet = Storage.prototype.setItem;
  Storage.prototype.getItem = function () { throw new Error('Desktop attempted browser storage read'); };
  Storage.prototype.setItem = function () { throw new Error('Desktop attempted browser storage write'); };
  try {
    await until(() => document.body.textContent.includes('Desktop SQLite') && document.querySelector('.kpi-grid') && button('Refresh'), 'shared desktop interface loaded');
    await window.__TAURI__.core.invoke('smoke_progress', {stage:'interface loaded'});
    const pages = [
      ['Dashboard', '.kpi-grid'], ['Members', '.toolbar'], ['NFC Attendance', '.scanner'],
      ['Memberships', '.table-card'], ['Payments', '.table-card'], ['Sales & Inventory', '.table-card'],
      ['Expenses', '.table-card'], ['Reports', '.report-grid'], ['Settings', '.settings-grid'],
    ];
    assert(document.querySelectorAll('.sidebar nav button').length === 9, 'all nine navigation items present');
    for (const [page, selector] of pages) {
      await click(page);
      assert(document.querySelector('.topbar h1')?.textContent === page, `${page} title`);
      assert(document.querySelector(`.content ${selector}`), `${page} correct screen`);
      assert(document.querySelector('.sidebar nav button.active')?.textContent.trim() === page, `${page} active navigation`);
      if (page !== 'Dashboard') assert(document.querySelector('.page-header h2')?.textContent === page, `${page} page heading`);
      assert(!document.querySelector('.login-page'), 'desktop never uses demo login');
      assert(!document.body.textContent.includes('Browser prototype'), 'desktop never shows demo banner');
      if (page === 'Expenses') assert(!document.querySelector('.page-header button').disabled, 'expense form available');
      if (page === 'Reports') await until(() => [...document.querySelectorAll('.report-card')].every(button => !button.disabled), 'native report totals loaded and exports enabled');
    }
    assert(getComputedStyle(document.documentElement).getPropertyValue('--amber').trim() === '#f5a800', 'original amber theme loaded');
    assert(getComputedStyle(document.querySelector('.app')).display === 'grid', 'original shell layout loaded');
    for (const tab of ['Gym profile', 'Users & roles', 'NFC reader', 'Receipt printing', 'Backup & restore', 'Server synchronization', 'Application updates']) {
      await click(tab);
      assert(document.querySelector('.settings-grid .form-card h2')?.textContent === tab, `${tab} settings panel`);
      assert(Boolean(document.querySelector('input[type="file"]')) === (tab === 'Backup & restore'), 'only native backup restore accepts files');
    }
    await click('Backup & restore');
    assert(!button('Export backup').disabled && !button('Restore backup').disabled, 'native backups available');
    console.log('ARMSTRONG_UI_SMOKE: all nine navigation destinations and settings tabs verified');
    await window.__TAURI__.core.invoke('smoke_progress', {stage:'navigation and settings verified'});
    let data = await state();
    if (data.members.length) {
      assert(data.members.length === 1 && data.members[0].name === 'Desktop persistence test', 'member persisted across process restart');
      assert(data.members[0].phone === '0779999999', 'concurrent edit persisted');
      assert(data.periods.length === 2 && data.periods.some(period => period.planName === 'Monthly test'), 'historical membership persisted');
      assert(data.plans[0].name === 'Updated monthly test' && data.plans[0].priceMinor === 650075, 'plan edit persisted');
      assert(data.pending === 19 && data.auditCount === 19, 'audit and pending operations persisted');
      assert(data.profile.name === 'Persisted desktop gym' && data.profile.phone === '0661234567' && data.profile.email === 'native@example.lk', 'all settings survive restart');
      assert(data.attendance.length === 1 && data.payments.some(payment => payment.amountMinor === 600050 && payment.status === 'Reversed') && data.payments.length === 3 && data.expenses[0].amountMinor === 125075, 'attendance and finance survive restart');
      assert(data.products[0].stock === 4 && data.sales[0].totalMinor === 40100, 'sale and stock survive restart');
      assert(data.invoices.length === 2 && data.allocations.length === 3 && data.allocations.filter(a => a.reversedBy).length === 1, 'finance links survive restart');
      assert(data.financialAccounts[0].outstandingMinor === 550075 && data.financialAccounts[0].creditMinor === 100000 && data.financialAccounts[0].netBalanceMinor === 450075, 'derived partial balance and overpayment persist');
      await click('Payments'); await click('Receipt');
      await until(() => document.querySelector('.finance-receipt'), 'saved receipt preview after restart');
      assert(document.querySelector('.finance-receipt').textContent.includes(data.payments[0].receiptNumber), 'stable receipt number shown after restart');
      assert(button('Print / system preview'), 'real print action available; printer acceptance is manual');
      await closeModal();
      await click('Members');
      assert(document.querySelector('tbody').textContent.includes('Desktop persistence test'), 'restarted member shown in shared table');
      await click('Membership dates');
      assert(document.querySelector('.desktop-history').textContent.includes('Monthly test'), 'membership history visible after restart');
      await closeModal();
      console.log('ARMSTRONG_UI_SMOKE: restart verified');
      await window.__TAURI__.core.invoke('smoke_progress', {stage:'restart verified'});
    } else {
      assert(data.plans.length === 0 && data.periods.length === 0 && data.pending === 0, 'fresh SQLite is empty');
      await click('Memberships'); await click('Add plan');
      await field('Package name', 'Monthly test'); await field('Price', '6000.50'); await click('Save package changes');
      await until(closed, 'plan committed');
      await click('Members'); await click('Add member');
      await field('Full name', 'Desktop persistence test'); await field('Phone', '0771234567'); await field('NFC', ' test-card '); await click('Save member');
      await until(closed, 'member committed');
      assert(document.querySelector('tbody').textContent.includes('No membership'), 'new member has no invented membership');
      await click('Membership dates'); await field('Start date', '2026-01-31'); await field('Last valid day', '2026-02-28'); await click('Save membership dates');
      await until(closed, 'membership committed');
      await click('Add member'); await field('Full name', 'Duplicate must fail'); await field('Phone', '0777654321'); await field('NFC', 'TEST-CARD'); await click('Save member');
      await until(() => document.querySelector('.modal-form [role="alert"]')?.textContent.includes('already assigned'), 'duplicate card rejected without closing form');
      await closeModal();
      await click('Membership dates'); await field('Start date', '2026-02-01'); await field('Last valid day', '2026-03-01'); await click('Save membership dates');
      await until(() => document.querySelector('.modal-form [role="alert"]'), 'native overlap rejection shown without closing form');
      await closeModal();
      data = await state();
      assert(data.members.length === 1 && data.periods.length === 1, 'failed forms add no records');
      assert(data.plans[0].priceMinor === 600050 && data.pending === 3 && data.auditCount === 3, 'exact minor units and atomic audit/outbox writes');
      await click('Memberships'); await click('Edit'); await field('Package name', 'Updated monthly test'); await field('Price', '6500.75'); await click('Save package changes');
      await until(closed, 'plan edit committed');
      await click('Members'); await click('Edit'); await field('Phone', '0772222222'); await click('Save member');
      await until(closed, 'member edit committed');
      // A background refresh must not replace the expected version captured by the open editor.
      await click('Edit'); await field('Phone', '0773333333');
      data = await state();
      const member = data.members[0];
      await window.__TAURI__.core.invoke('save_member', { input: { id: member.id, version: member.version, name: member.name, phone: '0779999999', email: member.email, nfcId: member.nfcId } });
      await click('Refresh'); await click('Save member');
      await until(() => document.querySelector('.modal-form [role="alert"]'), 'stale native edit rejection stays visible');
      assert((await state()).members[0].phone === '0779999999', 'stale UI cannot overwrite committed data');
      await closeModal();
      data = await state();
      assert(data.periods[0].planName === 'Monthly test' && data.periods[0].priceMinor === 600050, 'history retains original name and price');
      assert(data.pending === 6 && data.auditCount === 6, 'failed operations do not append audit/outbox');
      await click('Payments'); await click('Receive payment'); await field('Amount', '6000.50'); await click('Save payment'); await until(closed, 'payment committed');
      await click('Expenses'); await click('Add expense'); await field('Description', 'Native electricity'); await field('Category', 'Utilities'); await field('Amount', '1250.75'); await field('Method', 'Bank'); await click('Save expense'); await until(closed, 'expense committed');
      await click('Sales & Inventory'); await click('Add product'); await field('Product name', 'Native water'); await field('SKU', 'WATER-1'); await field('Cost', '100.25'); await field('Selling price', '200.50'); await field('Reorder', '2'); await field('Opening stock', '5'); await click('Save product'); await until(closed, 'product committed');
      await click('New sale'); await field('Quantity', '6'); await click('Complete sale'); await until(()=>document.querySelector('.modal-form [role="alert"]'), 'oversell rejected'); await field('Quantity','2'); await click('Complete sale'); await until(closed, 'sale and stock committed');
      await click('+1'); await until(()=>!button('+1').disabled, 'stock adjustment committed');
      await click('NFC Attendance');
      const select = document.querySelector('.scanner select');
      Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype,'value').set.call(select,data.members[0].id);
      select.dispatchEvent(new Event('change',{bubbles:true}));
      await until(()=>document.querySelector('.activity-panel').textContent.includes('1 attendance records'), 'manual attendance committed');
      await click('Settings'); await click('Gym profile'); await field('Gym name','Persisted desktop gym'); await field('Location','Matale local test'); await field('Phone','0661234567'); await field('Email','native@example.lk'); await click('Save changes');
      await until(()=>!button('Saving…')&&button('Save changes'), 'profile committed');
      data = await state();
      assert(data.pending === 13 && data.auditCount === 13, 'operations commit with audit/outbox');
      assert(data.products[0].stock === 4 && data.sales[0].totalMinor === 40100, 'stock and money use native transaction');
      await click('Payments'); await click('New invoice'); await field('Description','Staff-entered finance test'); await field('Amount','10000.00'); await click('Save invoice'); await until(closed,'invoice committed');
      data = await state();
      const invoiceId = data.invoices[0].id;
      const originalPayment = data.payments[0];
      const savedOriginal = await window.__TAURI__.core.invoke('payment_receipt',{paymentId:originalPayment.id});
      await click('Allocate'); await field('Invoice',invoiceId); await field('Amount','3000.25'); await click('Save allocation'); await until(closed,'partial allocation committed');
      data = await state();
      assert(data.invoices[0].paidMinor === 300025 && data.invoices[0].outstandingMinor === 699975 && data.financialAccounts[0].creditMinor === 300025,'partial balance and credit calculated');
      await click('Receipt'); await until(() => document.querySelector('.finance-receipt'),'real saved-data receipt preview');
      assert(document.querySelector('.finance-receipt').textContent.includes(originalPayment.receiptNumber) && button('Print / system preview'),'saved number and real print action shown');
      await closeModal();
      const reprintedOriginal = await window.__TAURI__.core.invoke('payment_receipt',{paymentId:originalPayment.id});
      assert(reprintedOriginal.number === savedOriginal.number && JSON.stringify(reprintedOriginal.snapshot) === JSON.stringify(savedOriginal.snapshot),'reprint does not rewrite receipt after allocation');
      assert(reprintedOriginal.currentStatus === 'Partly allocated','reprint shows current allocation status separately');
      await click('Reverse'); await field('Reversal reason','Incorrect received amount'); await click('Confirm full reversal'); await until(closed,'full reversal committed');
      data = await state();
      assert(data.payments.find(p=>p.id===originalPayment.id).status === 'Reversed' && data.allocations[0].reversedBy && data.financialAccounts[0].outstandingMinor === 1000000 && data.financialAccounts[0].creditMinor === 0,'original retained and allocations released atomically');
      assert(!button('Reverse'),'duplicate reversal action unavailable');
      await click('Renew membership'); await field('Start date','2026-03-01'); await field('Last valid day','2026-03-31'); await click('Save renewal and invoice'); await until(closed,'explicit renewal and invoice committed');
      data = await state();
      const renewal = data.periods.find(p=>p.startsOn==='2026-03-01');
      const renewalInvoice = data.invoices.find(i=>i.membershipPeriodId===renewal.id);
      assert(renewal.priceMinor === 650075 && renewalInvoice.amountMinor === 650075,'renewal snapshots plan and linked invoice');
      await click('Receive payment'); await field('Invoice',invoiceId); await field('Amount','12000.00'); await click('Save payment'); await until(closed,'overpayment committed');
      data = await state();
      const overpayment = data.payments.find(p=>p.amountMinor===1200000);
      assert(overpayment.unallocatedMinor === 200000 && data.invoices.find(i=>i.id===invoiceId).outstandingMinor === 0,'overpayment remains credit without negative invoice');
      const originalReceipt = await window.__TAURI__.core.invoke('payment_receipt',{paymentId:overpayment.id});
      await click('Allocate'); await field('Invoice',renewalInvoice.id); await field('Amount','1000.00'); await click('Save allocation'); await until(closed,'credit topup committed');
      data = await state();
      assert(data.financialAccounts[0].outstandingMinor === 550075 && data.financialAccounts[0].creditMinor === 100000 && data.financialAccounts[0].netBalanceMinor === 450075,'partial renewal and residual credit reconciled');
      const reprintedOverpayment = await window.__TAURI__.core.invoke('payment_receipt',{paymentId:overpayment.id});
      assert(reprintedOverpayment.number === originalReceipt.number && JSON.stringify(reprintedOverpayment.snapshot) === JSON.stringify(originalReceipt.snapshot),'later allocation does not overwrite issue snapshot');
      assert(data.pending === 19 && data.auditCount === 19,'finance workflows each commit one audit/outbox operation');
      console.log('ARMSTRONG_UI_SMOKE: finance forms, balances, renewal, reversal and saved receipt preview verified');
      console.log('ARMSTRONG_UI_SMOKE: all local forms, error handling and SQLite verified');
      await window.__TAURI__.core.invoke('smoke_progress', {stage:'business forms verified'});
    }
    const removalBefore = await state();
    assert(removalBefore.removalAuthorization.allowed === false, 'local operator never receives removal privilege');
    const protectedMember = removalBefore.members[0];
    await click('Members'); await click('Archive / Deactivate');
    assert(button('Confirm archive').disabled && document.querySelector('input[type="checkbox"][required]'), 'archive confirmation present and unauthorized save locked');
    const archiveDenied = await window.__TAURI__.core.invoke('archive_member',{input:{requestId:crypto.randomUUID(),memberId:protectedMember.id,version:protectedMember.version}}).then(()=>false,error=>String(error).includes('authenticated Administrator'));
    assert(archiveDenied,'native archive cannot bypass unauthenticated lock'); await closeModal();
    const deleteDenied = await window.__TAURI__.core.invoke('delete_member',{input:{requestId:crypto.randomUUID(),memberId:protectedMember.id,version:protectedMember.version}}).then(()=>false,error=>String(error).includes('authenticated Administrator'));
    assert(deleteDenied,'native deletion cannot bypass unauthenticated lock');
    assert(!button('Delete permanently'),'linked member has no permanent-delete control');
    await click('Show archived members'); assert(document.body.textContent.includes('No archived members.'),'archive history view works on empty archive'); await click('Show active members');
    await click('Expenses'); await click('Void / Reverse');
    assert(button('Confirm expense void').disabled && document.querySelector('input[type="checkbox"][required]'),'expense confirmation present and unauthorized save locked');
    const reasonField=[...document.querySelectorAll('.modal-form label')].find(label=>label.textContent.includes('Required void reason'))?.querySelector('input');
    assert(reasonField?.required,'void reason required');
    const voidDenied = await window.__TAURI__.core.invoke('void_expense',{input:{requestId:crypto.randomUUID(),expenseId:removalBefore.expenses[0].id,reason:'Unauthorized must fail'}}).then(()=>false,error=>String(error).includes('authenticated Administrator'));
    assert(voidDenied,'native expense void cannot bypass unauthenticated lock'); await closeModal();
    const removalAfter = await state();
    assert(JSON.stringify(removalAfter)===JSON.stringify(removalBefore),'denied removal adds no records, audit or pending writes');
    console.log('ARMSTRONG_UI_SMOKE: removal confirmations, archive view and native unauthorized denial verified');
    const before = await state();
    await click('Dashboard');
    document.querySelector('.sync').click(); await sleep(100);
    const after = await state();
    assert(after.pending === before.pending, 'sync control never acknowledges pending operations without a server');
    assert(document.querySelector('.sync').textContent.includes('NO SERVER'), 'no false synced status');
    assert([...document.querySelectorAll('.bar-chart i')].filter(bar => parseFloat(bar.style.height) > 0).length === 1, 'trend uses the single real attendance day');
    await click('Reports');
    await until(() => [...document.querySelectorAll('.report-card')].every(button => !button.disabled), 'native report totals loaded');
    await click('Today');
    await until(() => [...document.querySelectorAll('.report-card')].every(button => !button.disabled), 'today report totals loaded');
    assert([...document.querySelectorAll('.content input[type="date"]')].every(input => input.value === after.today), 'report date controls use the native Colombo business date');
    const report = await window.__TAURI__.core.invoke('report_summary', { range: {fromOn:after.today,toOn:after.today} });
    const incomeMinor = after.payments.filter(row=>row.businessOn===after.today).reduce((sum,row)=>sum+row.netAmountMinor,0) + after.sales.filter(row=>row.businessOn===after.today).reduce((sum,row)=>sum+row.totalMinor,0);
    assert(report.incomeMinor === incomeMinor, 'native report received amounts reconcile to saved payment/sale history');
    document.querySelector('.report-card').click(); await until(()=>document.querySelector('.storage-file-result'), 'date-filtered native report file saved');
    await window.__TAURI__.core.invoke('smoke_progress', {stage:'native report export verified'});
    await click('Settings'); await click('Backup & restore'); await click('Export backup'); await until(()=>document.querySelector('.storage-file-result'), 'validated native backup saved');
    await window.__TAURI__.core.invoke('smoke_progress', {stage:'native backup export verified'});
    assert(errors.length === 0, `no uncaught render/storage errors: ${errors.join('; ')}`);
    Storage.prototype.getItem = originalGet; Storage.prototype.setItem = originalSet;
    await window.__TAURI__.core.invoke('smoke_finished', { error: null });
  } catch (error) {
    Storage.prototype.getItem = originalGet; Storage.prototype.setItem = originalSet;
    await window.__TAURI__.core.invoke('smoke_finished', { error: String(error) });
  }
})();
