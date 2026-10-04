import { useRef, useState } from 'react'
import { Modal, PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { Product } from '../types/domain'
import { money } from '../utils/format'
import { minorUnits } from '../desktop/api'
import { errorText } from '../desktop/DesktopGymProvider'
export function InventoryPage() {
  const { data, adjustStock, completeSale, desktop } = useGym()
  const [open, setOpen] = useState(false)
  const [productId, setProduct] = useState('')
  const [quantity, setQuantity] = useState(1)
  const [method, setMethod] = useState<'Cash' | 'Card' | 'Transfer'>('Cash')
  const [editing, setEditing] = useState<Product | null>(null)
  const [openingStock, setOpening] = useState(0)
  const [requestId, setRequest] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const stockRequests = useRef(new Map<string, string>())
  function newSale() { setProduct(data.products[0]?.id ?? ''); setQuantity(1); setRequest((desktop ? crypto.randomUUID() : '')); setError(''); setOpen(true) }
  function editProduct(product?: Product) {
    setEditing(product ? { ...product } : { id: '', name: '', sku: '', cost: 0, price: 0, reorderLevel: 0, stock: 0 })
    setOpening(0); setRequest((desktop ? crypto.randomUUID() : '')); setError('')
  }
  async function sale(event: React.FormEvent) {
    event.preventDefault()
    if (busy) return
    setBusy(true); setError('')
    try { await completeSale(productId, quantity, method, requestId); setOpen(false) }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  async function saveProduct(event: React.FormEvent) {
    event.preventDefault()
    if (!editing || !desktop || busy) return
    setBusy(true); setError('')
    try {
      await desktop.saveProduct({ requestId, id: editing.id || undefined, version: editing.version, name: editing.name, sku: editing.sku, costMinor: minorUnits(editing.cost), priceMinor: minorUnits(editing.price), reorderLevel: editing.reorderLevel, openingStock })
      setEditing(null)
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  async function adjust(id: string, amount: number) {
    if (busy) return
    setBusy(true); setError('')
    const key = id + ':' + amount
    const operationId = stockRequests.current.get(key) ?? (desktop ? crypto.randomUUID() : '')
    stockRequests.current.set(key, operationId)
    try { await adjustStock(id, amount, operationId); stockRequests.current.delete(key) }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <><PageHeader title="Sales & Inventory" subtitle="Retail sales, stock levels and automatic stock deduction" action="New sale" disabled={!data.products.length || busy} onAction={newSale}/>
    {desktop && <div className="toolbar"><button className="secondary" disabled={busy} onClick={() => editProduct()}>Add product</button></div>}
    {error && !open && !editing && <div role="alert" className="login-error">{error}</div>}
    <div className="card table-card"><table><thead><tr><th>Product</th><th>SKU</th><th>Stock</th><th>Cost</th><th>Selling price</th><th>Status</th><th>Adjust</th></tr></thead><tbody>{data.products.map(product => <tr key={product.id}><td><b>{product.name}</b></td><td>{product.sku}</td><td>{product.stock}</td><td>{money(product.cost)}</td><td>{money(product.price)}</td><td><span className={product.stock <= product.reorderLevel ? 'tag amber' : 'tag green'}>{product.stock <= product.reorderLevel ? 'Low stock' : 'In stock'}</span></td><td><button className="secondary compact" disabled={busy} onClick={() => void adjust(product.id, 1)}>+1</button> <button className="secondary compact" disabled={busy} onClick={() => void adjust(product.id, -1)}>−1</button> {desktop && <button className="secondary compact" disabled={busy} onClick={() => editProduct(product)}>Edit</button>}</td></tr>)}</tbody></table>{!data.products.length && <p className="foundation-empty">No products recorded.</p>}</div>
    {open && <Modal title="Create retail sale" onClose={() => { if (!busy) setOpen(false) }}><form className="modal-form" onSubmit={event => void sale(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Product</span><select value={productId} onChange={event => setProduct(event.target.value)}>{data.products.map(product => <option key={product.id} value={product.id}>{product.name} — {product.stock} available</option>)}</select></label>
      <label><span>Quantity</span><input type="number" required min="1" step="1" value={quantity} onChange={event => setQuantity(Number(event.target.value))}/></label>
      <label><span>Method</span><select value={method} onChange={event => setMethod(event.target.value as typeof method)}><option>Cash</option><option>Card</option><option>Transfer</option></select></label>
      {error && <div role="alert" className="login-error">{error}</div>}<button className="primary">{busy ? 'Saving…' : 'Complete sale'}</button>
    </fieldset></form></Modal>}
    {editing && desktop && <Modal title={editing.id ? 'Edit product' : 'Add product'} onClose={() => { if (!busy) setEditing(null) }}><form className="modal-form" onSubmit={event => void saveProduct(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Product name</span><input required maxLength={120} value={editing.name} onChange={event => setEditing({ ...editing, name: event.target.value })}/></label>
      <label><span>SKU</span><input required maxLength={80} value={editing.sku} onChange={event => setEditing({ ...editing, sku: event.target.value })}/></label>
      <label><span>Cost (LKR)</span><input type="number" required min="0" step="0.01" value={editing.cost} onChange={event => setEditing({ ...editing, cost: Number(event.target.value) })}/></label>
      <label><span>Selling price (LKR)</span><input type="number" required min="0" step="0.01" value={editing.price} onChange={event => setEditing({ ...editing, price: Number(event.target.value) })}/></label>
      <label><span>Reorder level</span><input type="number" required min="0" max="1000000" step="1" value={editing.reorderLevel} onChange={event => setEditing({ ...editing, reorderLevel: Number(event.target.value) })}/></label>
      {!editing.id && <label><span>Opening stock</span><input type="number" required min="0" max="1000000" step="1" value={openingStock} onChange={event => setOpening(Number(event.target.value))}/></label>}
      <p className="form-note">Existing stock changes use the +1/−1 ledger controls. Past sale prices and costs are retained.</p>
      {error && <div role="alert" className="login-error">{error}</div>}<button className="primary">{busy ? 'Saving…' : 'Save product'}</button>
    </fieldset></form></Modal>}
  </>
}
