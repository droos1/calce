import { useState, useEffect, useRef, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from '../api/client'
import { usePageTitle } from '../hooks/usePageTitle'
import Button from '../components/Button'
import Card from '../components/Card'
import StatCard from '../components/StatCard'
import Badge from '../components/Badge'

interface SseUpdate {
  type: 'price' | 'fx'
  key: string
  kind: 'current' | 'history'
}

interface EventCounts {
  price_current: number
  price_history: number
  fx_current: number
}

const MAX_RECENT = 50

export default function SimulatorPage() {
  usePageTitle('Price Simulator')
  const queryClient = useQueryClient()

  const { data: stats, isLoading } = useQuery({
    queryKey: ['simulator-status'],
    queryFn: () => api.getSimulatorStatus(),
    refetchInterval: (query) => query.state.data?.running ? 1000 : false,
  })

  const startMutation = useMutation({
    mutationFn: () => api.startSimulator(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['simulator-status'] })
    },
  })

  const stopMutation = useMutation({
    mutationFn: () => api.stopSimulator(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['simulator-status'] })
    },
  })

  const running = stats?.running ?? false
  const busy = startMutation.isPending || stopMutation.isPending

  // SSE connection for live events
  const [sseConnected, setSseConnected] = useState(false)
  const [recentEvents, setRecentEvents] = useState<SseUpdate[]>([])
  const [counts, setCounts] = useState<EventCounts>({ price_current: 0, price_history: 0, fx_current: 0 })
  const eventSourceRef = useRef<EventSource | null>(null)
  // Accumulate events in a ref and flush to state periodically to avoid
  // re-rendering on every single SSE message (can be 100s/sec).
  const pendingRef = useRef<SseUpdate[]>([])
  const pendingCountsRef = useRef<EventCounts>({ price_current: 0, price_history: 0, fx_current: 0 })

  const connectSse = useCallback(() => {
    const token = api.getAccessToken()
    if (!token) return

    // EventSource doesn't support headers, so pass token as query param.
    // The backend also accepts ?token= for SSE connections.
    const es = new EventSource(`/v1/admin/simulator/events?token=${encodeURIComponent(token)}`)
    eventSourceRef.current = es

    es.onopen = () => setSseConnected(true)
    es.onerror = () => setSseConnected(false)
    es.addEventListener('update', (e: MessageEvent) => {
      try {
        const update: SseUpdate = JSON.parse(e.data)
        pendingRef.current.push(update)
        if (update.type === 'price' && update.kind === 'current') {
          pendingCountsRef.current.price_current++
        } else if (update.type === 'price' && update.kind === 'history') {
          pendingCountsRef.current.price_history++
        } else if (update.type === 'fx') {
          pendingCountsRef.current.fx_current++
        }
      } catch { /* ignore parse errors */ }
    })
  }, [])

  const disconnectSse = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close()
      eventSourceRef.current = null
    }
    setSseConnected(false)
  }, [])

  // Flush pending events to React state every 500ms
  useEffect(() => {
    const interval = setInterval(() => {
      const pending = pendingRef.current
      if (pending.length === 0) return

      pendingRef.current = []
      setRecentEvents(prev => [...pending, ...prev].slice(0, MAX_RECENT))
      setCounts(prev => ({
        price_current: prev.price_current + pendingCountsRef.current.price_current,
        price_history: prev.price_history + pendingCountsRef.current.price_history,
        fx_current: prev.fx_current + pendingCountsRef.current.fx_current,
      }))
      pendingCountsRef.current = { price_current: 0, price_history: 0, fx_current: 0 }
    }, 500)
    return () => clearInterval(interval)
  }, [])

  // Cleanup on unmount
  useEffect(() => () => disconnectSse(), [disconnectSse])

  function formatNumber(n: number | undefined): string {
    if (n === undefined) return '-'
    return n.toLocaleString()
  }

  return (
    <div className="ds-page">
      <div className="ds-page__header">
        <div>
          <h1 className="ds-page__title">Price Simulator</h1>
          <p className="ds-text--secondary" style={{ marginTop: 'var(--spacing-xs)' }}>
            Simulates price movements for development and testing
          </p>
        </div>
        <div className="ds-page__actions">
          <Badge variant={running ? 'success' : 'neutral'}>
            {running ? 'Running' : 'Stopped'}
          </Badge>
          {running ? (
            <Button variant="danger" onClick={() => stopMutation.mutate()} disabled={busy}>
              Stop
            </Button>
          ) : (
            <Button variant="primary" onClick={() => startMutation.mutate()} disabled={busy}>
              Start
            </Button>
          )}
        </div>
      </div>

      {isLoading ? (
        <p className="ds-text--secondary">Loading...</p>
      ) : stats ? (
        <>
          <div className="ds-stat-grid">
            <StatCard label="Ticks" value={formatNumber(stats.ticks)} />
            <StatCard label="Price Updates" value={formatNumber(stats.price_updates)} />
            <StatCard label="FX Updates" value={formatNumber(stats.fx_updates)} />
            <StatCard label="History Updates" value={formatNumber(stats.history_updates)} />
            <StatCard label="Errors" value={formatNumber(stats.errors)} />
          </div>

          <Card>
            <h3 style={{ marginBottom: 'var(--spacing-sm)' }}>Configuration</h3>
            <table className="ds-table">
              <tbody>
                <tr>
                  <td className="ds-table__cell">Tick interval</td>
                  <td className="ds-table__cell ds-table__cell--numeric">100 ms</td>
                </tr>
                <tr>
                  <td className="ds-table__cell">Prices per tick</td>
                  <td className="ds-table__cell ds-table__cell--numeric">50</td>
                </tr>
                <tr>
                  <td className="ds-table__cell">FX pairs per tick</td>
                  <td className="ds-table__cell ds-table__cell--numeric">10</td>
                </tr>
                <tr>
                  <td className="ds-table__cell">History updates per tick</td>
                  <td className="ds-table__cell ds-table__cell--numeric">10</td>
                </tr>
              </tbody>
            </table>
          </Card>

          <Card>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 'var(--spacing-sm)' }}>
              <h3>Live PubSub Events</h3>
              <div className="ds-page__actions">
                <Badge variant={sseConnected ? 'success' : 'neutral'}>
                  {sseConnected ? 'Connected' : 'Disconnected'}
                </Badge>
                {sseConnected ? (
                  <Button variant="outline" onClick={disconnectSse}>Disconnect</Button>
                ) : (
                  <Button variant="outline" onClick={connectSse}>Connect</Button>
                )}
              </div>
            </div>

            {(counts.price_current > 0 || counts.fx_current > 0 || counts.price_history > 0) && (
              <div className="ds-stat-grid" style={{ marginBottom: 'var(--spacing-md)' }}>
                <StatCard label="Price (current)" value={formatNumber(counts.price_current)} />
                <StatCard label="Price (history)" value={formatNumber(counts.price_history)} />
                <StatCard label="FX (current)" value={formatNumber(counts.fx_current)} />
              </div>
            )}

            {recentEvents.length > 0 ? (
              <div style={{ maxHeight: '300px', overflow: 'auto' }}>
                <table className="ds-table">
                  <thead>
                    <tr>
                      <th className="ds-table__cell">Type</th>
                      <th className="ds-table__cell">Key</th>
                      <th className="ds-table__cell">Kind</th>
                    </tr>
                  </thead>
                  <tbody>
                    {recentEvents.map((ev, i) => (
                      <tr key={i}>
                        <td className="ds-table__cell">
                          <Badge variant={ev.type === 'price' ? 'info' : 'warning'}>{ev.type}</Badge>
                        </td>
                        <td className="ds-table__cell ds-text--mono">{ev.key}</td>
                        <td className="ds-table__cell">{ev.kind}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <p className="ds-text--secondary">
                {sseConnected ? 'Waiting for events... Start the simulator to see updates.' : 'Click Connect to subscribe to live PubSub events.'}
              </p>
            )}
          </Card>
        </>
      ) : null}
    </div>
  )
}
