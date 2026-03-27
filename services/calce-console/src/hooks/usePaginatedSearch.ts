import { useState, useEffect } from 'react'

const DEBOUNCE_MS = 300

export function usePaginatedSearch(pageSize: number) {
  const [page, setPage] = useState(1)
  const [search, setSearch] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')

  useEffect(() => {
    const timeout = setTimeout(() => {
      setDebouncedSearch(search)
      setPage(1)
    }, DEBOUNCE_MS)
    return () => clearTimeout(timeout)
  }, [search])

  const offset = (page - 1) * pageSize

  return {
    page,
    setPage,
    search,
    setSearch,
    debouncedSearch,
    offset,
    pageSize,
    totalPages: (total: number) => Math.ceil(total / pageSize),
  }
}
