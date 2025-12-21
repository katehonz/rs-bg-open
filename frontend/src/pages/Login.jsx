import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

function Login() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [showRecovery, setShowRecovery] = useState(false);
  const [recoveryCode, setRecoveryCode] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const navigate = useNavigate();
  const { login } = useAuth();

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
      });

      if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || 'Грешка при вход');
      }

      const data = await response.json();
      login(data.token, data.user);
      navigate('/dashboard');
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleRecoverySubmit = async (e) => {
    e.preventDefault();
    setError('');

    // Validation
    if (newPassword.length < 6) {
      setError('Новата парола трябва да е поне 6 символа');
      return;
    }

    if (newPassword !== confirmPassword) {
      setError('Паролите не съвпадат');
      return;
    }

    setLoading(true);

    try {
      // TODO: Implement backend endpoint for recovery code validation
      // For now, this is a placeholder that shows the UI flow
      const response = await fetch('/api/auth/recover-password', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username,
          recovery_code: recoveryCode.toUpperCase(),
          new_password: newPassword
        }),
      });

      if (!response.ok) {
        let errorMessage = 'Невалиден код за възстановяване';
        try {
          const text = await response.text();
          // Try to parse as JSON first
          try {
            const data = JSON.parse(text);
            errorMessage = data.error || errorMessage;
          } catch (e) {
            // If not JSON, use the text as-is
            if (text) errorMessage = text;
          }
        } catch (e) {
          // If we can't read the response at all, use default message
        }
        throw new Error(errorMessage);
      }

      alert('Паролата е променена успешно! Моля, влезте с новата парола.');
      setShowRecovery(false);
      setRecoveryCode('');
      setNewPassword('');
      setConfirmPassword('');
      setPassword('');
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="max-w-md w-full space-y-8 p-10 bg-white rounded-xl shadow-2xl">
        <div>
          <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Ръст-АС
          </h2>
          <p className="mt-2 text-center text-lg font-medium text-gray-700">
            Счетоводна Система
          </p>
          <p className="mt-1 text-center text-sm text-gray-600">
            {showRecovery ? 'Възстановяване на парола' : 'Вход в системата'}
          </p>
        </div>

        {!showRecovery ? (
        <form className="mt-8 space-y-6" onSubmit={handleSubmit}>
          {error && (
            <div className="rounded-md bg-red-50 p-4">
              <div className="flex">
                <div className="ml-3">
                  <h3 className="text-sm font-medium text-red-800">{error}</h3>
                </div>
              </div>
            </div>
          )}
          <div className="rounded-md shadow-sm -space-y-px">
            <div>
              <label htmlFor="username" className="sr-only">
                Потребителско име
              </label>
              <input
                id="username"
                name="username"
                type="text"
                required
                className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                placeholder="Потребителско име"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={loading}
              />
            </div>
            <div>
              <label htmlFor="password" className="sr-only">
                Парола
              </label>
              <input
                id="password"
                name="password"
                type="password"
                required
                className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                placeholder="Парола"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={loading}
              />
            </div>
          </div>

          <div>
            <button
              type="submit"
              disabled={loading}
              className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-gray-400 disabled:cursor-not-allowed"
            >
              {loading ? 'Влизане...' : 'Вход'}
            </button>
          </div>

          <div className="text-center">
            <button
              type="button"
              onClick={() => setShowRecovery(true)}
              className="text-sm text-indigo-600 hover:text-indigo-500"
            >
              Забравена парола? Използвайте код за възстановяване
            </button>
          </div>
        </form>
        ) : (
        <form className="mt-8 space-y-6" onSubmit={handleRecoverySubmit}>
          {error && (
            <div className="rounded-md bg-red-50 p-4">
              <div className="flex">
                <div className="ml-3">
                  <h3 className="text-sm font-medium text-red-800">{error}</h3>
                </div>
              </div>
            </div>
          )}

          <div className="bg-blue-50 border border-blue-200 rounded-md p-4">
            <p className="text-sm text-blue-800">
              Въведете вашето потребителско име, кода за възстановяване и нова парола.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label htmlFor="recovery-username" className="sr-only">
                Потребителско име
              </label>
              <input
                id="recovery-username"
                name="username"
                type="text"
                required
                className="appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                placeholder="Потребителско име"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={loading}
              />
            </div>

            <div>
              <label htmlFor="recovery-code" className="sr-only">
                Код за възстановяване
              </label>
              <input
                id="recovery-code"
                name="recoveryCode"
                type="text"
                required
                className="appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm font-mono tracking-wider"
                placeholder="XXXX-XXXX"
                value={recoveryCode}
                onChange={(e) => setRecoveryCode(e.target.value.toUpperCase())}
                disabled={loading}
                maxLength={9}
              />
            </div>

            <div>
              <label htmlFor="new-password" className="sr-only">
                Нова парола
              </label>
              <input
                id="new-password"
                name="newPassword"
                type="password"
                required
                minLength={6}
                className="appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                placeholder="Нова парола (минимум 6 символа)"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                disabled={loading}
              />
            </div>

            <div>
              <label htmlFor="confirm-password" className="sr-only">
                Потвърди парола
              </label>
              <input
                id="confirm-password"
                name="confirmPassword"
                type="password"
                required
                minLength={6}
                className="appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                placeholder="Потвърди новата парола"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                disabled={loading}
              />
            </div>
          </div>

          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => {
                setShowRecovery(false);
                setError('');
                setRecoveryCode('');
                setNewPassword('');
                setConfirmPassword('');
              }}
              disabled={loading}
              className="flex-1 py-2 px-4 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
            >
              Назад
            </button>
            <button
              type="submit"
              disabled={loading}
              className="flex-1 py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-gray-400 disabled:cursor-not-allowed"
            >
              {loading ? 'Възстановяване...' : 'Възстанови паролата'}
            </button>
          </div>
        </form>
        )}
      </div>
    </div>
  );
}

export default Login;
