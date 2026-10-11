import assert from 'node:assert/strict'
import { createServer } from 'vite'
import { createElement as h } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

const server = await createServer({server:{middlewareMode:true,hmr:false,ws:false},optimizeDeps:{noDiscovery:true,include:[]}})
try {
  const { nfcSuccess, createNfcChime } = await server.ssrLoadModule('/src/utils/nfcFeedback.ts')
  const { NfcSuccessNotice } = await server.ssrLoadModule('/src/components/NfcSuccessNotice.tsx')
  for (const entity of ['Member','Staff']) for (const type of ['Check-in','Check-out']) {
    const result = {id:'saved-event',entity,type,name:'<Saved person>',duplicate:false}
    assert.deepEqual(nfcSuccess(result), {id:result.id,entity,type,name:result.name})
    const markup = renderToStaticMarkup(h(NfcSuccessNotice,{success:nfcSuccess(result)}))
    assert.ok(markup.includes('&lt;Saved person&gt;') && markup.includes(`${entity} · ${type} recorded`) && markup.includes('role="status"'))
    assert.equal(nfcSuccess({...result,duplicate:true}),null)
    assert.equal(nfcSuccess({...result,id:undefined}),null)
    assert.equal(nfcSuccess({...result,type:undefined}),null)
  }
  const trace = []
  const context = {state:'suspended',currentTime:10,destination:{},resume:async function(){trace.push('resume');this.state='running'},close:async()=>trace.push('close'),
    createOscillator:()=>({frequency:{value:0},connect(){},disconnect(){},start:time=>trace.push(['start',time]),stop:time=>trace.push(['stop',time])}),
    createGain:()=>({gain:{setValueAtTime(){},linearRampToValueAtTime(){},exponentialRampToValueAtTime(){}},connect(){},disconnect(){}})}
  const chime = createNfcChime(()=>context)
  chime.play(); assert.deepEqual(trace, [], 'no unprepared or premature sound')
  chime.prepare(); assert.deepEqual(trace,['resume'], 'scan gesture unlocks audio without a success sound')
  chime.play(); assert.equal(trace.filter(row=>Array.isArray(row)&&row[0]==='start').length,2)
  chime.dispose(); assert.equal(trace.at(-1),'close'); const count=trace.length
  chime.play(); assert.equal(trace.length,count,'unmounted scanner cannot play')
  const unavailable = createNfcChime(()=>{throw new Error('No audio output')})
  assert.doesNotThrow(()=>{unavailable.prepare();unavailable.play();unavailable.dispose()})
  console.log('PASS NFC saved-event feedback, duplicate suppression, safe audio and member/staff confirmations')
} finally { await server.close() }
