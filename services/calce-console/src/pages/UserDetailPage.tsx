import { useParams, Link } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import type { ColumnDef } from '@tanstack/react-table'
import { useMemo } from 'react'
import { api } from '../api/client'
import type { AccountSummary, PositionSummary } from '../api/types'
import { IconChevronLeft } from '../components/icons'
import Card from '../components/Card'
import Badge from '../components/Badge'
import DataTable from '../components/DataTable'
import Spinner from '../components/Spinner'
import { usePageTitle } from '../hooks/usePageTitle'

export default function UserDetailPage() {
  const { id } = useParams()

  const { data: user, isLoading: userLoading, error: userError } = useQuery({
    queryKey: ['user', id],
    queryFn: () => api.getUser(id!),
    enabled: !!id,
  })

  usePageTitle(user?.name || user?.email || id || 'User')

  const { data: accounts, isLoading: accountsLoading } = useQuery({
    queryKey: ['user-accounts', id],
    queryFn: () => api.getUserAccounts(id!),
    enabled: !!id,
  })

  const { data: positions, isLoading: positionsLoading } = useQuery({
    queryKey: ['user-positions', id],
    queryFn: () => api.getUserPositions(id!),
    enabled: !!id,
  })

  const accountColumns = useMemo<ColumnDef<AccountSummary, unknown>[]>(
    () => [
      { accessorKey: 'label', header: 'Label' },
      { accessorKey: 'currency', header: 'Currency' },
      { accessorKey: 'trade_count', header: 'Trades' },
    ],
    []
  )

  const positionColumns = useMemo<ColumnDef<PositionSummary, unknown>[]>(
    () => [
      { accessorKey: 'instrument_id', header: 'Instrument' },
      {
        accessorKey: 'quantity',
        header: 'Quantity',
        cell: ({ getValue }) => (
          <span className="ds-text--mono">
            {getValue<number>().toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 4,
            })}
          </span>
        ),
      },
      { accessorKey: 'currency', header: 'Currency' },
      { accessorKey: 'trade_count', header: 'Trades' },
    ],
    []
  )

  if (userLoading) {
    return (
      <div className="ds-page">
        <Spinner size="lg" center />
      </div>
    )
  }

  if (userError || !user) {
    return (
      <div className="ds-page">
        <Link to="/users" className="ds-back-link">
          <IconChevronLeft size={12} /> Back to Users
        </Link>
        <p className="ds-text--secondary">{userError?.message || 'User not found.'}</p>
      </div>
    )
  }

  return (
    <div className="ds-page">
      <Link to="/users" className="ds-back-link">
        <IconChevronLeft size={12} /> Back to Users
      </Link>
      <div className="ds-page__header">
        <div className="ds-page__actions">
          <h1 className="ds-page__title">{user.name || user.id}</h1>
          {user.organization_name && (
            <Badge variant="neutral">{user.organization_name}</Badge>
          )}
        </div>
      </div>

      <Card header="User Details">
        <div className="ds-kv-grid">
          <span className="ds-kv-grid__label">ID</span>
          <span className="ds-text--mono">{user.id}</span>
          <span className="ds-kv-grid__label">Name</span>
          <span>{user.name || '-'}</span>
          <span className="ds-kv-grid__label">Email</span>
          <span>{user.email || '-'}</span>
          <span className="ds-kv-grid__label">Organization</span>
          <span>{user.organization_name || '-'}</span>
          <span className="ds-kv-grid__label">Accounts</span>
          <span>{user.account_count}</span>
          <span className="ds-kv-grid__label">Trades</span>
          <span>{user.trade_count}</span>
        </div>
      </Card>

      <Card header="Accounts" className="ds-mt-xl">
        {accountsLoading ? (
          <Spinner size="md" center />
        ) : accounts && accounts.length > 0 ? (
          <DataTable data={accounts} columns={accountColumns} />
        ) : (
          <p className="ds-text--secondary">No accounts.</p>
        )}
      </Card>

      <Card header="Positions" className="ds-mt-xl">
        {positionsLoading ? (
          <Spinner size="md" center />
        ) : positions && positions.length > 0 ? (
          <DataTable data={positions} columns={positionColumns} />
        ) : (
          <p className="ds-text--secondary">No positions.</p>
        )}
      </Card>
    </div>
  )
}
