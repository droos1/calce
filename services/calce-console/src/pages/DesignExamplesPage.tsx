import { useState } from 'react'
import Button from '../components/Button'
import Input from '../components/Input'
import Card from '../components/Card'
import StatCard from '../components/StatCard'
import Modal from '../components/Modal'
import Badge from '../components/Badge'
import PriceChart from '../components/PriceChart'
import type { Price } from '../api/types'

// Generate sample price data for the chart
function generatePriceData(): Price[] {
  const data: Price[] = []
  let price = 142.5
  const start = new Date('2024-01-02')
  for (let i = 0; i < 120; i++) {
    const d = new Date(start)
    d.setDate(d.getDate() + i)
    if (d.getDay() === 0 || d.getDay() === 6) continue
    price += (Math.random() - 0.47) * 3
    data.push({ date: d.toISOString().slice(0, 10), price: Math.round(price * 100) / 100 })
  }
  return data
}

const priceData = generatePriceData()

export default function DesignExamplesPage() {
  const [loginOpen, setLoginOpen] = useState(false)

  return (
    <div className="ds-page">
      <div className="ds-page__header">
        <div>
          <h1 className="ds-page__title">Examples</h1>
          <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)', marginTop: 'var(--spacing-xs)' }}>
            Typical screens and dialogs composed from design system components
          </div>
        </div>
      </div>

      {/* Login Modal Example */}
      <Card header="Login Dialog">
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-xl)' }}>
          <Button variant="primary" onClick={() => setLoginOpen(true)}>
            Open Login Dialog
          </Button>
          <span style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-tertiary)' }}>
            Modal with form inputs, validation state, and actions
          </span>
        </div>
        <Modal
          open={loginOpen}
          onClose={() => setLoginOpen(false)}
          title="Sign in"
          footer={
            <>
              <Button variant="ghost" onClick={() => setLoginOpen(false)}>Cancel</Button>
              <Button variant="primary" onClick={() => setLoginOpen(false)}>Sign in</Button>
            </>
          }
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-lg)' }}>
            <div className="ds-form-group">
              <label className="ds-label" htmlFor="demo-email">Email</label>
              <Input id="demo-email" type="email" placeholder="you@example.com" />
            </div>
            <div className="ds-form-group">
              <label className="ds-label" htmlFor="demo-password">Password</label>
              <Input id="demo-password" type="password" placeholder="Password" />
            </div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)' }}>
              Forgot your password? Contact your administrator.
            </div>
          </div>
        </Modal>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Dashboard Example */}
      <Card header="Dashboard">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-xl)' }}>
          {/* KPI row */}
          <div className="ds-grid ds-grid--cols-4">
            <StatCard label="Portfolio Value" value="$2,847,320" change="+4.2%" changeDirection="positive" />
            <StatCard label="Daily P&L" value="$12,450" change="+0.44%" changeDirection="positive" />
            <StatCard label="Positions" value="47" />
            <StatCard label="Cash Balance" value="$324,180" change="-2.1%" changeDirection="negative" />
          </div>

          {/* Chart + sidebar */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 280px', gap: 'var(--spacing-xl)' }}>
            <Card header="AAPL — Apple Inc.">
              <PriceChart data={priceData} />
            </Card>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-lg)' }}>
              <Card header="Top Movers">
                <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-md)' }}>
                  {[
                    { ticker: 'NVDA', name: 'NVIDIA', change: '+5.3%', dir: 'positive' as const },
                    { ticker: 'TSLA', name: 'Tesla', change: '+3.1%', dir: 'positive' as const },
                    { ticker: 'META', name: 'Meta', change: '-2.8%', dir: 'negative' as const },
                    { ticker: 'AMZN', name: 'Amazon', change: '-1.4%', dir: 'negative' as const },
                  ].map((item) => (
                    <div key={item.ticker} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: 'var(--spacing-sm) 0' }}>
                      <div>
                        <span className="ds-text--mono" style={{ fontSize: 'var(--font-size-sm)', fontWeight: 600 }}>{item.ticker}</span>
                        <span style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginLeft: 'var(--spacing-md)' }}>{item.name}</span>
                      </div>
                      <Badge variant={item.dir === 'positive' ? 'success' : 'error'}>{item.change}</Badge>
                    </div>
                  ))}
                </div>
              </Card>
              <Card header="Allocation">
                <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-sm)' }}>
                  {[
                    { label: 'Equities', pct: 68 },
                    { label: 'Fixed Income', pct: 18 },
                    { label: 'Cash', pct: 11 },
                    { label: 'Alternatives', pct: 3 },
                  ].map((item) => (
                    <div key={item.label} style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)' }}>
                      <span style={{ fontSize: 'var(--font-size-xs)', width: 80, color: 'var(--color-text-secondary)' }}>{item.label}</span>
                      <div style={{ flex: 1, height: 6, background: 'var(--color-hover)', borderRadius: 'var(--radius-full)' }}>
                        <div style={{ width: `${item.pct}%`, height: '100%', background: 'var(--color-primary)', borderRadius: 'var(--radius-full)' }} />
                      </div>
                      <span className="ds-text--mono" style={{ fontSize: 'var(--font-size-xs)', width: 30, textAlign: 'right', color: 'var(--color-text-tertiary)' }}>{item.pct}%</span>
                    </div>
                  ))}
                </div>
              </Card>
            </div>
          </div>
        </div>
      </Card>
    </div>
  )
}
