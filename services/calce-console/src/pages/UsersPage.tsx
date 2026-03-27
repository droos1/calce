import { useQuery } from '@tanstack/react-query'
import type { ColumnDef } from '@tanstack/react-table'
import { useMemo, useState, useEffect } from 'react'
import { api } from '../api/client'
import type { User } from '../api/types'
import DataTable from '../components/DataTable'
import SearchInput from '../components/SearchInput'
import Pagination from '../components/Pagination'
import Spinner from '../components/Spinner'

const PAGE_SIZE = 30

export default function UsersPage() {
  const [page, setPage] = useState(1)
  const [search, setSearch] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')

  useEffect(() => {
    const timeout = setTimeout(() => {
      setDebouncedSearch(search)
      setPage(1)
    }, 300)
    return () => clearTimeout(timeout)
  }, [search])

  const { data, isLoading } = useQuery({
    queryKey: ['users', { page, search: debouncedSearch, pageSize: PAGE_SIZE }],
    queryFn: () =>
      api.getUsers({
        offset: (page - 1) * PAGE_SIZE,
        limit: PAGE_SIZE,
        search: debouncedSearch || undefined,
      }),
  })

  const totalPages = data ? Math.ceil(data.total / PAGE_SIZE) : 0

  const columns = useMemo<ColumnDef<User, unknown>[]>(
    () => [
      {
        accessorKey: 'email',
        header: 'Email',
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
        accessorKey: 'organization_id',
        header: 'Organization',
        cell: ({ getValue }) => getValue<string | null>() || '-',
      },
      {
        accessorKey: 'trade_count',
        header: 'Trades',
      },
    ],
    []
  )

  return (
    <div className="ds-page">
      <div className="ds-page__header">
        <h1 className="ds-page__title">Users</h1>
        <div className="ds-page__actions">
          <SearchInput
            value={search}
            onChange={setSearch}
            placeholder="Search users..."
          />
        </div>
      </div>
      {isLoading ? (
        <Spinner size="lg" center />
      ) : data ? (
        <>
          <DataTable data={data.items} columns={columns} />
          {totalPages > 1 && (
            <Pagination page={page} totalPages={totalPages} onPageChange={setPage} />
          )}
        </>
      ) : null}
    </div>
  )
}
