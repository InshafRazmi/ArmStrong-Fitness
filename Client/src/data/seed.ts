import type { GymData } from '../types/domain'

export const seedData: GymData = {
  members: [
    {id:'MF-10234',name:'Rahul Sharma',phone:'077 512 8834',email:'rahul@example.com',plan:'Platinum',expiry:'2027-05-19',status:'Active',initials:'RS',nfcId:'NFC-10234',joinedAt:'2025-05-19'},
    {id:'MF-09876',name:'Priya Mehta',phone:'076 887 1920',email:'priya@example.com',plan:'Gold',expiry:'2026-09-28',status:'Active',initials:'PM',nfcId:'NFC-09876',joinedAt:'2026-03-28'},
    {id:'MF-09901',name:'Arjun Verma',phone:'071 400 8291',email:'arjun@example.com',plan:'Gold',expiry:'2026-09-12',status:'Expiring',initials:'AV',nfcId:'NFC-09901',joinedAt:'2026-03-12'},
    {id:'MF-10112',name:'Neha Kapoor',phone:'075 619 0034',email:'neha@example.com',plan:'Silver',expiry:'2027-01-05',status:'Active',initials:'NK',nfcId:'NFC-10112',joinedAt:'2026-10-05'},
    {id:'MF-09654',name:'Aditya Sharma',phone:'077 981 2304',email:'aditya@example.com',plan:'Gold',expiry:'2026-09-06',status:'Expiring',initials:'AS',nfcId:'NFC-09654',joinedAt:'2026-03-06'},
  ],
  attendance: [
    {id:'AT-1',memberId:'MF-10234',name:'Rahul Sharma',time:'10:24 AM',date:'2026-09-01',type:'Check-in',source:'NFC',syncState:'synced'},
    {id:'AT-2',memberId:'MF-09876',name:'Priya Mehta',time:'10:18 AM',date:'2026-09-01',type:'Check-in',source:'NFC',syncState:'synced'},
    {id:'AT-3',memberId:'MF-09901',name:'Arjun Verma',time:'10:12 AM',date:'2026-09-01',type:'Check-out',source:'NFC',syncState:'synced'},
    {id:'AT-4',memberId:'MF-10112',name:'Neha Kapoor',time:'10:08 AM',date:'2026-09-01',type:'Check-in',source:'Manual',syncState:'synced'},
  ],
  plans:[{id:'PL-1',name:'Platinum',durationMonths:12,price:48000,activeMembers:384,status:'Active'},{id:'PL-2',name:'Gold',durationMonths:6,price:27000,activeMembers:512,status:'Active'},{id:'PL-3',name:'Silver',durationMonths:3,price:15000,activeMembers:279,status:'Active'},{id:'PL-4',name:'Monthly',durationMonths:1,price:6000,activeMembers:73,status:'Active'}],
  payments:[{id:'INV-1088',memberId:'MF-10234',memberName:'Rahul Sharma',date:'2026-09-01',method:'Cash',amount:7500,status:'Paid',syncState:'synced'},{id:'INV-1087',memberId:'MF-09876',memberName:'Priya Mehta',date:'2026-09-01',method:'Card',amount:5000,status:'Paid',syncState:'synced'},{id:'INV-1086',memberId:'MF-09901',memberName:'Arjun Verma',date:'2026-08-31',method:'Cash',amount:3000,status:'Partial',syncState:'synced'}],
  products:[{id:'PR-1',name:'Whey Protein 1kg',sku:'SUP-WP-1KG',stock:3,reorderLevel:5,cost:8500,price:10500},{id:'PR-2',name:'Protein Bar Box',sku:'SUP-PB-BOX',stock:2,reorderLevel:5,cost:3200,price:4200},{id:'PR-3',name:'Energy Drink 250ml',sku:'BEV-ED-250',stock:24,reorderLevel:8,cost:280,price:400},{id:'PR-4',name:'Gym Towel',sku:'ACC-TWL',stock:15,reorderLevel:5,cost:700,price:1200}],
  sales:[],
  expenses:[{id:'EX-1',title:'Monthly electricity',category:'Utilities',date:'2026-09-01',method:'Bank',amount:42800,recordedBy:'Prinzz',syncState:'synced'},{id:'EX-2',title:'Equipment service',category:'Maintenance',date:'2026-08-30',method:'Cash',amount:18500,recordedBy:'Admin',syncState:'synced'},{id:'EX-3',title:'Cleaning supplies',category:'Operations',date:'2026-08-29',method:'Cash',amount:6750,recordedBy:'Reception',syncState:'synced'}],
  audit:[], queue:[]
}
