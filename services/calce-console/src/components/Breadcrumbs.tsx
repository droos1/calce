import { Link } from 'react-router'
import { IconChevronRight } from './icons'

export interface BreadcrumbItem {
  label: string
  to?: string
}

interface BreadcrumbsProps {
  items: BreadcrumbItem[]
}

function Breadcrumbs({ items }: BreadcrumbsProps) {
  return (
    <div className="ds-breadcrumbs">
      {items.map((item, i) => (
        <span key={i} style={{ display: 'contents' }}>
          {i > 0 && (
            <span className="ds-breadcrumbs__separator">
              <IconChevronRight size={10} />
            </span>
          )}
          <span className="ds-breadcrumbs__item">
            {item.to ? <Link to={item.to}>{item.label}</Link> : item.label}
          </span>
        </span>
      ))}
    </div>
  )
}

export default Breadcrumbs
