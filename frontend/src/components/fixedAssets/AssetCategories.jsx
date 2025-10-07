import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

// Edit Category Modal Component
function EditCategoryModal({ category, onClose, onSave }) {
  const [formData, setFormData] = useState({
    assetAccountCode: category.assetAccountCode || '',
    depreciationAccountCode: category.depreciationAccountCode || '',
    expenseAccountCode: category.expenseAccountCode || '',
    defaultAccountingDepreciationRate: category.defaultAccountingDepreciationRate || category.maxTaxDepreciationRate
  });
  const [saving, setSaving] = useState(false);
  const [accounts, setAccounts] = useState([]);

  // Load accounts for dropdown
  useEffect(() => {
    loadAccounts();
  }, []);

  const loadAccounts = async () => {
    const ACCOUNTS_QUERY = `
      query GetAccounts($companyId: Int!) {
        accounts(companyId: $companyId) {
          code
          name
          accountType
        }
      }
    `;

    try {
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      const response = await graphqlRequest(ACCOUNTS_QUERY, { companyId });
      setAccounts(response.accounts || []);
    } catch (err) {
      console.error('Error loading accounts:', err);
    }
  };

  const UPDATE_CATEGORY_MUTATION = `
    mutation UpdateAssetCategory($id: Int!, $input: UpdateAssetCategoryInput!) {
      updateFixedAssetCategory(id: $id, input: $input) {
        id
        assetAccountCode
        depreciationAccountCode
        expenseAccountCode
        defaultAccountingDepreciationRate
      }
    }
  `;

  const handleSave = async () => {
    try {
      setSaving(true);
      await graphqlRequest(UPDATE_CATEGORY_MUTATION, {
        id: category.id,
        input: formData
      });
      alert('Категорията е обновена успешно!');
      onSave();
      onClose();
    } catch (err) {
      alert('Грешка при запазване: ' + err.message);
    } finally {
      setSaving(false);
    }
  };

  // Filter accounts by type for suggestions
  const assetAccounts = accounts.filter(a => a.code.startsWith('20') || a.code.startsWith('21'));
  const depreciationAccounts = accounts.filter(a => a.code.startsWith('20') || a.code.startsWith('21'));
  const expenseAccounts = accounts.filter(a => a.code.startsWith('603'));

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-2/3 lg:w-1/2 shadow-lg rounded-md bg-white">
        <div className="mt-3">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">
              Настройки на категория: {category.name}
            </h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>

          <div className="space-y-4">
            <div className="bg-blue-50 p-3 rounded-lg">
              <p className="text-sm text-blue-800">
                Категория {category.taxCategory} по ЗКПО • Код: {category.code}
              </p>
              <p className="text-xs text-blue-600 mt-1">
                Максимална данъчна норма: {category.maxTaxDepreciationRate}%
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Счетоводна амортизационна норма (%)</label>
              <input
                type="number"
                step="0.1"
                min="0"
                max="100"
                value={formData.defaultAccountingDepreciationRate}
                onChange={(e) => setFormData({...formData, defaultAccountingDepreciationRate: parseFloat(e.target.value)})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                placeholder={`По подразбиране: ${category.maxTaxDepreciationRate}%`}
              />
              <p className="text-xs text-gray-500 mt-1">
                Счетоводната норма може да се различава от данъчната (макс. {category.maxTaxDepreciationRate}%)
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Сметка за ДМА активи
              </label>
              <input
                type="text"
                value={formData.assetAccountCode}
                onChange={(e) => setFormData({...formData, assetAccountCode: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                placeholder="напр. 203, 204, 206"
                list="asset-accounts"
              />
              <datalist id="asset-accounts">
                {assetAccounts.map(acc => (
                  <option key={acc.code} value={acc.code}>
                    {acc.name}
                  </option>
                ))}
              </datalist>
              <p className="text-xs text-gray-500 mt-1">
                Сметка от група 20x за отчитане на активите (напр. 203 за машини, 204 за транспорт)
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Сметка за натрупана амортизация
              </label>
              <input
                type="text"
                value={formData.depreciationAccountCode}
                onChange={(e) => setFormData({...formData, depreciationAccountCode: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                placeholder="напр. 2031, 2041, 2061"
                list="depreciation-accounts"
              />
              <datalist id="depreciation-accounts">
                {depreciationAccounts.map(acc => (
                  <option key={acc.code} value={acc.code}>
                    {acc.name}
                  </option>
                ))}
              </datalist>
              <p className="text-xs text-gray-500 mt-1">
                Коректив към сметката за активи (напр. 2031 за натрупана амортизация на машини)
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Сметка за разход за амортизация
              </label>
              <input
                type="text"
                value={formData.expenseAccountCode}
                onChange={(e) => setFormData({...formData, expenseAccountCode: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                placeholder="напр. 603"
                list="expense-accounts"
              />
              <datalist id="expense-accounts">
                {expenseAccounts.map(acc => (
                  <option key={acc.code} value={acc.code}>
                    {acc.name}
                  </option>
                ))}
              </datalist>
              <p className="text-xs text-gray-500 mt-1">
                Разходна сметка за начисляване на амортизацията (обикновено 603)
              </p>
            </div>

            <div className="bg-yellow-50 p-3 rounded-lg">
              <p className="text-xs text-yellow-800">
                <strong>Препоръчителни сметки според НСС:</strong><br/>
                • Сгради: 201 / 2011 / 603<br/>
                • Машини: 203 / 2031 / 603<br/>
                • Транспорт: 204 / 2041 / 603<br/>
                • Компютри: 206 / 2061 / 603<br/>
                • МОК: 209 / 603
              </p>
            </div>
          </div>

          <div className="flex items-center justify-end space-x-3 mt-6">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
            >
              Отказ
            </button>
            <button
              onClick={handleSave}
              disabled={saving}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
            >
              {saving ? 'Запазва...' : 'Запази'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function AssetCategories() {
  const [categories, setCategories] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showEditModal, setShowEditModal] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState(null);

  const CATEGORIES_QUERY = `
    query GetCategories {
      fixedAssetCategories {
        id
        code
        name
        description
        taxCategory
        maxTaxDepreciationRate
        defaultAccountingDepreciationRate
        minUsefulLife
        maxUsefulLife
        assetAccountCode
        depreciationAccountCode
        expenseAccountCode
        isActive
      }
    }
  `;

  useEffect(() => {
    loadCategories();
  }, []);

  const loadCategories = async () => {
    try {
      setLoading(true);
      const response = await graphqlRequest(CATEGORIES_QUERY);
      setCategories(response.fixedAssetCategories || []);
    } catch (err) {
      console.error('Error loading categories:', err);
    } finally {
      setLoading(false);
    }
  };

  const getCategoryIcon = (code) => {
    const icons = {
      'BUILDINGS': '🏢',
      'MACHINERY': '⚙️',
      'TRANSPORT': '🚚',
      'COMPUTERS': '💻',
      'VEHICLES': '🚗',
      'OTHER': '📦'
    };
    return icons[code] || '📄';
  };

  const formatLifespan = (minMonths, maxMonths) => {
    if (!minMonths && !maxMonths) return '-';
    
    const minYears = minMonths ? Math.floor(minMonths / 12) : null;
    const maxYears = maxMonths ? Math.floor(maxMonths / 12) : null;
    
    if (minYears && maxYears) {
      return `${minYears}-${maxYears} години`;
    } else if (minYears) {
      return `мин. ${minYears} години`;
    } else if (maxYears) {
      return `макс. ${maxYears} години`;
    }
    return '-';
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
      <div className="mb-6">
        <h3 className="text-lg font-medium text-gray-900">Категории според ЗКПО</h3>
        <p className="mt-1 text-sm text-gray-500">
          Категории дълготрайни активи според Закона за корпоративното подоходно облагане
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {categories.map(category => (
          <div key={category.id} className="bg-white border border-gray-200 rounded-lg p-6">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center text-2xl">
                  {getCategoryIcon(category.code)}
                </div>
              </div>
              <div className="ml-4 flex-1">
                <h4 className="text-lg font-medium text-gray-900">
                  {category.name}
                </h4>
                <p className="text-sm text-gray-500 mt-1">
                  Категория {category.taxCategory} по ЗКПО • Код: {category.code}
                </p>
                
                {category.description && (
                  <p className="text-sm text-gray-600 mt-2">
                    {category.description}
                  </p>
                )}

                <div className="mt-4 grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-gray-500">Макс. данъчна норма:</span>
                    <div className="font-medium text-gray-900">
                      {category.maxTaxDepreciationRate}%
                      {category.code === 'MACHINERY' && (
                        <span className="text-xs text-green-600 ml-1">
                          (50% за нови)
                        </span>
                      )}
                    </div>
                  </div>
                  
                  <div>
                    <span className="text-gray-500">Счетоводна норма:</span>
                    <div className="font-medium text-gray-900">
                      {category.defaultAccountingDepreciationRate || category.maxTaxDepreciationRate}%
                    </div>
                  </div>
                  
                  <div>
                    <span className="text-gray-500">Полезен живот:</span>
                    <div className="font-medium text-gray-900">
                      {formatLifespan(category.minUsefulLife, category.maxUsefulLife)}
                    </div>
                  </div>
                  
                  <div>
                    <span className="text-gray-500">Статус:</span>
                    <div>
                      {category.isActive ? (
                        <span className="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800">
                          Активна
                        </span>
                      ) : (
                        <span className="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800">
                          Неактивна
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-gray-200">
                  <div className="text-xs text-gray-500">Счетоводни сметки:</div>
                  <div className="mt-1 flex flex-wrap gap-2">
                    <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-blue-100 text-blue-700">
                      {category.assetAccountCode} Актив
                    </span>
                    <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-yellow-100 text-yellow-700">
                      {category.depreciationAccountCode} Натрупана
                    </span>
                    <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-red-100 text-red-700">
                      {category.expenseAccountCode} Разход
                    </span>
                  </div>
                </div>

                <div className="mt-4 flex justify-end">
                  <button
                    onClick={() => {
                      setSelectedCategory(category);
                      setShowEditModal(true);
                    }}
                    className="inline-flex items-center px-3 py-1.5 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                  >
                    <svg className="mr-1.5 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                    Настрой сметки
                  </button>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-8 p-4 bg-blue-50 rounded-lg">
        <h4 className="text-sm font-medium text-blue-900 mb-2">
          Информация за данъчните категории
        </h4>
        <ul className="text-sm text-blue-800 space-y-1">
          <li>• Категория I (Сгради) - макс. 4% годишна норма</li>
          <li>• Категория II (Машини) - макс. 30% (50% за нови първоначални инвестиции)</li>
          <li>• Категория III (Транспорт без автомобили) - макс. 10%</li>
          <li>• Категория IV (Компютри и софтуер) - макс. 50%</li>
          <li>• Категория V (Автомобили) - макс. 25%</li>
          <li>• Категория VII (Други ДМА) - макс. 15%</li>
        </ul>
      </div>

      {/* Edit Category Modal */}
      {showEditModal && selectedCategory && (
        <EditCategoryModal
          category={selectedCategory}
          onClose={() => {
            setShowEditModal(false);
            setSelectedCategory(null);
          }}
          onSave={() => {
            loadCategories();
            setShowEditModal(false);
            setSelectedCategory(null);
          }}
        />
      )}
    </div>
  );
}