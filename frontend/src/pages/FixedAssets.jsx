import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

// Tab Components
import AssetsList from '../components/fixedAssets/AssetsList';
import AssetCategories from '../components/fixedAssets/AssetCategories';
import DepreciationCalculation from '../components/fixedAssets/DepreciationCalculation';
import DepreciationJournal from '../components/fixedAssets/DepreciationJournal';
import DepreciationReport from '../components/fixedAssets/DepreciationReport';

export default function FixedAssets() {
  const [activeTab, setActiveTab] = useState('assets');
  const [stats, setStats] = useState({
    totalAssets: 0,
    totalAcquisitionCost: 0,
    totalBookValue: 0,
    activeAssets: 0
  });
  const [error, setError] = useState(null);

  const STATS_QUERY = `
    query GetFixedAssetsStats($companyId: Int!) {
      fixedAssetsSummary(companyId: $companyId) {
        totalAssets
        totalAcquisitionCost
        totalAccountingBookValue
        totalTaxBookValue
        totalAccumulatedDepreciation
        activeAssets
        disposedAssets
      }
    }
  `;

  useEffect(() => {
    loadStats();
  }, []);

  const loadStats = async () => {
    try {
      setError(null);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      const response = await graphqlRequest(STATS_QUERY, { companyId });
      
      if (response.fixedAssetsSummary) {
        setStats({
          totalAssets: response.fixedAssetsSummary.totalAssets,
          totalAcquisitionCost: response.fixedAssetsSummary.totalAcquisitionCost,
          totalBookValue: response.fixedAssetsSummary.totalAccountingBookValue,
          activeAssets: response.fixedAssetsSummary.activeAssets
        });
      }
    } catch (err) {
      console.error('Error loading stats:', err);
      setError(err.message);
    }
  };

  const tabs = [
    { id: 'assets', label: 'ДМА Регистър', icon: '🏭' },
    { id: 'categories', label: 'Категории', icon: '📊' },
    { id: 'depreciation', label: 'Амортизация', icon: '📉' },
    { id: 'journal', label: 'Дневник', icon: '📖' },
    { id: 'report', label: 'Справка', icon: '📄' }
  ];

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      style: 'currency',
      currency: 'BGN'
    }).format(amount || 0);
  };

  if (error) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <h2 className="text-lg font-semibold text-red-800">Грешка при зареждане</h2>
          <p className="text-red-600 mt-2">{error}</p>
          <button 
            onClick={() => loadStats()} 
            className="mt-4 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
          >
            Опитай отново
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header with Stats */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Дълготрайни материални активи</h1>
            <p className="mt-1 text-sm text-gray-500">
              Управление на ДМА и амортизации според ЗКПО
            </p>
          </div>
          <button 
            onClick={() => setActiveTab('assets')}
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
          >
            <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
            </svg>
            Нов актив
          </button>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-gray-50 rounded-lg p-4">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
                  <span className="text-xl">🏭</span>
                </div>
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-500">Общо активи</p>
                <p className="text-2xl font-semibold text-gray-900">{stats.totalAssets}</p>
              </div>
            </div>
          </div>

          <div className="bg-gray-50 rounded-lg p-4">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
                  <span className="text-xl">💰</span>
                </div>
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-500">Отчетна стойност</p>
                <p className="text-xl font-semibold text-gray-900">
                  {formatCurrency(stats.totalAcquisitionCost)}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-gray-50 rounded-lg p-4">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-12 h-12 bg-yellow-100 rounded-lg flex items-center justify-center">
                  <span className="text-xl">📊</span>
                </div>
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-500">Балансова стойност</p>
                <p className="text-xl font-semibold text-gray-900">
                  {formatCurrency(stats.totalBookValue)}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-gray-50 rounded-lg p-4">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
                  <span className="text-xl">✅</span>
                </div>
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-500">Активни</p>
                <p className="text-2xl font-semibold text-gray-900">{stats.activeAssets}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white rounded-lg shadow">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8 px-6">
            {tabs.map(tab => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`
                  py-4 px-1 border-b-2 font-medium text-sm flex items-center
                  ${activeTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }
                `}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        <div className="p-6">
          {activeTab === 'assets' && <AssetsList onRefreshStats={loadStats} />}
          {activeTab === 'categories' && <AssetCategories />}
          {activeTab === 'depreciation' && <DepreciationCalculation onRefreshStats={loadStats} />}
          {activeTab === 'journal' && <DepreciationJournal />}
          {activeTab === 'report' && <DepreciationReport />}
        </div>
      </div>
    </div>
  );
}