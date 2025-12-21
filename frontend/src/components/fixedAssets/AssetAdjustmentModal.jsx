import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

export default function AssetAdjustmentModal({ asset, type, onClose }) {
  const [formData, setFormData] = useState({
    adjustmentDate: new Date().toISOString().split('T')[0],
    amount: '',
    reason: '',
    documentNumber: '',
    newAccountingRate: '',
    newTaxRate: '',
    newAccountingLife: '',
    newTaxLife: '',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const IMPROVEMENT_MUTATION = `
    mutation ApplyImprovement($input: ApplyImprovementInput!, $userId: Int!) {
      applyAssetImprovement(input: $input, userId: $userId) {
        success
        adjustmentId
        journalEntryId
        depreciationPeriodsCalculated
        message
      }
    }
  `;

  const IMPAIRMENT_MUTATION = `
    mutation ApplyImpairment($input: ApplyImpairmentInput!, $userId: Int!) {
      applyAssetImpairment(input: $input, userId: $userId) {
        success
        adjustmentId
        journalEntryId
        depreciationPeriodsCalculated
        message
      }
    }
  `;

  const handleSubmit = async (e) => {
    e.preventDefault();

    if (!formData.amount || parseFloat(formData.amount) <= 0) {
      setError('Моля въведете валидна сума');
      return;
    }

    if (!formData.reason) {
      setError('Моля въведете основание');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const userId = parseInt(localStorage.getItem('userId')) || 1;

      if (type === 'improvement') {
        const input = {
          assetId: asset.id,
          adjustmentDate: formData.adjustmentDate,
          improvementAmount: parseFloat(formData.amount),
          reason: formData.reason,
          documentNumber: formData.documentNumber || null,
          newAccountingDepreciationRate: formData.newAccountingRate ? parseFloat(formData.newAccountingRate) : null,
          newTaxDepreciationRate: formData.newTaxRate ? parseFloat(formData.newTaxRate) : null,
          newAccountingUsefulLife: formData.newAccountingLife ? parseInt(formData.newAccountingLife) : null,
          newTaxUsefulLife: formData.newTaxLife ? parseInt(formData.newTaxLife) : null,
        };

        const response = await graphqlRequest(IMPROVEMENT_MUTATION, { input, userId });

        if (response.applyAssetImprovement?.success) {
          alert(response.applyAssetImprovement.message);
          onClose();
        }
      } else if (type === 'impairment') {
        const input = {
          assetId: asset.id,
          adjustmentDate: formData.adjustmentDate,
          impairmentAmount: parseFloat(formData.amount),
          reason: formData.reason,
          documentNumber: formData.documentNumber || null,
        };

        const response = await graphqlRequest(IMPAIRMENT_MUTATION, { input, userId });

        if (response.applyAssetImpairment?.success) {
          alert(response.applyAssetImpairment.message);
          onClose();
        }
      }
    } catch (err) {
      console.error('Error applying adjustment:', err);
      setError(err.message || 'Грешка при прилагане на корекция');
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

  const isImprovement = type === 'improvement';
  const title = isImprovement ? 'Увеличение на стойността' : 'Намаление на стойността';
  const icon = isImprovement ? '📈' : '📉';
  const buttonColor = isImprovement ? 'bg-green-600 hover:bg-green-700' : 'bg-orange-600 hover:bg-orange-700';

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-full max-w-3xl shadow-lg rounded-md bg-white">
        <div className="flex justify-between items-center mb-6">
          <h3 className="text-xl font-bold text-gray-900 flex items-center">
            <span className="mr-2 text-2xl">{icon}</span>
            {title}
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-2xl"
          >
            ×
          </button>
        </div>

        {/* Asset Info */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-gray-600">Актив</p>
              <p className="font-semibold">{asset.inventoryNumber} - {asset.name}</p>
            </div>
            <div>
              <p className="text-sm text-gray-600">Категория</p>
              <p className="font-semibold">{asset.category?.name}</p>
            </div>
            <div>
              <p className="text-sm text-gray-600">Текуща счетоводна стойност</p>
              <p className="font-semibold text-blue-600">{formatCurrency(asset.accountingBookValue)}</p>
            </div>
            <div>
              <p className="text-sm text-gray-600">Текуща данъчна стойност</p>
              <p className="font-semibold text-blue-600">{formatCurrency(asset.taxBookValue)}</p>
            </div>
          </div>
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-800 text-sm">{error}</p>
          </div>
        )}

        <form onSubmit={handleSubmit}>
          <div className="space-y-4">
            {/* Basic Fields */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Дата на корекцията *
                </label>
                <input
                  type="date"
                  value={formData.adjustmentDate}
                  onChange={(e) => setFormData({ ...formData, adjustmentDate: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Сума {isImprovement ? 'увеличение' : 'намаление'} (BGN) *
                </label>
                <input
                  type="number"
                  step="0.01"
                  value={formData.amount}
                  onChange={(e) => setFormData({ ...formData, amount: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="0.00"
                  required
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Основание *
              </label>
              <textarea
                value={formData.reason}
                onChange={(e) => setFormData({ ...formData, reason: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                rows="3"
                placeholder={isImprovement
                  ? 'Напр.: Подобрение на актива, модернизация, преустройство...'
                  : 'Напр.: Физическо повреждане, обезценка, морално остаряване...'}
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Номер на документ
              </label>
              <input
                type="text"
                value={formData.documentNumber}
                onChange={(e) => setFormData({ ...formData, documentNumber: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Напр.: Протокол №, Фактура №..."
              />
            </div>

            {/* Depreciation Parameters - Only for improvements */}
            {isImprovement && (
              <div className="border-t pt-4">
                <h4 className="font-medium text-gray-900 mb-3">
                  Промяна на параметри на амортизация (незадължително)
                </h4>
                <p className="text-sm text-gray-600 mb-4">
                  При увеличение на стойността можете да промените нормите и срока на амортизация
                </p>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Нова счетоводна норма (%)
                    </label>
                    <input
                      type="number"
                      step="0.01"
                      value={formData.newAccountingRate}
                      onChange={(e) => setFormData({ ...formData, newAccountingRate: e.target.value })}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder={asset.accountingDepreciationRate}
                    />
                    <p className="text-xs text-gray-500 mt-1">
                      Текуща: {asset.accountingDepreciationRate}%
                    </p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Нова данъчна норма (%)
                    </label>
                    <input
                      type="number"
                      step="0.01"
                      value={formData.newTaxRate}
                      onChange={(e) => setFormData({ ...formData, newTaxRate: e.target.value })}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder={asset.taxDepreciationRate}
                    />
                    <p className="text-xs text-gray-500 mt-1">
                      Текуща: {asset.taxDepreciationRate}%
                    </p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Нов счетоводен срок (месеци)
                    </label>
                    <input
                      type="number"
                      value={formData.newAccountingLife}
                      onChange={(e) => setFormData({ ...formData, newAccountingLife: e.target.value })}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder={asset.accountingUsefulLife}
                    />
                    <p className="text-xs text-gray-500 mt-1">
                      Текущ: {asset.accountingUsefulLife} месеца
                    </p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Нов данъчен срок (месеци)
                    </label>
                    <input
                      type="number"
                      value={formData.newTaxLife}
                      onChange={(e) => setFormData({ ...formData, newTaxLife: e.target.value })}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder={asset.taxUsefulLife || 'Не е зададен'}
                    />
                    <p className="text-xs text-gray-500 mt-1">
                      Текущ: {asset.taxUsefulLife || 'Не е зададен'}
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* Info Note */}
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
              <p className="text-sm text-yellow-800">
                <strong>Важно:</strong> Автоматично ще се начисли амортизация до месеца на корекцията.
                {isImprovement && ' Новата стойност ще увеличи отчетната стойност и балансовите стойности.'}
                {!isImprovement && ' Ще се намалят само балансовите стойности.'}
              </p>
            </div>
          </div>

          {/* Actions */}
          <div className="mt-6 flex justify-end space-x-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
              disabled={loading}
            >
              Отказ
            </button>
            <button
              type="submit"
              className={`px-4 py-2 text-white rounded-md ${buttonColor} disabled:opacity-50`}
              disabled={loading}
            >
              {loading ? 'Прилагане...' : (isImprovement ? 'Приложи увеличение' : 'Приложи намаление')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
