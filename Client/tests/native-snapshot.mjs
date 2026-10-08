// Test-only native-shaped data. Production desktop never imports this fixture.
export function nativeSnapshot() {
  return {
    trainers: [], memberTrainers: [], trainingCharges: [], staffPayouts: [], staffTrainingAllocations: [],
    removalAuthorization: {allowed:false,userId:null,user:null,reason:'Removal requires an authenticated Administrator session'},
    members: [{ id:'sqlite-member',version:2,active:true,archivedAt:null,archivedByUserId:null,canDelete:false,name:'SQLite member',phone:'0771234567',email:'',nfcId:'CARD-1',joinedOn:'2026-10-01' }],
    plans: [{ id:'sqlite-plan',version:3,name:'SQLite plan',priceMinor:600050,durationMonths:1,active:true,activeMembers:1 }],
    periods: [{ id:'sqlite-period',memberId:'sqlite-member',planId:'sqlite-plan',planName:'Historical plan',priceMinor:500000,startsOn:'2026-10-01',endsOn:'2026-10-31',status:'Active' }],
    profile: {version:1,name:'Stored gym',location:'Matale',phone:'0771234567',email:'gym@example.test'},
    attendance: [{id:'event-1',memberId:'sqlite-member',name:'SQLite member',businessOn:'2026-10-03',occurredAt:'2026-10-02T18:35:00Z',type:'Check-in',source:'NFC',cardId:'card-1',cardUid:'CARD-1',voidsId:null}],
    payments: [{id:'payment-1',memberId:'sqlite-member',memberName:'SQLite member',amountMinor:12345,netAmountMinor:12345,allocatedMinor:0,unallocatedMinor:12345,status:'Recorded',receiptNumber:'AF-R-device-payment-1',reversalReason:null,method:'Cash',businessOn:'2026-10-03',createdAt:'2026-10-03T01:00:00Z',actor:'local-operator (unauthenticated)',reversesId:null}],
    invoices: [{id:'invoice-1',number:'AF-I-device-invoice-1',memberId:'sqlite-member',memberName:'SQLite member',membershipPeriodId:null,amountMinor:100000,paidMinor:0,outstandingMinor:100000,description:'Stored membership invoice',issuedOn:'2026-10-03',createdAt:'2026-10-03T01:00:00Z',status:'Unpaid'}],
    allocations: [],
    financialAccounts: [{memberId:'sqlite-member',memberName:'SQLite member',outstandingMinor:100000,creditMinor:12345,netBalanceMinor:87655}],
    expenses: [{id:'expense-1',title:'Stored electricity',category:'Utilities',amountMinor:2345,effectiveAmountMinor:2345,status:'Recorded',voidReason:null,voidedAt:null,voidedBy:null,voidedByUserId:null,method:'Bank',businessOn:'2026-10-03',createdAt:'2026-10-03T01:00:00Z',actor:'local-operator (unauthenticated)',reversesId:null}],
    products: [{id:'product-1',version:2,name:'Stored bottle',sku:'BOTTLE',costMinor:10025,priceMinor:20050,reorderLevel:2,stock:3}],
    sales: [{id:'sale-1',totalMinor:40100,method:'Card',businessOn:'2026-10-03',createdAt:'2026-10-03T01:00:00Z',actor:'local-operator (unauthenticated)',reversesId:null,items:[{id:'item-1',productId:'product-1',name:'Historical bottle',sku:'BOTTLE',quantity:2,priceMinor:20050,costMinor:10025}]}],
    audit:[{id:'audit-1',action:'sale',entity:'sale',entityId:'sale-1',user:'local-operator (unauthenticated)',deviceId:'device-1',beforeJson:null,afterJson:'{}',timestamp:'2026-10-03T01:00:00Z'}],
    users:[],today:'2026-10-03',pending:8,auditCount:8,restoreRequiresReconciliation:false,
  }
}
