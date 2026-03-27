import { Link } from 'react-router'

export default function NotFoundPage() {
  return (
    <div className="ds-page">
      <div className="ds-empty">
        <div className="ds-empty__title">404</div>
        <div className="ds-empty__description">
          This page doesn't exist.
        </div>
        <div className="ds-empty__action">
          <Link to="/dashboard" className="ds-btn ds-btn--primary">Back to Dashboard</Link>
        </div>
      </div>
    </div>
  )
}
