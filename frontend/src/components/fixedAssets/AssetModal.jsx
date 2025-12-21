import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

export default function AssetModal({ asset, categories, onClose }) {
  const [formData, setFormData] = useState({
    inventoryNumber: '',
    name: '',
    description: '',
    categoryId: '',
    acquisitionCost: '',
    acquisitionDate: '',
    putIntoServiceDate: '',
    accountingUsefulLife: 60,
    accountingDepreciationRate: 20,
    accountingDepreciationMethod: 'straight_line',
    accountingSalvageValue: 0,
    taxUsefulLife: 60,
    taxDepreciationRate: 20,
    isNewFirstTimeInvestment: false,
    location: '',
    responsiblePerson: '',
    serialNumber: '',
    manufacturer: '',
    model: '',
    notes: '',
    status: 'active',
    disposalDate: '',
    disposalAmount: ''
  });

  const [errors, setErrors] = useState({});
  const [loading, setLoading] = useState(false);

  const CREATE_MUTATION = `
    mutation CreateFixedAsset($input: CreateFixedAssetInput!, $companyId: Int!) {
      createFixedAsset(input: $input, companyId: $companyId) {
        id
        inventoryNumber
        name
      }
    }
  `;

  const UPDATE_MUTATION = `
    mutation UpdateFixedAsset($input: UpdateFixedAssetInput!) {
      updateFixedAsset(input: $input) {
        id
        inventoryNumber
        name
      }
    }
  `;

  useEffect(() => {
    if (asset) {
      setFormData({
        ...asset,
        categoryId: asset.categoryId.toString(),
        acquisitionCost: asset.acquisitionCost.toString(),
        accountingUsefulLife: asset.accountingUsefulLife || 60,
        accountingDepreciationRate: asset.accountingDepreciationRate || 20,
        accountingDepreciationMethod: asset.accountingDepreciationMethod || 'straight_line',
        accountingSalvageValue: asset.accountingSalvageValue || 0,
        taxUsefulLife: asset.taxUsefulLife || 60,
        taxDepreciationRate: asset.taxDepreciationRate || 20,
        isNewFirstTimeInvestment: asset.isNewFirstTimeInvestment || false
      });
    }
  }, [asset]);

  useEffect(() => {
    // Auto-calculate depreciation rate based on useful life
    if (formData.accountingUsefulLife > 0) {
      const yearlyRate = (100 * 12) / formData.accountingUsefulLife;
      setFormData(prev => ({ ...prev, accountingDepreciationRate: yearlyRate.toFixed(2) }));
    }
  }, [formData.accountingUsefulLife]);

  useEffect(() => {
    // Set tax depreciation rate based on category
    if (formData.categoryId) {
      const category = categories.find(c => c.id.toString() === formData.categoryId);
      if (category) {
        let taxRate = category.maxTaxDepreciationRate;
        
        // Category II (Machinery) can have 50% for new investments
        if (category.taxCategory === 2 && formData.isNewFirstTimeInvestment) {
          taxRate = 50;
        }
        
        setFormData(prev => ({ ...prev, taxDepreciationRate: taxRate }));
      }
    }
  }, [formData.categoryId, formData.isNewFirstTimeInvestment, categories]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    // Validation
    const newErrors = {};
    if (!formData.inventoryNumber) newErrors.inventoryNumber = 'Задължително поле';
    if (!formData.name) newErrors.name = 'Задължително поле';
    if (!formData.categoryId) newErrors.categoryId = 'Задължително поле';
    if (!formData.acquisitionCost) newErrors.acquisitionCost = 'Задължително поле';
    if (!formData.acquisitionDate) newErrors.acquisitionDate = 'Задължително поле';
    
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    try {
      setLoading(true);
      
      if (asset) {
        // Update
        const input = {
          id: asset.id,
          inventoryNumber: formData.inventoryNumber,
          name: formData.name,
          description: formData.description || null,
          location: formData.location || null,
          responsiblePerson: formData.responsiblePerson || null,
          serialNumber: formData.serialNumber || null,
          manufacturer: formData.manufacturer || null,
          model: formData.model || null,
          notes: formData.notes || null,
          status: formData.status,
          disposalDate: formData.disposalDate || null,
          disposalAmount: formData.disposalAmount ? parseFloat(formData.disposalAmount) : null
        };
        
        await graphqlRequest(UPDATE_MUTATION, { input });
      } else {
        // Create
        const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
        const input = {
          inventoryNumber: formData.inventoryNumber,
          name: formData.name,
          description: formData.description || null,
          categoryId: parseInt(formData.categoryId),
          acquisitionCost: parseFloat(formData.acquisitionCost),
          acquisitionDate: formData.acquisitionDate,
          putIntoServiceDate: formData.putIntoServiceDate || null,
          accountingUsefulLife: parseInt(formData.accountingUsefulLife),
          accountingDepreciationRate: parseFloat(formData.accountingDepreciationRate),
          accountingDepreciationMethod: formData.accountingDepreciationMethod,
          accountingSalvageValue: parseFloat(formData.accountingSalvageValue || 0),
          taxUsefulLife: parseInt(formData.taxUsefulLife || formData.accountingUsefulLife),
          taxDepreciationRate: parseFloat(formData.taxDepreciationRate),
          isNewFirstTimeInvestment: formData.isNewFirstTimeInvestment,
          location: formData.location || null,
          responsiblePerson: formData.responsiblePerson || null,
          serialNumber: formData.serialNumber || null,
          manufacturer: formData.manufacturer || null,
          model: formData.model || null,
          notes: formData.notes || null
        };
        
        await graphqlRequest(CREATE_MUTATION, { input, companyId });
      }
      
      onClose();
    } catch (err) {
      alert('Грешка: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-medium text-gray-900">
            {asset ? 'Редактиране на актив' : 'Нов дълготраен актив'}
          </h3>
        </div>

        <form onSubmit={handleSubmit} className="p-6">
          <div className="grid grid-cols-2 gap-6">
            {/* Basic Info */}
            <div className="col-span-2">
              <h4 className="text-sm font-medium text-gray-900 mb-4">Основна информация</h4>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700">
                Инвентарен номер *
              </label>
              <input
                type="text"
                value={formData.inventoryNumber}
                onChange={(e) => setFormData({ ...formData, inventoryNumber: e.target.value })}
                className={`mt-1 block w-full rounded-md border ${errors.inventoryNumber ? 'border-red-500' : 'border-gray-300'} px-3 py-2`}
                disabled={asset}
              />
              {errors.inventoryNumber && <p className="text-red-500 text-xs mt-1">{errors.inventoryNumber}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Наименование *
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className={`mt-1 block w-full rounded-md border ${errors.name ? 'border-red-500' : 'border-gray-300'} px-3 py-2`}
              />
              {errors.name && <p className="text-red-500 text-xs mt-1">{errors.name}</p>}
            </div>

            <div className="col-span-2">
              <label className="block text-sm font-medium text-gray-700">
                Описание
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows="2"
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Категория ЗКПО *
              </label>
              <select
                value={formData.categoryId}
                onChange={(e) => setFormData({ ...formData, categoryId: e.target.value })}
                className={`mt-1 block w-full rounded-md border ${errors.categoryId ? 'border-red-500' : 'border-gray-300'} px-3 py-2`}
                disabled={asset}
              >
                <option value="">-- Избери --</option>
                {categories.map(cat => (
                  <option key={cat.id} value={cat.id}>
                    {cat.name} (макс. {cat.maxTaxDepreciationRate}%)
                  </option>
                ))}
              </select>
              {errors.categoryId && <p className="text-red-500 text-xs mt-1">{errors.categoryId}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Местоположение
              </label>
              <input
                type="text"
                value={formData.location}
                onChange={(e) => setFormData({ ...formData, location: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
              />
            </div>

            {/* Financial Info */}
            <div className="col-span-2 mt-4">
              <h4 className="text-sm font-medium text-gray-900 mb-4">Финансови данни</h4>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Отчетна стойност *
              </label>
              <input
                type="number"
                step="0.01"
                value={formData.acquisitionCost}
                onChange={(e) => setFormData({ ...formData, acquisitionCost: e.target.value })}
                className={`mt-1 block w-full rounded-md border ${errors.acquisitionCost ? 'border-red-500' : 'border-gray-300'} px-3 py-2`}
                disabled={asset}
              />
              {errors.acquisitionCost && <p className="text-red-500 text-xs mt-1">{errors.acquisitionCost}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Дата на придобиване *
              </label>
              <input
                type="date"
                value={formData.acquisitionDate}
                onChange={(e) => setFormData({ ...formData, acquisitionDate: e.target.value })}
                className={`mt-1 block w-full rounded-md border ${errors.acquisitionDate ? 'border-red-500' : 'border-gray-300'} px-3 py-2`}
                disabled={asset}
              />
              {errors.acquisitionDate && <p className="text-red-500 text-xs mt-1">{errors.acquisitionDate}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Дата въвеждане в експлоатация
              </label>
              <input
                type="date"
                value={formData.putIntoServiceDate}
                onChange={(e) => setFormData({ ...formData, putIntoServiceDate: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
                disabled={asset}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Остатъчна стойност
              </label>
              <input
                type="number"
                step="0.01"
                value={formData.accountingSalvageValue}
                onChange={(e) => setFormData({ ...formData, accountingSalvageValue: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
                disabled={asset}
              />
            </div>

            {/* Depreciation Settings */}
            <div className="col-span-2 mt-4">
              <h4 className="text-sm font-medium text-gray-900 mb-4">Амортизация</h4>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Счетоводен полезен живот (месеци)
              </label>
              <input
                type="number"
                value={formData.accountingUsefulLife}
                onChange={(e) => setFormData({ ...formData, accountingUsefulLife: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
                disabled={asset}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Счетоводна норма (%)
              </label>
              <input
                type="number"
                step="0.01"
                value={formData.accountingDepreciationRate}
                readOnly
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 bg-gray-50"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Данъчен полезен живот (месеци)
              </label>
              <input
                type="number"
                value={formData.taxUsefulLife}
                onChange={(e) => setFormData({ ...formData, taxUsefulLife: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
                disabled={asset}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Данъчна норма ЗКПО (%)
              </label>
              <input
                type="number"
                step="0.01"
                value={formData.taxDepreciationRate}
                readOnly
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 bg-gray-50"
              />
            </div>

            {!asset && formData.categoryId && categories.find(c => c.id.toString() === formData.categoryId)?.taxCategory === 2 && (
              <div className="col-span-2">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={formData.isNewFirstTimeInvestment}
                    onChange={(e) => setFormData({ ...formData, isNewFirstTimeInvestment: e.target.checked })}
                    className="mr-2"
                  />
                  <span className="text-sm text-gray-700">
                    Нова първоначална инвестиция (50% данъчна норма за машини)
                  </span>
                </label>
              </div>
            )}

            {/* Status (only for edit) */}
            {asset && (
              <>
                <div className="col-span-2 mt-4">
                  <h4 className="text-sm font-medium text-gray-900 mb-4">Статус</h4>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Статус на актива
                  </label>
                  <select
                    value={formData.status}
                    onChange={(e) => setFormData({ ...formData, status: e.target.value })}
                    className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
                  >
                    <option value="active">Активен</option>
                    <option value="disposed">Ликвидиран</option>
                    <option value="sold">Продаден</option>
                  </select>
                </div>

                {(formData.status === 'disposed' || formData.status === 'sold') && (
                  <>
                    <div>
                      <label className="block text-sm font-medium text-gray-700">
                        Дата на изваждане
                      </label>
                      <input
                        type="date"
                        value={formData.disposalDate}
                        onChange={(e) => setFormData({ ...formData, disposalDate: e.target.value })}
                        className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700">
                        Стойност при продажба
                      </label>
                      <input
                        type="number"
                        step="0.01"
                        value={formData.disposalAmount}
                        onChange={(e) => setFormData({ ...formData, disposalAmount: e.target.value })}
                        className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
                      />
                    </div>
                  </>
                )}
              </>
            )}

            {/* Additional Info */}
            <div className="col-span-2 mt-4">
              <h4 className="text-sm font-medium text-gray-900 mb-4">Допълнителна информация</h4>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Отговорно лице
              </label>
              <input
                type="text"
                value={formData.responsiblePerson}
                onChange={(e) => setFormData({ ...formData, responsiblePerson: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Сериен номер
              </label>
              <input
                type="text"
                value={formData.serialNumber}
                onChange={(e) => setFormData({ ...formData, serialNumber: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Производител
              </label>
              <input
                type="text"
                value={formData.manufacturer}
                onChange={(e) => setFormData({ ...formData, manufacturer: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Модел
              </label>
              <input
                type="text"
                value={formData.model}
                onChange={(e) => setFormData({ ...formData, model: e.target.value })}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
              />
            </div>

            <div className="col-span-2">
              <label className="block text-sm font-medium text-gray-700">
                Бележки
              </label>
              <textarea
                value={formData.notes}
                onChange={(e) => setFormData({ ...formData, notes: e.target.value })}
                rows="2"
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2"
              />
            </div>
          </div>

          <div className="mt-6 flex justify-end space-x-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
            >
              Отказ
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
            >
              {loading ? 'Запазване...' : (asset ? 'Запази' : 'Създай')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}