import { Link, useLocation } from 'react-router-dom';

export default function Production({ children }) {
  const location = useLocation();

  const isActive = (path) => {
    if (path === '/production' && (location.pathname === '/production' || location.pathname === '/production/technology-cards')) {
      return true;
    }
    return location.pathname.startsWith(path);
  };

  return (
    <div className="container mx-auto px-4 py-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Производство</h1>
        <p className="text-sm text-gray-600">
          Производствено счетоводство с технологични карти и автоматични операции
        </p>
      </div>

      {/* Tabs Navigation */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="-mb-px flex space-x-8">
          <Link
            to="/production/technology-cards"
            className={`whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm ${
              isActive('/production') || isActive('/production/technology-cards')
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Технологични карти
          </Link>
          <Link
            to="/production/batches"
            className={`whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm ${
              isActive('/production/batches')
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Производствени партиди
          </Link>
          <Link
            to="/production/reports"
            className={`whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm ${
              isActive('/production/reports')
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Справки
          </Link>
        </nav>
      </div>

      {children}
    </div>
  );
}
