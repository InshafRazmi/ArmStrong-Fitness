import type { SyncOperation } from '../types/domain'

export async function pushOperations(queue:SyncOperation[]):Promise<string[]>{
  if(!navigator.onLine) throw new Error('No internet connection')
  await new Promise(resolve=>setTimeout(resolve,650))
  return queue.map(item=>item.id)
}
