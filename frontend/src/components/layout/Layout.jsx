import { useState, useEffect } from 'react';
import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import Header from './Header';
import Breadcrumbs from './Breadcrumbs';
import { graphqlRequest } from '../../utils/graphqlClient';

export default function Layout() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [currentCompany, setCurrentCompany] = useState('Зареждане...');

  const COMPANY_QUERY = `
    query GetCompany($id: Int!) {
      company(id: $id) {
        name
        eik
      }
    }
  `;

  useEffect(() => {
    const loadCurrentCompany = async () => {
      try {
        const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
        const response = await graphqlRequest(COMPANY_QUERY, { id: companyId });
        if (response.company) {
          setCurrentCompany(response.company.name);
        }
      } catch (err) {
        console.error('Грешка при зареждане на компания:', err);
        setCurrentCompany('Неопределена компания');
      }
    };
    loadCurrentCompany();
  }, []);

  const toggleSidebar = () => {
    setSidebarCollapsed(!sidebarCollapsed);
  };

  return (
    <div className="min-h-screen bg-gray-50 flex">
      <Sidebar 
        collapsed={sidebarCollapsed} 
        setCollapsed={setSidebarCollapsed}
      />
      
      <div className="flex-1 flex flex-col">
        <Header 
          currentCompany={currentCompany}
          toggleSidebar={toggleSidebar}
        />
        
        <main className="flex-1 overflow-auto">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
            <Breadcrumbs />
            <div className="mt-6">
              <Outlet />
            </div>
          </div>
        </main>
        
        <footer className="bg-white border-t border-gray-200 px-6 py-3">
          <div className="flex justify-between items-center text-sm text-gray-500">
            <div>
              © 2025 RS-AC-BG | Българска счетоводна система
            </div>
            <div className="flex items-center space-x-4">
              <span className="flex items-center">
                <div className="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                БНБ връзка активна
              </span>
              <span>v1.0.0</span>
            </div>
          </div>
        </footer>
      </div>
    </div>
  );
}