import { useQuery } from '@tanstack/react-query'
import type { ColumnDef } from '@tanstack/react-table'
import { useMemo } from 'react'
import { api } from '../api/client'
import type { Organization } from '../api/types'
import DataTable from '../components/DataTable'
import Spinner from '../components/Spinner'

export default function OrganizationsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ['organizations'],
    queryFn: () => api.getOrganizations(),
  })

  const columns = useMemo<ColumnDef<Organization, unknown>[]>(
    () => [
      {
        accessorKey: 'name',
        header: 'Name',
        cell: ({ getValue }) => getValue<string | null>() || '-',
      },
      {
        accessorKey: 'id',
        header: 'ID',
        cell: ({ getValue }) => (
          <span className="ds-text--mono">{getValue<string>()}</span>
        ),
      },
      {
        accessorKey: 'created_at',
        header: 'Created',
        cell: ({ getValue }) =>
          new Date(getValue<string>()).toLocaleDateString(),
      },
    ],
    []
  )

  return (
    <div className="ds-page">
      <div className="ds-page__header">
        <h1 className="ds-page__title">Organizations</h1>
      </div>
      {isLoading ? (
        <Spinner size="lg" center />
      ) : data ? (
        <DataTable data={data} columns={columns} />
      ) : null}
    </div>
  )
}
