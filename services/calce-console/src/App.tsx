import { Routes, Route, Navigate } from 'react-router'
import { ProtectedRoute } from './auth/AuthContext'
import AppLayout from './layouts/AppLayout'
import LoginPage from './pages/LoginPage'
import DashboardPage from './pages/DashboardPage'
import OrganizationsPage from './pages/OrganizationsPage'
import UsersPage from './pages/UsersPage'
import InstrumentsPage from './pages/InstrumentsPage'
import InstrumentDetailPage from './pages/InstrumentDetailPage'
import DesignSystemPage from './pages/DesignSystemPage'
import DesignComponentsPage from './pages/DesignComponentsPage'
import DesignExamplesPage from './pages/DesignExamplesPage'
import NotFoundPage from './pages/NotFoundPage'

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route element={<ProtectedRoute />}>
        <Route element={<AppLayout />}>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<DashboardPage />} />
          <Route path="/organizations" element={<OrganizationsPage />} />
          <Route path="/users" element={<UsersPage />} />
          <Route path="/instruments" element={<InstrumentsPage />} />
          <Route path="/instruments/:id" element={<InstrumentDetailPage />} />
          <Route path="/design/system" element={<DesignSystemPage />} />
          <Route path="/design/components" element={<DesignComponentsPage />} />
          <Route path="/design/examples" element={<DesignExamplesPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Route>
    </Routes>
  )
}
