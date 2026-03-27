import { NavLink } from 'react-router'
import { useAuth } from '../auth/AuthContext'
import {
  IconDashboard,
  IconBuilding,
  IconUsers,
  IconChart,
  IconPalette,
  IconLogout,
  IconSun,
  IconMoon,
} from './icons'
import { useState, useEffect } from 'react'

function getTheme(): 'light' | 'dark' {
  return (document.documentElement.getAttribute('data-theme') as 'light' | 'dark') || 'light'
}

function Sidebar() {
  const { user, logout } = useAuth()
  const [theme, setTheme] = useState<'light' | 'dark'>(getTheme)

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
  }, [theme])

  const toggleTheme = () => setTheme((t) => (t === 'light' ? 'dark' : 'light'))

  const linkClass = ({ isActive }: { isActive: boolean }) =>
    ['ds-sidebar__item', isActive && 'ds-sidebar__item--active'].filter(Boolean).join(' ')

  return (
    <aside className="ds-sidebar">
      <div className="ds-sidebar__logo">Calce</div>
      <nav className="ds-sidebar__nav">
        <div className="ds-sidebar__section">
          <div className="ds-sidebar__section-title">Overview</div>
          <NavLink to="/dashboard" className={linkClass}>
            <IconDashboard /> Dashboard
          </NavLink>
        </div>
        <div className="ds-sidebar__section">
          <div className="ds-sidebar__section-title">Data</div>
          <NavLink to="/organizations" className={linkClass}>
            <IconBuilding /> Organizations
          </NavLink>
          <NavLink to="/users" className={linkClass}>
            <IconUsers /> Users
          </NavLink>
          <NavLink to="/instruments" className={linkClass}>
            <IconChart /> Instruments
          </NavLink>
        </div>
        <div className="ds-sidebar__section">
          <div className="ds-sidebar__section-title">Design</div>
          <NavLink to="/design/system" className={linkClass}>
            <IconPalette /> System
          </NavLink>
          <NavLink to="/design/components" className={linkClass}>
            <IconDashboard /> Components
          </NavLink>
          <NavLink to="/design/examples" className={linkClass}>
            <IconChart /> Examples
          </NavLink>
        </div>
      </nav>
      <div className="ds-sidebar__user">
        <button className="ds-sidebar__item ds-sidebar__user-action" onClick={toggleTheme}>
          {theme === 'light' ? <IconMoon /> : <IconSun />}
        </button>
        <span className="ds-sidebar__user-name">
          {user?.email || 'Admin'}
        </span>
        <button className="ds-sidebar__item ds-sidebar__user-action" onClick={logout}>
          <IconLogout />
        </button>
      </div>
    </aside>
  )
}

export default Sidebar
