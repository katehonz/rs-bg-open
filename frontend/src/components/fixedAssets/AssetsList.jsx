import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';
import AssetModal from './AssetModal';

export default function AssetsList({ onRefreshStats }) {
  const [assets, setAssets] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [editingAsset, setEditingAsset] = useState(null);
  const [filterStatus, setFilterStatus] = useState('all');
  const [filterCategory, setFilterCategory] = useState('all');
  const [categories, setCategories] = useState([]);

  const ASSETS_QUERY = `
    query GetFixedAssets($companyId: Int!, $status: String, $categoryId: Int) {
      fixedAssets(companyId: $companyId, status: $status, categoryId: $categoryId) {
        id
        inventoryNumber
        name
        description
        categoryId
        acquisitionCost
        acquisitionDate
        putIntoServiceDate
        accountingUsefulLife
        accountingDepreciationRate
        accountingBookValue
        accountingAccumulatedDepreciation
        taxDepreciationRate
        taxBookValue
        taxAccumulatedDepreciation
        status
        location
        responsiblePerson
        category {
          id
          code
          name
          taxCategory
        }
      }
    }
  `;

  const CATEGORIES_QUERY = `
    query GetCategories {
      fixedAssetCategories {
        id
        code
        name
        taxCategory
        maxTaxDepreciationRate
      }
    }
  `;

  const DELETE_MUTATION = `
    mutation DeleteAsset($id: Int!) {
      deleteFixedAsset(id: $id)
    }
  `;

  useEffect(() => {
    loadCategories();
    loadAssets();
  }, [filterStatus, filterCategory]);

  const loadCategories = async () => {
    try {
      const response = await graphqlRequest(CATEGORIES_QUERY);
      setCategories(response.fixedAssetCategories || []);
    } catch (err) {
      console.error('Error loading categories:', err);
    }
  };

  const loadAssets = async () => {
    try {
      setLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      
      const variables = { 
        companyId,
        status: filterStatus === 'all' ? null : filterStatus,
        categoryId: filterCategory === 'all' ? null : parseInt(filterCategory)
      };
      
      const response = await graphqlRequest(ASSETS_QUERY, variables);
      setAssets(response.fixedAssets || []);
    } catch (err) {
      console.error('Error loading assets:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleEdit = (asset) => {
    setEditingAsset(asset);
    setShowModal(true);
  };

  const handleDelete = async (id) => {
    if (!confirm('Сигурни ли сте, че искате да изтриете този актив?')) return;
    
    try {
      await graphqlRequest(DELETE_MUTATION, { id });
      loadAssets();
      onRefreshStats();
    } catch (err) {
      alert('Грешка при изтриване: ' + err.message);
    }
  };

  const handleModalClose = () => {
    setShowModal(false);
    setEditingAsset(null);
    loadAssets();
    onRefreshStats();
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

  const getStatusBadge = (status) => {
    const badges = {
      active: 'bg-green-100 text-green-800',
      disposed: 'bg-red-100 text-red-800',
      sold: 'bg-yellow-100 text-yellow-800'
    };
    const labels = {
      active: 'Активен',
      disposed: 'Ликвидиран',
      sold: 'Продаден'
    };
    
    return (
      <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${badges[status]}`}>
        {labels[status]}
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
    <>
      {/* Filters */}
      <div className="mb-6 flex gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Статус</label>
          <select
            value={filterStatus}
            onChange={(e) => setFilterStatus(e.target.value)}
            className="block w-48 px-3 py-2 border border-gray-300 rounded-md"
          >
            <option value="all">Всички</option>
            <option value="active">Активни</option>
            <option value="disposed">Ликвидирани</option>
            <option value="sold">Продадени</option>
          </select>
        </div>
        
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Категория</label>
          <select
            value={filterCategory}
            onChange={(e) => setFilterCategory(e.target.value)}
            className="block w-48 px-3 py-2 border border-gray-300 rounded-md"
          >
            <option value="all">Всички</option>
            {categories.map(cat => (
              <option key={cat.id} value={cat.id}>
                {cat.name}
              </option>
            ))}
          </select>
        </div>

        <div className="flex-1"></div>
        
        <div className="flex items-end">
          <button
            onClick={() => setShowModal(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
          >
            Добави актив
          </button>
        </div>
      </div>

      {/* Assets Table */}
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Инв. №
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Наименование
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Категория
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                Отчетна ст-ст
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                Счет. балансова
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                Данъчна балансова
              </th>
              <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                Дата придобиване
              </th>
              <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                Статус
              </th>
              <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                Действия
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {assets.map((asset) => (
              <tr key={asset.id} className="hover:bg-gray-50">
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                  {asset.inventoryNumber}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div>
                    <div className="text-sm font-medium text-gray-900">{asset.name}</div>
                    {asset.location && (
                      <div className="text-xs text-gray-500">📍 {asset.location}</div>
                    )}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm text-gray-900">{asset.category?.name}</div>
                  <div className="text-xs text-gray-500">Кат. {asset.category?.taxCategory} ЗКПО</div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right text-sm text-gray-900">
                  {formatCurrency(asset.acquisitionCost)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right">
                  <div className="text-sm text-gray-900">
                    {formatCurrency(asset.accountingBookValue)}
                  </div>
                  <div className="text-xs text-gray-500">
                    -{formatCurrency(asset.accountingAccumulatedDepreciation)}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right">
                  <div className="text-sm text-gray-900">
                    {formatCurrency(asset.taxBookValue)}
                  </div>
                  <div className="text-xs text-gray-500">
                    -{formatCurrency(asset.taxAccumulatedDepreciation)}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-center text-sm text-gray-900">
                  {formatDate(asset.acquisitionDate)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-center">
                  {getStatusBadge(asset.status)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-center text-sm font-medium">
                  <button
                    onClick={() => handleEdit(asset)}
                    className="text-blue-600 hover:text-blue-900 mr-3"
                  >
                    Редактирай
                  </button>
                  {asset.status === 'active' && (
                    <button
                      onClick={() => handleDelete(asset.id)}
                      className="text-red-600 hover:text-red-900"
                    >
                      Изтрий
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {assets.length === 0 && (
          <div className="text-center py-12">
            <p className="text-gray-500">Няма намерени активи</p>
          </div>
        )}
      </div>

      {/* Modal */}
      {showModal && (
        <AssetModal
          asset={editingAsset}
          categories={categories}
          onClose={handleModalClose}
        />
      )}
    </>
  );
}