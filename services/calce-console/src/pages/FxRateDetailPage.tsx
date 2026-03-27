import { useParams, Link } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import type { ColumnDef } from '@tanstack/react-table'
import { useMemo } from 'react'
import { api } from '../api/client'
import type { Price } from '../api/types'
import { IconChevronLeft } from '../components/icons'
import Badge from '../components/Badge'
import DataTable from '../components/DataTable'
import PriceChart from '../components/PriceChart'
import Spinner from '../components/Spinner'
import { usePageTitle } from '../hooks/usePageTitle'

export default function FxRateDetailPage() {
  const { from, to } = useParams()
  const pair = `${from}/${to}`

  usePageTitle(pair)

  const { data: history, isLoading } = useQuery({
    queryKey: ['fx-rate-history', from, to],
    queryFn: () => {
      const toDate = new Date().toISOString().slice(0, 10)
      const fromDate = new Date(Date.now() - 5 * 365 * 24 * 60 * 60 * 1000)
        .toISOString()
        .slice(0, 10)
      return api.getFxRateHistory(from!, to!, { from: fromDate, to: toDate })
    },
    enabled: !!from && !!to,
  })

  const latestRate = history && history.length > 0 ? history[history.length - 1] : null

  const columns = useMemo<ColumnDef<Price, unknown>[]>(
    () => [
      {
        accessorKey: 'date',
        header: 'Date',
        cell: ({ getValue }) =>
          new Date(getValue<string>()).toLocaleDateString(),
      },
      {
        accessorKey: 'price',
        header: 'Rate',
        cell: ({ getValue }) => (
          <span className="ds-text--mono">
            {getValue<number>().toFixed(4)}
          </span>
        ),
      },
    ],
    []
  )

  return (
    <div className="ds-page">
      <Link to="/fx-rates" className="ds-back-link">
        <IconChevronLeft size={12} /> Back to FX Rates
      </Link>
      <div className="ds-page__header">
        <div className="ds-page__actions">
          <h1 className="ds-page__title">{pair}</h1>
          <Badge variant="neutral">{from}</Badge>
          <Badge variant="neutral">{to}</Badge>
        </div>
      </div>

      <div className="ds-kv-inline ds-mt-md">
        <span className="ds-kv-inline__item">
          <span className="ds-kv-inline__label">From</span>
          <span>{from}</span>
        </span>
        <span className="ds-kv-inline__item">
          <span className="ds-kv-inline__label">To</span>
          <span>{to}</span>
        </span>
        {latestRate && (
          <>
            <span className="ds-kv-inline__item">
              <span className="ds-kv-inline__label">Latest Rate</span>
              <span className="ds-text--mono">{latestRate.price.toFixed(4)}</span>
            </span>
            <span className="ds-kv-inline__item">
              <span className="ds-kv-inline__label">Latest Date</span>
              <span>{new Date(latestRate.date).toLocaleDateString()}</span>
            </span>
          </>
        )}
        {history && (
          <span className="ds-kv-inline__item">
            <span className="ds-kv-inline__label">Data Points</span>
            <span>{history.length.toLocaleString()}</span>
          </span>
        )}
      </div>

      <div className="ds-chart-container ds-mt-lg">
        {isLoading ? (
          <Spinner size="lg" center />
        ) : history && history.length > 0 ? (
          <PriceChart data={history} />
        ) : (
          <p className="ds-text--secondary">No rate history available.</p>
        )}
      </div>

      {!isLoading && history && history.length > 0 && (
        <DataTable data={history} columns={columns} />
      )}
    </div>
  )
}
