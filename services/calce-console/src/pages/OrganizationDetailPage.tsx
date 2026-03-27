import { useParams, Link } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import { api } from '../api/client'
import { IconChevronLeft } from '../components/icons'
import Card from '../components/Card'
import Spinner from '../components/Spinner'
import { usePageTitle } from '../hooks/usePageTitle'

export default function OrganizationDetailPage() {
  const { id } = useParams()

  const { data: org, isLoading, error } = useQuery({
    queryKey: ['organization', id],
    queryFn: () => api.getOrganization(id!),
    enabled: !!id,
  })

  usePageTitle(org?.name || id || 'Organization')

  if (isLoading) {
    return (
      <div className="ds-page">
        <Spinner size="lg" center />
      </div>
    )
  }

  if (error || !org) {
    return (
      <div className="ds-page">
        <Link to="/organizations" className="ds-back-link">
          <IconChevronLeft size={12} /> Back to Organizations
        </Link>
        <p className="ds-text--secondary">{error?.message || 'Organization not found.'}</p>
      </div>
    )
  }

  return (
    <div className="ds-page">
      <Link to="/organizations" className="ds-back-link">
        <IconChevronLeft size={12} /> Back to Organizations
      </Link>
      <div className="ds-page__header">
        <h1 className="ds-page__title">{org.name || org.id}</h1>
      </div>

      <Card header="Organization Details">
        <div className="ds-kv-grid">
          <span className="ds-kv-grid__label">ID</span>
          <span className="ds-text--mono">{org.id}</span>
          <span className="ds-kv-grid__label">Name</span>
          <span>{org.name || '-'}</span>
          <span className="ds-kv-grid__label">Users</span>
          <span>
            <Link to={`/users?organization_id=${org.id}`} className="ds-link">
              {org.user_count}
            </Link>
          </span>
          <span className="ds-kv-grid__label">Created</span>
          <span>{new Date(org.created_at).toLocaleDateString()}</span>
        </div>
      </Card>
    </div>
  )
}
