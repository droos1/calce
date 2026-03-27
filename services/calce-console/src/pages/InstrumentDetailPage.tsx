import { useParams, Link } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import type { ColumnDef } from '@tanstack/react-table'
import { useMemo, useState } from 'react'
import { api } from '../api/client'
import type { Price } from '../api/types'
import { IconChevronLeft } from '../components/icons'
import Card from '../components/Card'
import Badge from '../components/Badge'
import Tabs from '../components/Tabs'
import DataTable from '../components/DataTable'
import PriceChart from '../components/PriceChart'
import Spinner from '../components/Spinner'

export default function InstrumentDetailPage() {
  const { id } = useParams()
  const [activeTab, setActiveTab] = useState('Overview')

  const { data: instrumentsData, isLoading: instrumentsLoading } = useQuery({
    queryKey: ['instruments', { page: 1, search: '', pageSize: 1000 }],
    queryFn: () => api.getInstruments({ limit: 1000 }),
  })

  const instrument = instrumentsData?.items.find((i) => i.id === id)

  const { data: prices, isLoading: pricesLoading } = useQuery({
    queryKey: ['instrument-prices', id],
    queryFn: () => api.getInstrumentPrices(id!),
    enabled: activeTab === 'Price History',
  })

  const priceColumns = useMemo<ColumnDef<Price, unknown>[]>(
    () => [
      {
        accessorKey: 'date',
        header: 'Date',
        cell: ({ getValue }) =>
          new Date(getValue<string>()).toLocaleDateString(),
      },
      {
        accessorKey: 'price',
        header: 'Price',
        cell: ({ getValue }) => (
          <span className="ds-text--mono">
            {getValue<number>().toFixed(2)}
          </span>
        ),
      },
    ],
    []
  )

  if (instrumentsLoading) {
    return (
      <div className="ds-page">
        <Spinner size="lg" center />
      </div>
    )
  }

  if (!instrument) {
    return (
      <div className="ds-page">
        <p>Instrument not found.</p>
      </div>
    )
  }

  return (
    <div className="ds-page">
      <Link to="/instruments" className="ds-back-link">
        <IconChevronLeft size={12} /> Back to Instruments
      </Link>
      <div className="ds-page__header">
        <div className="ds-page__actions">
          <h1 className="ds-page__title">{instrument.name || instrument.id}</h1>
          <Badge variant="neutral">{instrument.id}</Badge>
          <Badge variant="info">{instrument.instrument_type}</Badge>
        </div>
      </div>

      <Tabs tabs={['Overview', 'Price History']} active={activeTab} onChange={setActiveTab} />

      {activeTab === 'Overview' && (
        <Card header="Instrument Details" className="ds-mt-xl">
          <div className="ds-kv-grid">
            <span className="ds-kv-grid__label">ID</span>
            <span className="ds-text--mono">{instrument.id}</span>
            <span className="ds-kv-grid__label">Name</span>
            <span>{instrument.name || '-'}</span>
            <span className="ds-kv-grid__label">Type</span>
            <span>{instrument.instrument_type}</span>
            <span className="ds-kv-grid__label">Currency</span>
            <span>{instrument.currency}</span>
          </div>
        </Card>
      )}

      {activeTab === 'Price History' && (
        <>
          <div className="ds-chart-container ds-mt-xl">
            {pricesLoading ? (
              <Spinner size="lg" center />
            ) : prices ? (
              <PriceChart data={prices} />
            ) : null}
          </div>
          {!pricesLoading && prices && (
            <DataTable data={prices} columns={priceColumns} />
          )}
        </>
      )}
    </div>
  )
}
