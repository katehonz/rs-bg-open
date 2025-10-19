import React from 'react';
import { Routes, Route, Navigate, useLocation } from 'react-router-dom';
import { IntrastatDashboard, IntrastatReports, IntrastatSettings } from '../components/Intrastat/index.jsx';

const IntrastatTabs = [
  { id: 'dashboard', name: 'Табло', icon: '📊' },
  { id: 'reports', name: 'Справки', icon: '📈' },
  { id: 'settings', name: 'Настройки', icon: '⚙️' }
];

const Intrastat = () => {
  const location = useLocation();
  const currentTab = location.pathname.split('/').pop();
  
  // Get company ID from localStorage or default to 1
  const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

  const getTabClassName = (tabId) => {
    const isActive = currentTab === tabId || (currentTab === 'intrastat' && tabId === 'dashboard');
    return `inline-flex items-center px-4 py-2 border-b-2 font-medium text-sm transition-colors ${
      isActive
        ? 'border-orange-500 text-orange-600 bg-orange-50'
        : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
    }`;
  };

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h1 className="text-2xl font-bold text-gray-900 flex items-center gap-2">
              🌍 INTRASTAT Модул
            </h1>
            <p className="text-gray-600 mt-1">
              Управление на декларации за вътреобщностна търговия
            </p>
          </div>
          <div className="bg-orange-100 text-orange-800 px-3 py-1 rounded-full text-sm font-medium">
            Нов модул
          </div>
        </div>
        
        {/* Tabs Navigation */}
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8">
            {IntrastatTabs.map((tab) => (
              <a
                key={tab.id}
                href={`/intrastat/${tab.id}`}
                className={getTabClassName(tab.id)}
                onClick={(e) => {
                  e.preventDefault();
                  window.history.pushState({}, '', `/intrastat/${tab.id}`);
                  window.dispatchEvent(new PopStateEvent('popstate'));
                }}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.name}
              </a>
            ))}
          </nav>
        </div>
      </div>

      {/* Tab Content */}
      <Routes>
        <Route index element={<Navigate to="dashboard" replace />} />
        <Route path="dashboard" element={<IntrastatDashboard companyId={companyId} />} />
        <Route path="reports" element={<IntrastatReports companyId={companyId} />} />
        <Route path="settings" element={<IntrastatSettings companyId={companyId} />} />
      </Routes>
    </div>
  );
};

export default Intrastat;
