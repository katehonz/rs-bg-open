import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

export default function AssetAdjustmentsHistory() {
  const [adjustments, setAdjustments] = useState([]);
  const [loading, setLoading] = useState(true);
  const [filterType, setFilterType] = useState('all');
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');

  const ADJUSTMENTS_QUERY = `
    query GetAssetAdjustments($companyId: Int!, $fromDate: NaiveDate, $toDate: NaiveDate, $adjustmentType: String) {
      companyAssetAdjustments(
        companyId: $companyId,
        fromDate: $fromDate,
        toDate: $toDate,
        adjustmentType: $adjustmentType
      ) {
        id
        fixedAssetId
        adjustmentDate
        adjustmentType
        adjustmentAmount
        accountingValueBefore
        accountingValueAfter
        taxValueBefore
        taxValueAfter
        reason
        documentNumber
        isPosted
        newAccountingDepreciationRate
        newTaxDepreciationRate
        createdAt
        asset {
          id
          inventoryNumber
          name
          category {
            name
          }
        }
      }
    }
  `;

  useEffect(() => {
    loadAdjustments();
  }, [filterType, fromDate, toDate]);

  const loadAdjustments = async () => {
    try {
      setLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const variables = {
        companyId,
        fromDate: fromDate || null,
        toDate: toDate || null,
        adjustmentType: filterType === 'all' ? null : filterType
      };

      const response = await graphqlRequest(ADJUSTMENTS_QUERY, variables);
      setAdjustments(response.companyAssetAdjustments || []);
    } catch (err) {
      console.error('Error loading adjustments:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      style: 'currency',
      currency: 'BGN'
    }).format(amount || 0);
  };

  const formatDate = (date) => {
    if (!date) return '-';
    return new Date(date).toLocaleDateString('bg-BG');
  };

  const getTypeBadge = (type) => {
    const config = {
      improvement: {
        bg: 'bg-green-100',
        text: 'text-green-800',
        label: 'Увеличение',
        icon: '📈'
      },
      impairment: {
        bg: 'bg-orange-100',
        text: 'text-orange-800',
        label: 'Намаление',
        icon: '📉'
      },
      acquisition: {
        bg: 'bg-blue-100',
        text: 'text-blue-800',
        label: 'Придобиване',
        icon: '🆕'
      },
      revaluation: {
        bg: 'bg-purple-100',
        text: 'text-purple-800',
        label: 'Преоценка',
        icon: '⚖️'
      },
      disposal: {
        bg: 'bg-red-100',
        text: 'text-red-800',
        label: 'Отписване',
        icon: '🗑️'
      },
      internal_transfer: {
        bg: 'bg-yellow-100',
        text: 'text-yellow-800',
        label: 'Вътрешен трансфер',
        icon: '🔄'
      },
      scrap: {
        bg: 'bg-gray-100',
        text: 'text-gray-800',
        label: 'Брак',
        icon: '❌'
      }
    };

    const typeConfig = config[type] || config.acquisition;

    return (
      <span className={`inline-flex items-center px-2 py-1 text-xs font-medium rounded-full ${typeConfig.bg} ${typeConfig.text}`}>
        <span className="mr-1">{typeConfig.icon}</span>
        {typeConfig.label}
      </span>
    );
  };

  const getStatusBadge = (isPosted) => {
    return isPosted ? (
      <span className="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800">
        ✅ Приключен
      </span>
    ) : (
      <span className="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800">
        ⏳ Неприключен
      </span>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
      </div>
    );
  }

  return (
    <div>
      {/* Filters */}
      <div className="mb-6 flex gap-4 flex-wrap">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Тип корекция</label>
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className="block w-48 px-3 py-2 border border-gray-300 rounded-md"
          >
            <option value="all">Всички</option>
            <option value="improvement">Увеличение</option>
            <option value="impairment">Намаление</option>
            <option value="revaluation">Преоценка</option>
            <option value="disposal">Отписване</option>
            <option value="internal_transfer">Вътрешен трансфер</option>
            <option value="scrap">Брак</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">От дата</label>
          <input
            type="date"
            value={fromDate}
            onChange={(e) => setFromDate(e.target.value)}
            className="block w-48 px-3 py-2 border border-gray-300 rounded-md"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">До дата</label>
          <input
            type="date"
            value={toDate}
            onChange={(e) => setToDate(e.target.value)}
            className="block w-48 px-3 py-2 border border-gray-300 rounded-md"
          />
        </div>

        <div className="flex items-end">
          <button
            onClick={loadAdjustments}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
          >
            Обнови
          </button>
        </div>
      </div>

      {/* Summary Stats */}
      {adjustments.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
          <div className="bg-green-50 border border-green-200 rounded-lg p-4">
            <p className="text-sm text-gray-600">Общо увеличения</p>
            <p className="text-2xl font-bold text-green-700">
              {formatCurrency(
                adjustments
                  .filter(a => a.adjustmentType === 'improvement')
                  .reduce((sum, a) => sum + parseFloat(a.adjustmentAmount), 0)
              )}
            </p>
          </div>

          <div className="bg-orange-50 border border-orange-200 rounded-lg p-4">
            <p className="text-sm text-gray-600">Общо намаления</p>
            <p className="text-2xl font-bold text-orange-700">
              {formatCurrency(
                Math.abs(adjustments
                  .filter(a => a.adjustmentType === 'impairment')
                  .reduce((sum, a) => sum + parseFloat(a.adjustmentAmount), 0))
              )}
            </p>
          </div>

          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <p className="text-sm text-gray-600">Брой корекции</p>
            <p className="text-2xl font-bold text-blue-700">{adjustments.length}</p>
          </div>
        </div>
      )}

      {/* Adjustments Table */}
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Дата
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Актив
              </th>
              <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase">
                Тип
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                Сума
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                Счет. стойност преди/след
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                Дан. стойност преди/след
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Основание
              </th>
              <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase">
                Статус
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {adjustments.map((adjustment) => (
              <tr key={adjustment.id} className="hover:bg-gray-50">
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {formatDate(adjustment.adjustmentDate)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm font-medium text-gray-900">
                    {adjustment.asset?.inventoryNumber}
                  </div>
                  <div className="text-xs text-gray-500">{adjustment.asset?.name}</div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-center">
                  {getTypeBadge(adjustment.adjustmentType)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right">
                  <span className={`font-semibold ${
                    parseFloat(adjustment.adjustmentAmount) > 0
                      ? 'text-green-600'
                      : 'text-orange-600'
                  }`}>
                    {formatCurrency(adjustment.adjustmentAmount)}
                  </span>
                  {adjustment.documentNumber && (
                    <div className="text-xs text-gray-500">
                      №{adjustment.documentNumber}
                    </div>
                  )}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right text-sm">
                  <div className="text-gray-600">
                    {formatCurrency(adjustment.accountingValueBefore)}
                  </div>
                  <div className="text-gray-400">→</div>
                  <div className="font-semibold text-gray-900">
                    {formatCurrency(adjustment.accountingValueAfter)}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right text-sm">
                  <div className="text-gray-600">
                    {formatCurrency(adjustment.taxValueBefore)}
                  </div>
                  <div className="text-gray-400">→</div>
                  <div className="font-semibold text-gray-900">
                    {formatCurrency(adjustment.taxValueAfter)}
                  </div>
                </td>
                <td className="px-6 py-4 text-sm text-gray-900 max-w-xs">
                  <div className="truncate" title={adjustment.reason}>
                    {adjustment.reason}
                  </div>
                  {(adjustment.newAccountingDepreciationRate || adjustment.newTaxDepreciationRate) && (
                    <div className="text-xs text-blue-600 mt-1">
                      🔄 Променени норми
                    </div>
                  )}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-center">
                  {getStatusBadge(adjustment.isPosted)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {adjustments.length === 0 && (
          <div className="text-center py-12">
            <div className="text-4xl mb-4">📝</div>
            <p className="text-gray-500">Няма намерени корекции</p>
          </div>
        )}
      </div>
    </div>
  );
}
