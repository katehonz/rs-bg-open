import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';

const menuSections = [
  {
    title: "Основни",
    icon: "📊",
    items: [
      {
        title: "Табло",
        icon: "🏠",
        path: "/dashboard",
        badge: null,
        color: null,
      },
    ]
  },
  {
    title: "Счетоводство",
    icon: "📚",
    items: [
      {
        title: "Дневници",
        icon: "📝",
        path: "/accounting/entries",
        badge: null,
        color: null,
      },
      {
        title: "ДДС Операции",
        icon: "💼",
        path: "/accounting/vat-entry",
        badge: "NEW",
        color: "bg-green-100 text-green-600",
      },
      {
        title: "Сметкоплан",
        icon: "🗂️",
        path: "/accounting/chart",
        badge: null,
        color: null,
      },
      {
        title: "Журнални записи",
        icon: "📋",
        path: "/accounting/journal-entries",
        badge: null,
        color: null,
      },
      {
        title: "Дълготрайни активи",
        icon: "🏭",
        path: "/fixed-assets",
        badge: "NEW",
        color: "bg-green-100 text-green-600",
      },
      {
        title: "Импорти",
        icon: "📥",
        path: "/imports",
        badge: "NEW",
        color: "bg-purple-100 text-purple-600",
      },
      {
        title: "Банки",
        icon: "🏦",
        path: "/banks",
        badge: null,
        color: null,
      },
      {
        title: "Отчети",
        icon: "📋",
        path: "/reports",
        badge: null,
        color: null,
      },
      {
        title: "SAF-T Export",
        icon: "📤",
        path: "/saft-export",
        badge: "NEW",
        color: "bg-blue-100 text-blue-600",
      },
      {
        title: "INTRASTAT",
        icon: "🌍",
        path: "/intrastat",
        badge: "NEW",
        color: "bg-orange-100 text-orange-600",
      },
    ]
  },
  {
    title: "ДДС",
    icon: "💰",
    items: [
      {
        title: "Декларации",
        icon: "📄",
        path: "/vat/returns",
        badge: "1",
        color: "bg-red-100 text-red-600",
      },
      {
        title: "Ставки",
        icon: "🏷️",
        path: "/vat/rates",
        badge: null,
        color: null,
      },
    ]
  },
  {
    title: "Системни",
    icon: "⚙️",
    items: [
      {
        title: "Валути",
        icon: "💱",
        path: "/currencies",
        badge: null,
        color: null,
      },
      {
        title: "Контрагенти",
        icon: "👥",
        path: "/counterparts",
        badge: null,
        color: null,
      },
      {
        title: "Настройки",
        icon: "🔧",
        path: "/settings",
        badge: null,
        color: null,
      },
    ]
  }
];

export default function Sidebar({ collapsed, setCollapsed }) {
  const location = useLocation();
  const [expandedSections, setExpandedSections] = useState(new Set(['Основни']));

  const toggleSection = (sectionTitle) => {
    setExpandedSections(prev => {
      const newSet = new Set(prev);
      if (newSet.has(sectionTitle)) {
        newSet.delete(sectionTitle);
      } else {
        newSet.add(sectionTitle);
      }
      return newSet;
    });
  };

  const isActive = (path) => {
    if (path === '/dashboard' && (location.pathname === '/' || location.pathname === '/dashboard')) {
      return true;
    }
    return location.pathname.startsWith(path);
  };

  return (
    <div className={`bg-white border-r border-gray-200 flex flex-col transition-all duration-300 shadow-sm ${
      collapsed ? 'w-16' : 'w-64'
    }`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center">
          {!collapsed && (
            <div>
              <h1 className="text-lg font-bold text-gray-900">RS-AC-BG</h1>
              <p className="text-xs text-gray-500">Българска счетоводна система</p>
            </div>
          )}
          <button
            onClick={() => setCollapsed(!collapsed)}
            className={`p-1 rounded-md hover:bg-gray-100 ${collapsed ? 'mx-auto' : 'ml-auto'}`}
          >
            <div className="w-4 h-4 text-gray-600">
              {collapsed ? '→' : '←'}
            </div>
          </button>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto py-4">
        {menuSections.map((section) => (
          <div key={section.title} className="mb-6">
            {/* Section Header */}
            <button
              onClick={() => toggleSection(section.title)}
              className={`w-full flex items-center px-4 py-2 text-left hover:bg-gray-50 ${
                collapsed ? 'justify-center' : 'justify-between'
              }`}
            >
              <div className="flex items-center">
                <span className="text-lg">{section.icon}</span>
                {!collapsed && (
                  <span className="ml-3 text-sm font-medium text-gray-700">
                    {section.title}
                  </span>
                )}
              </div>
              {!collapsed && (
                <div className="text-gray-400">
                  {expandedSections.has(section.title) ? '▼' : '▶'}
                </div>
              )}
            </button>

            {/* Menu Items */}
            {(!collapsed && expandedSections.has(section.title)) && (
              <div className="ml-4 mt-2 space-y-1">
                {section.items.map((item) => (
                  <Link
                    key={item.path}
                    to={item.path}
                    className={`flex items-center px-4 py-2 rounded-md text-sm transition-colors ${
                      isActive(item.path)
                        ? 'bg-blue-50 text-blue-700 border-r-2 border-blue-700'
                        : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                    }`}
                  >
                    <span className="text-lg">{item.icon}</span>
                    <span className="ml-3">{item.title}</span>
                    {item.badge && (
                      <span className={`ml-auto px-2 py-0.5 text-xs rounded-full ${
                        item.color || 'bg-gray-100 text-gray-600'
                      }`}>
                        {item.badge}
                      </span>
                    )}
                  </Link>
                ))}
              </div>
            )}
          </div>
        ))}
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-gray-200">
        {!collapsed && (
          <div className="text-xs text-gray-500 text-center">
            <div>БНБ връзка активна</div>
            <div className="mt-1">v1.0.0</div>
          </div>
        )}
      </div>
    </div>
  );
}
