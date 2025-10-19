import { Link, useLocation } from 'react-router-dom';

const routeNames = {
  '/': 'Начало',
  '/dashboard': 'Табло',
  '/test': 'Тест',
  '/accounting': 'Счетоводство',
  '/accounting/entries': 'Дневници',
  '/vat': 'ДДС',
  '/vat/returns': 'Декларации',
  '/vat/rates': 'Ставки',
  '/reports': 'Отчети',
  '/banks': 'Банкови профили',
  '/currencies': 'Валути',
  '/counterparts': 'Контрагенти',
  '/settings': 'Настройки',
  '/settings/users': 'Потребители',
  '/settings/companies': 'Фирми',
  '/settings/system': 'Системни настройки',
};

export default function Breadcrumbs() {
  const location = useLocation();
  const pathnames = location.pathname.split('/').filter(x => x);
  
  // Build breadcrumb items
  const breadcrumbs = [];
  let currentPath = '';
  
  // Always start with dashboard
  breadcrumbs.push({
    name: 'Табло',
    path: '/dashboard',
    current: location.pathname === '/' || location.pathname === '/dashboard'
  });
  
  // Add subsequent path parts
  pathnames.forEach((name, index) => {
    currentPath += '/' + name;
    const isLast = index === pathnames.length - 1;
    
    if (currentPath !== '/dashboard') {
      breadcrumbs.push({
        name: routeNames[currentPath] || name.charAt(0).toUpperCase() + name.slice(1),
        path: currentPath,
        current: isLast
      });
    }
  });

  return (
    <nav className="flex items-center space-x-2 text-sm text-gray-500">
      {breadcrumbs.map((breadcrumb, index) => (
        <div key={breadcrumb.path} className="flex items-center">
          {index > 0 && <span className="mx-2 text-gray-300">/</span>}
          {breadcrumb.current ? (
            <span className="text-gray-900 font-medium">{breadcrumb.name}</span>
          ) : (
            <Link 
              to={breadcrumb.path} 
              className="flex items-center hover:text-gray-700"
            >
              {index === 0 && <span className="mr-1">🏠</span>}
              {breadcrumb.name}
            </Link>
          )}
        </div>
      ))}
    </nav>
  );
}
