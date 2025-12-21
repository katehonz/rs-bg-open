import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../../context/AuthContext';

export default function Header({ currentCompany, toggleSidebar }) {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  const getUserInitials = () => {
    if (!user) return 'ИП';
    const firstName = user.first_name || '';
    const lastName = user.last_name || '';
    return `${firstName.charAt(0)}${lastName.charAt(0)}`.toUpperCase() || 'ИП';
  };

  const getUserName = () => {
    if (!user) return 'Иван Петров';
    return `${user.first_name || ''} ${user.last_name || ''}`.trim() || user.username;
  };

  return (
    <header className="bg-white border-b border-gray-200 px-6 py-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          <button
            onClick={toggleSidebar}
            className="p-2 rounded-md text-gray-400 hover:text-gray-600 hover:bg-gray-50 lg:hidden transition-colors"
          >
            <div className="w-6 h-6">☰</div>
          </button>

          <div className="ml-4">
            <h2 className="text-lg font-semibold text-gray-900">
              {currentCompany || "Демо ЕООД"}
            </h2>
            <p className="text-sm text-gray-500">
              Българска счетоводна система
            </p>
          </div>

          {/* Company Settings Link */}
          <Link
            to="/settings/companies"
            className="ml-4 px-3 py-1 text-xs bg-blue-50 text-blue-600 hover:bg-blue-100 rounded-md transition-colors"
            title="Управление на фирми"
          >
            Смени фирма
          </Link>
        </div>

        <div className="flex items-center space-x-3">
          {/* Notifications */}
          <button className="relative p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-md transition-colors">
            <div className="w-6 h-6">🔔</div>
            <span className="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center font-medium shadow-sm">
              3
            </span>
          </button>

          {/* User Menu */}
          <div className="relative group">
            <button className="flex items-center space-x-3 p-2 rounded-lg hover:bg-gray-50 transition-colors">
              <div className="w-9 h-9 bg-gradient-to-r from-blue-600 to-blue-700 text-white rounded-full flex items-center justify-center text-sm font-semibold shadow-sm">
                {getUserInitials()}
              </div>
              <div className="hidden sm:block text-left">
                <div className="text-sm font-medium text-gray-900">{getUserName()}</div>
                <div className="text-xs text-gray-500">Счетоводител</div>
              </div>
              <div className="w-4 h-4 text-gray-400">▼</div>
            </button>

            {/* Dropdown Menu */}
            <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 hidden group-hover:block z-50">
              <button
                onClick={handleLogout}
                className="block w-full text-left px-4 py-2 text-sm text-red-700 hover:bg-red-50 transition-colors"
              >
                Изход
              </button>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}