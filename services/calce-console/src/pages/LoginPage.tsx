import { useState, type FormEvent } from 'react'
import { useNavigate, Navigate } from 'react-router'
import { useAuth } from '../auth/AuthContext'
import Button from '../components/Button'
import Input from '../components/Input'

export default function LoginPage() {
  const { isAuthenticated, login } = useAuth()
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  if (isAuthenticated) {
    return <Navigate to="/dashboard" replace />
  }

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError(null)
    setLoading(true)
    try {
      await login(email, password)
      navigate('/dashboard')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="ds-login">
      <div className="ds-login__card">
        <div className="ds-login__title">Calce Console</div>
        <div className="ds-login__subtitle">Sign in to your account</div>
        {error && <div className="ds-login__error">{error}</div>}
        <form onSubmit={handleSubmit}>
          <div className="ds-form-group">
            <label className="ds-label" htmlFor="email">Email</label>
            <Input
              id="email"
              type="email"
              placeholder="you@example.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>
          <div className="ds-form-group">
            <label className="ds-label" htmlFor="password">Password</label>
            <Input
              id="password"
              type="password"
              placeholder="Password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>
          <Button type="submit" variant="primary" fullWidth disabled={loading}>
            {loading ? 'Signing in...' : 'Sign in'}
          </Button>
        </form>
      </div>
    </div>
  )
}
