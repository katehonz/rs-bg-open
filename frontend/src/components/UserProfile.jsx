import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

export default function UserProfile() {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState('profile'); // 'profile', 'password', 'recovery'

  // Password change state
  const [passwordForm, setPasswordForm] = useState({
    currentPassword: '',
    newPassword: '',
    confirmPassword: ''
  });
  const [passwordLoading, setPasswordLoading] = useState(false);
  const [passwordError, setPasswordError] = useState('');
  const [passwordSuccess, setPasswordSuccess] = useState(false);

  // Recovery code state
  const [recoveryCode, setRecoveryCode] = useState(null);
  const [showRecoveryCode, setShowRecoveryCode] = useState(false);
  const [generatingCode, setGeneratingCode] = useState(false);

  const USER_QUERY = `
    query GetCurrentUser {
      me {
        id
        username
        email
        firstName
        lastName
        groupId
        isActive
        createdAt
      }
    }
  `;

  const CHANGE_OWN_PASSWORD_MUTATION = `
    mutation ChangeOwnPassword($currentPassword: String!, $newPassword: String!) {
      changeOwnPassword(currentPassword: $currentPassword, newPassword: $newPassword)
    }
  `;

  useEffect(() => {
    loadUser();
  }, []);

  const loadUser = async () => {
    try {
      setLoading(true);
      const authUser = localStorage.getItem('authUser');
      if (authUser) {
        setUser(JSON.parse(authUser));
      }
    } catch (err) {
      console.error('Failed to load user:', err);
    } finally {
      setLoading(false);
    }
  };

  const handlePasswordChange = async (e) => {
    e.preventDefault();
    setPasswordError('');
    setPasswordSuccess(false);

    // Validation
    if (passwordForm.newPassword.length < 6) {
      setPasswordError('Новата парола трябва да е поне 6 символа');
      return;
    }

    if (passwordForm.newPassword !== passwordForm.confirmPassword) {
      setPasswordError('Новата парола и потвърждението не съвпадат');
      return;
    }

    try {
      setPasswordLoading(true);
      await graphqlRequest(CHANGE_OWN_PASSWORD_MUTATION, {
        currentPassword: passwordForm.currentPassword,
        newPassword: passwordForm.newPassword
      });

      setPasswordSuccess(true);
      setPasswordForm({
        currentPassword: '',
        newPassword: '',
        confirmPassword: ''
      });
    } catch (err) {
      setPasswordError(err.message || 'Грешка при смяна на паролата');
    } finally {
      setPasswordLoading(false);
    }
  };

  const generateRecoveryCode = () => {
    // Generate a random 8-character recovery code
    const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'; // Excluded similar looking chars
    let code = '';
    for (let i = 0; i < 8; i++) {
      code += chars.charAt(Math.floor(Math.random() * chars.length));
      if (i === 3) code += '-'; // Add separator after 4 chars
    }
    return code;
  };

  const handleGenerateRecoveryCode = async () => {
    try {
      setGeneratingCode(true);
      const code = generateRecoveryCode();

      // Save the hashed code to the backend
      const GENERATE_RECOVERY_CODE_MUTATION = `
        mutation GenerateRecoveryCode($recoveryCode: String!) {
          generateRecoveryCode(recoveryCode: $recoveryCode)
        }
      `;

      await graphqlRequest(GENERATE_RECOVERY_CODE_MUTATION, {
        recoveryCode: code
      });

      // Show the code to the user (only once)
      setRecoveryCode(code);
      setShowRecoveryCode(true);

    } catch (err) {
      alert('Грешка при генериране на код: ' + err.message);
    } finally {
      setGeneratingCode(false);
    }
  };

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
    alert('Кодът е копиран в клипборда!');
  };

  const downloadRecoveryCode = (code) => {
    const content = `RS-AC-BG Код за възстановяване на парола\n\n` +
                   `Потребител: ${user?.username}\n` +
                   `Име: ${user?.firstName} ${user?.lastName}\n` +
                   `Email: ${user?.email}\n\n` +
                   `КОД: ${code}\n\n` +
                   `Генериран на: ${new Date().toLocaleString('bg-BG')}\n\n` +
                   `ВАЖНО:\n` +
                   `- Запазете този код на сигурно място\n` +
                   `- Никога не го споделяйте с никого\n` +
                   `- Използвайте го за възстановяване на достъп при нужда\n`;

    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `recovery-code-${user?.username}-${Date.now()}.txt`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
        <span className="ml-2 text-gray-600">Зареждане...</span>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto">
      <div className="bg-white shadow rounded-lg overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-600 to-blue-700 px-6 py-8">
          <div className="flex items-center space-x-4">
            <div className="h-20 w-20 rounded-full bg-white/20 flex items-center justify-center text-white text-2xl font-bold">
              {user?.firstName?.[0]}{user?.lastName?.[0]}
            </div>
            <div className="text-white">
              <h1 className="text-2xl font-bold">{user?.firstName} {user?.lastName}</h1>
              <p className="text-blue-100">@{user?.username}</p>
              <p className="text-blue-100 text-sm">{user?.email}</p>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div className="border-b border-gray-200">
          <nav className="flex -mb-px">
            <button
              onClick={() => setActiveTab('profile')}
              className={`px-6 py-4 text-sm font-medium border-b-2 transition-colors ${
                activeTab === 'profile'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              👤 Профил
            </button>
            <button
              onClick={() => setActiveTab('password')}
              className={`px-6 py-4 text-sm font-medium border-b-2 transition-colors ${
                activeTab === 'password'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              🔒 Смяна на парола
            </button>
            <button
              onClick={() => setActiveTab('recovery')}
              className={`px-6 py-4 text-sm font-medium border-b-2 transition-colors ${
                activeTab === 'recovery'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              🔑 Код за възстановяване
            </button>
          </nav>
        </div>

        {/* Content */}
        <div className="p-6">
          {/* Profile Tab */}
          {activeTab === 'profile' && (
            <div className="space-y-6">
              <h2 className="text-xl font-semibold text-gray-900">Информация за профила</h2>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Име</label>
                  <div className="px-4 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-900">
                    {user?.firstName}
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Фамилия</label>
                  <div className="px-4 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-900">
                    {user?.lastName}
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Потребителско име</label>
                  <div className="px-4 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-900">
                    {user?.username}
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Email</label>
                  <div className="px-4 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-900">
                    {user?.email}
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Статус</label>
                  <div className="px-4 py-2 bg-gray-50 border border-gray-200 rounded-md">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      user?.isActive
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}>
                      {user?.isActive ? '✓ Активен' : '✗ Неактивен'}
                    </span>
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Създаден на</label>
                  <div className="px-4 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-900">
                    {user?.createdAt ? new Date(user.createdAt).toLocaleDateString('bg-BG') : 'N/A'}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Password Tab */}
          {activeTab === 'password' && (
            <div className="space-y-6">
              <h2 className="text-xl font-semibold text-gray-900">Смяна на парола</h2>

              {passwordError && (
                <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
                  {passwordError}
                </div>
              )}

              {passwordSuccess && (
                <div className="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded">
                  ✓ Паролата е сменена успешно!
                </div>
              )}

              <form onSubmit={handlePasswordChange} className="space-y-4">
                <div>
                  <label htmlFor="currentPassword" className="block text-sm font-medium text-gray-700 mb-1">
                    Текуща парола
                  </label>
                  <input
                    type="password"
                    id="currentPassword"
                    value={passwordForm.currentPassword}
                    onChange={(e) => setPasswordForm({...passwordForm, currentPassword: e.target.value})}
                    required
                    className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>

                <div>
                  <label htmlFor="newPassword" className="block text-sm font-medium text-gray-700 mb-1">
                    Нова парола (минимум 6 символа)
                  </label>
                  <input
                    type="password"
                    id="newPassword"
                    value={passwordForm.newPassword}
                    onChange={(e) => setPasswordForm({...passwordForm, newPassword: e.target.value})}
                    required
                    minLength={6}
                    className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>

                <div>
                  <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-700 mb-1">
                    Потвърди нова парола
                  </label>
                  <input
                    type="password"
                    id="confirmPassword"
                    value={passwordForm.confirmPassword}
                    onChange={(e) => setPasswordForm({...passwordForm, confirmPassword: e.target.value})}
                    required
                    minLength={6}
                    className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>

                <div className="pt-2">
                  <button
                    type="submit"
                    disabled={passwordLoading}
                    className="w-full bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {passwordLoading ? 'Смяна...' : 'Смени парола'}
                  </button>
                </div>
              </form>
            </div>
          )}

          {/* Recovery Code Tab */}
          {activeTab === 'recovery' && (
            <div className="space-y-6">
              <h2 className="text-xl font-semibold text-gray-900">Код за възстановяване на парола</h2>

              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <div className="flex">
                  <div className="flex-shrink-0">
                    <svg className="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                    </svg>
                  </div>
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-blue-800">Какво е код за възстановяване?</h3>
                    <div className="mt-2 text-sm text-blue-700">
                      <p>Кодът за възстановяване ви позволява да възстановите достъп до профила си без нужда от email верификация. Запазете го на сигурно място!</p>
                    </div>
                  </div>
                </div>
              </div>

              {!showRecoveryCode ? (
                <div className="text-center py-8">
                  <div className="text-6xl mb-4">🔑</div>
                  <p className="text-gray-600 mb-6">
                    Генерирайте код за възстановяване, който можете да използвате за достъп до профила си.
                  </p>
                  <button
                    onClick={handleGenerateRecoveryCode}
                    disabled={generatingCode}
                    className="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 transition-colors"
                  >
                    {generatingCode ? 'Генериране...' : '🔑 Генерирай код'}
                  </button>
                </div>
              ) : (
                <div className="space-y-4">
                  <div className="bg-yellow-50 border-2 border-yellow-400 rounded-lg p-6">
                    <div className="flex items-start">
                      <div className="flex-shrink-0">
                        <svg className="h-6 w-6 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                        </svg>
                      </div>
                      <div className="ml-3 flex-1">
                        <h3 className="text-lg font-medium text-yellow-800 mb-2">
                          Вашият код за възстановяване
                        </h3>
                        <div className="bg-white border-2 border-yellow-300 rounded-md p-4 mb-3">
                          <div className="flex items-center justify-between">
                            <code className="text-2xl font-mono font-bold text-gray-900 tracking-wider">
                              {recoveryCode}
                            </code>
                            <div className="flex gap-2">
                              <button
                                onClick={() => copyToClipboard(recoveryCode)}
                                className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors"
                              >
                                📋 Копирай
                              </button>
                              <button
                                onClick={() => downloadRecoveryCode(recoveryCode)}
                                className="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 transition-colors"
                              >
                                💾 Изтегли
                              </button>
                            </div>
                          </div>
                        </div>
                        <div className="text-sm text-yellow-700">
                          <p className="font-semibold mb-1">⚠️ ВАЖНО:</p>
                          <ul className="list-disc list-inside space-y-1">
                            <li>Запишете този код на сигурно място</li>
                            <li>Никога не го споделяйте с никого</li>
                            <li>Този код ще се покаже само веднъж</li>
                            <li>Използвайте го за възстановяване на достъп при нужда</li>
                          </ul>
                        </div>
                      </div>
                    </div>
                  </div>

                  <button
                    onClick={handleGenerateRecoveryCode}
                    className="w-full px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 transition-colors"
                  >
                    🔄 Генерирай нов код
                  </button>
                </div>
              )}

              <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                <h4 className="text-sm font-medium text-gray-900 mb-2">Как да използвам кода?</h4>
                <ol className="list-decimal list-inside space-y-1 text-sm text-gray-700">
                  <li>На страницата за вход изберете "Забравена парола"</li>
                  <li>Въведете вашия код за възстановяване</li>
                  <li>Задайте нова парола</li>
                  <li>Влезте с новата парола</li>
                </ol>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
