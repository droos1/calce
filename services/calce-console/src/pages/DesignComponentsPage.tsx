import { useState, useMemo } from 'react'
import type { ColumnDef } from '@tanstack/react-table'
import Button from '../components/Button'
import Input from '../components/Input'
import SearchInput from '../components/SearchInput'
import Badge from '../components/Badge'
import Card from '../components/Card'
import StatCard from '../components/StatCard'
import DataTable from '../components/DataTable'
import Modal from '../components/Modal'
import Pagination from '../components/Pagination'
import Tabs from '../components/Tabs'
import Spinner from '../components/Spinner'
import EmptyState from '../components/EmptyState'
import { IconPlus } from '../components/icons'
import { usePageTitle } from '../hooks/usePageTitle'

interface SampleRow {
  id: number
  name: string
  ticker: string
  type: string
  currency: string
}

const sampleData: SampleRow[] = [
  { id: 1, name: 'Apple Inc.', ticker: 'AAPL', type: 'equity', currency: 'USD' },
  { id: 2, name: 'Microsoft Corp.', ticker: 'MSFT', type: 'equity', currency: 'USD' },
  { id: 3, name: 'NVIDIA Corp.', ticker: 'NVDA', type: 'equity', currency: 'USD' },
]

export default function DesignComponentsPage() {
  usePageTitle('Design Components')
  const [modalOpen, setModalOpen] = useState(false)
  const [demoPage, setDemoPage] = useState(3)
  const [demoTab, setDemoTab] = useState('First')
  const [searchValue, setSearchValue] = useState('')

  const sampleColumns = useMemo<ColumnDef<SampleRow, unknown>[]>(
    () => [
      { accessorKey: 'name', header: 'Name' },
      {
        accessorKey: 'ticker',
        header: 'Ticker',
        cell: ({ getValue }) => (
          <span className="ds-text--mono">{getValue<string>()}</span>
        ),
      },
      {
        accessorKey: 'type',
        header: 'Type',
        cell: ({ getValue }) => <Badge>{getValue<string>()}</Badge>,
      },
      { accessorKey: 'currency', header: 'Currency' },
    ],
    []
  )

  return (
    <div className="ds-page">
      <div className="ds-page__header">
        <div>
          <h1 className="ds-page__title">Components</h1>
          <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)', marginTop: 'var(--spacing-xs)' }}>
            Interactive component library
          </div>
        </div>
      </div>

      {/* Buttons */}
      <Card header="Buttons">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-xl)' }}>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Variants</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)', flexWrap: 'wrap' }}>
              <Button variant="primary">Primary</Button>
              <Button variant="secondary">Secondary</Button>
              <Button variant="outline">Outline</Button>
              <Button variant="ghost">Ghost</Button>
              <Button variant="danger">Danger</Button>
            </div>
          </div>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Sizes</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)' }}>
              <Button size="sm">Small</Button>
              <Button size="md">Medium</Button>
              <Button size="lg">Large</Button>
            </div>
          </div>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Other</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)', flexWrap: 'wrap' }}>
              <Button variant="primary"><IconPlus size={14} /> With Icon</Button>
              <Button disabled>Disabled</Button>
              <div style={{ width: 200 }}>
                <Button fullWidth variant="primary">Full Width</Button>
              </div>
            </div>
          </div>
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Inputs */}
      <Card header="Inputs">
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 'var(--spacing-xl)', maxWidth: 600 }}>
          <div className="ds-form-group">
            <label className="ds-label">Default</label>
            <Input placeholder="Enter text..." />
          </div>
          <div className="ds-form-group">
            <label className="ds-label">With value</label>
            <Input defaultValue="Hello world" />
          </div>
          <div className="ds-form-group">
            <label className="ds-label">Error</label>
            <Input error placeholder="Invalid input" />
          </div>
          <div className="ds-form-group">
            <label className="ds-label">Disabled</label>
            <Input disabled value="Cannot edit" />
          </div>
          <div className="ds-form-group" style={{ gridColumn: '1 / -1' }}>
            <label className="ds-label">Search Input</label>
            <SearchInput value={searchValue} onChange={setSearchValue} placeholder="Search something..." />
          </div>
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Badges */}
      <Card header="Badges">
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)', flexWrap: 'wrap' }}>
          <Badge variant="success">Success</Badge>
          <Badge variant="warning">Warning</Badge>
          <Badge variant="error">Error</Badge>
          <Badge variant="info">Info</Badge>
          <Badge variant="neutral">Neutral</Badge>
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Cards & StatCards */}
      <Card header="Cards">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-xl)' }}>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Standard Card</div>
            <Card header="Card Title">
              <p style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>
                This is a standard card with a header and body content. Cards are used to group related information together.
              </p>
            </Card>
          </div>
          <div>
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginBottom: 'var(--spacing-md)', textTransform: 'uppercase', letterSpacing: '0.04em', fontWeight: 600 }}>Stat Cards</div>
            <div className="ds-grid ds-grid--cols-3">
              <StatCard label="Revenue" value="$1,234,567" change="+12.5%" changeDirection="positive" />
              <StatCard label="Users" value="8,432" change="-3.2%" changeDirection="negative" />
              <StatCard label="Instruments" value="1,205" />
            </div>
          </div>
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Tables */}
      <Card header="Tables">
        <DataTable data={sampleData} columns={sampleColumns} />
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Modals */}
      <Card header="Modals">
        <Button variant="primary" onClick={() => setModalOpen(true)}>
          Open Modal
        </Button>
        <Modal
          open={modalOpen}
          onClose={() => setModalOpen(false)}
          title="Sample Modal"
          footer={
            <>
              <Button variant="ghost" onClick={() => setModalOpen(false)}>Cancel</Button>
              <Button variant="primary" onClick={() => setModalOpen(false)}>Confirm</Button>
            </>
          }
        >
          <p>This is a modal dialog. It can contain any content including forms, text, or other components.</p>
        </Modal>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Pagination */}
      <Card header="Pagination">
        <Pagination page={demoPage} totalPages={10} onPageChange={setDemoPage} />
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Tabs */}
      <Card header="Tabs">
        <Tabs tabs={['First', 'Second', 'Third']} active={demoTab} onChange={setDemoTab} />
        <div style={{ padding: 'var(--spacing-xl) 0', fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>
          Active tab: {demoTab}
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Spinner */}
      <Card header="Spinner">
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-2xl)' }}>
          <div style={{ textAlign: 'center' }}>
            <Spinner size="sm" />
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginTop: 'var(--spacing-md)' }}>Small</div>
          </div>
          <div style={{ textAlign: 'center' }}>
            <Spinner size="md" />
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginTop: 'var(--spacing-md)' }}>Medium</div>
          </div>
          <div style={{ textAlign: 'center' }}>
            <Spinner size="lg" />
            <div style={{ fontSize: 'var(--font-size-xs)', color: 'var(--color-text-tertiary)', marginTop: 'var(--spacing-md)' }}>Large</div>
          </div>
        </div>
      </Card>

      <div style={{ height: 'var(--spacing-xl)' }} />

      {/* Empty State */}
      <Card header="Empty State">
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 'var(--spacing-xl)' }}>
          <div style={{ border: '1px dashed var(--color-border)', borderRadius: 'var(--radius-lg)' }}>
            <EmptyState title="No data found" description="There are no items matching your criteria." />
          </div>
          <div style={{ border: '1px dashed var(--color-border)', borderRadius: 'var(--radius-lg)' }}>
            <EmptyState
              title="No instruments"
              description="Get started by adding your first instrument."
              action={<Button variant="primary" size="sm"><IconPlus size={14} /> Add Instrument</Button>}
            />
          </div>
        </div>
      </Card>
    </div>
  )
}
