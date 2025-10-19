import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

const GET_AI_ACCOUNTING_SETTINGS = `
  query GetAiAccountingSettings($companyId: Int!) {
    aiAccountingSettings(companyId: $companyId) {
      id
      companyId
      salesRevenueAccount
      salesServicesAccount
      salesReceivablesAccount
      purchaseExpenseAccount
      purchasePayablesAccount
      vatInputAccount
      vatOutputAccount
      nonRegisteredPersonsAccount
      nonRegisteredVatOperation
      accountCodeLength
      salesDescriptionTemplate
      purchaseDescriptionTemplate
      createdAt
      updatedAt
    }
  }
`;

const UPSERT_AI_ACCOUNTING_SETTINGS = `
  mutation UpsertAiAccountingSettings($input: CreateAiAccountingSettingInput!) {
    upsertAiAccountingSettings(input: $input) {
      id
      companyId
      salesRevenueAccount
      salesServicesAccount
      salesReceivablesAccount
      purchaseExpenseAccount
      purchasePayablesAccount
      vatInputAccount
      vatOutputAccount
      nonRegisteredPersonsAccount
      nonRegisteredVatOperation
      accountCodeLength
      salesDescriptionTemplate
      purchaseDescriptionTemplate
    }
  }
`;

export default function AIAccountingSettings({ companyId }) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [formData, setFormData] = useState({
    salesRevenueAccount: '701',
    salesServicesAccount: '703',
    salesReceivablesAccount: '411',
    purchaseExpenseAccount: '602',
    purchasePayablesAccount: '401',
    vatInputAccount: '4531',
    vatOutputAccount: '4531',
    nonRegisteredPersonsAccount: '',
    nonRegisteredVatOperation: 'пок09',
    accountCodeLength: 3,
    salesDescriptionTemplate: '{counterpart} - {document_number}',
    purchaseDescriptionTemplate: '{counterpart} - {document_number}'
  });

  useEffect(() => {
    if (companyId) {
      loadSettings();
    }
  }, [companyId]);

  const loadSettings = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await graphqlRequest(GET_AI_ACCOUNTING_SETTINGS, { companyId });

      if (data.aiAccountingSettings) {
        const settings = data.aiAccountingSettings;
        setFormData({
          salesRevenueAccount: settings.salesRevenueAccount || '701',
          salesServicesAccount: settings.salesServicesAccount || '703',
          salesReceivablesAccount: settings.salesReceivablesAccount || '411',
          purchaseExpenseAccount: settings.purchaseExpenseAccount || '602',
          purchasePayablesAccount: settings.purchasePayablesAccount || '401',
          vatInputAccount: settings.vatInputAccount || '4531',
          vatOutputAccount: settings.vatOutputAccount || '4531',
          nonRegisteredPersonsAccount: settings.nonRegisteredPersonsAccount || '',
          nonRegisteredVatOperation: settings.nonRegisteredVatOperation || 'пок09',
          accountCodeLength: settings.accountCodeLength || 3,
          salesDescriptionTemplate: settings.salesDescriptionTemplate || '{counterpart} - {document_number}',
          purchaseDescriptionTemplate: settings.purchaseDescriptionTemplate || '{counterpart} - {document_number}'
        });
      }
    } catch (err) {
      setError(err.message);
      console.error('Failed to load AI accounting settings:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      await graphqlRequest(UPSERT_AI_ACCOUNTING_SETTINGS, {
        input: {
          companyId,
          salesRevenueAccount: formData.salesRevenueAccount,
          salesServicesAccount: formData.salesServicesAccount,
          salesReceivablesAccount: formData.salesReceivablesAccount,
          purchaseExpenseAccount: formData.purchaseExpenseAccount,
          purchasePayablesAccount: formData.purchasePayablesAccount,
          vatInputAccount: formData.vatInputAccount,
          vatOutputAccount: formData.vatOutputAccount,
          nonRegisteredPersonsAccount: formData.nonRegisteredPersonsAccount || null,
          nonRegisteredVatOperation: formData.nonRegisteredVatOperation,
          accountCodeLength: parseInt(formData.accountCodeLength),
          salesDescriptionTemplate: formData.salesDescriptionTemplate,
          purchaseDescriptionTemplate: formData.purchaseDescriptionTemplate
        }
      });

      alert('Настройките за AI счетоводство са запазени успешно');
      await loadSettings();
    } catch (err) {
      alert('Грешка: ' + err.message);
      console.error('Failed to save AI accounting settings:', err);
    }
  };

  const resetToDefaults = () => {
    setFormData({
      salesRevenueAccount: '701',
      salesServicesAccount: '703',
      salesReceivablesAccount: '411',
      purchaseExpenseAccount: '602',
      purchasePayablesAccount: '401',
      vatInputAccount: '4531',
      vatOutputAccount: '4531',
      nonRegisteredPersonsAccount: '',
      nonRegisteredVatOperation: 'пок09',
      accountCodeLength: 3,
      salesDescriptionTemplate: '{counterpart} - {document_number}',
      purchaseDescriptionTemplate: '{counterpart} - {document_number}'
    });
  };

  if (loading) {
    return <div className="text-center py-8">Зареждане...</div>;
  }

  if (error) {
    return <div className="text-center py-8 text-red-600">Грешка: {error}</div>;
  }

  if (!companyId) {
    return (
      <div className="text-center py-8 text-gray-500">
        Моля, изберете фирма за да конфигурирате настройките
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h3 className="text-lg font-medium text-gray-900 flex items-center gap-2">
            <span className="text-2xl">🤖</span>
            AI Счетоводни настройки
          </h3>
          <p className="mt-1 text-sm text-gray-500">
            Конфигурирайте сметките, които AI scanner ще използва автоматично при обработка на фактури
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Sales Accounts */}
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Продажби</h3>
          </div>
          <div className="px-4 py-5 sm:p-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Приходи от стоки
                </label>
                <input
                  type="text"
                  value={formData.salesRevenueAccount}
                  onChange={(e) => setFormData({ ...formData, salesRevenueAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="701"
                />
                <p className="mt-1 text-xs text-gray-500">Сметка 701 - Приходи от продажба на стоки</p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Приходи от услуги
                </label>
                <input
                  type="text"
                  value={formData.salesServicesAccount}
                  onChange={(e) => setFormData({ ...formData, salesServicesAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="703"
                />
                <p className="mt-1 text-xs text-gray-500">Сметка 703 - Приходи от продажба на услуги</p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Вземания от клиенти
                </label>
                <input
                  type="text"
                  value={formData.salesReceivablesAccount}
                  onChange={(e) => setFormData({ ...formData, salesReceivablesAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="411"
                />
                <p className="mt-1 text-xs text-gray-500">Сметка 411 - Клиенти</p>
              </div>
            </div>
          </div>
        </div>

        {/* Purchase Accounts */}
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Покупки</h3>
          </div>
          <div className="px-4 py-5 sm:p-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Разходи за стоки
                </label>
                <input
                  type="text"
                  value={formData.purchaseExpenseAccount}
                  onChange={(e) => setFormData({ ...formData, purchaseExpenseAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="602"
                />
                <p className="mt-1 text-xs text-gray-500">Сметка 602 - Разходи за материали</p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Задължения към доставчици
                </label>
                <input
                  type="text"
                  value={formData.purchasePayablesAccount}
                  onChange={(e) => setFormData({ ...formData, purchasePayablesAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="401"
                />
                <p className="mt-1 text-xs text-gray-500">Сметка 401 - Доставчици</p>
              </div>
            </div>
          </div>
        </div>

        {/* VAT Accounts */}
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">ДДС сметки</h3>
          </div>
          <div className="px-4 py-5 sm:p-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  ДДС за приспадане
                </label>
                <input
                  type="text"
                  value={formData.vatInputAccount}
                  onChange={(e) => setFormData({ ...formData, vatInputAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="4531"
                />
                <p className="mt-1 text-xs text-gray-500">Сметка 4531 - ДДС за приспадане (покупки)</p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700">
                  ДДС за внасяне
                </label>
                <input
                  type="text"
                  value={formData.vatOutputAccount}
                  onChange={(e) => setFormData({ ...formData, vatOutputAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="4531"
                />
                <p className="mt-1 text-xs text-gray-500">Сметка 4531 - ДДС за внасяне (продажби)</p>
              </div>
            </div>
          </div>
        </div>

        {/* Special Cases */}
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Специални случаи</h3>
          </div>
          <div className="px-4 py-5 sm:p-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Сметка за нерегистрирани лица (опционално)
                </label>
                <input
                  type="text"
                  value={formData.nonRegisteredPersonsAccount}
                  onChange={(e) => setFormData({ ...formData, nonRegisteredPersonsAccount: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  placeholder="Оставете празно за стандартна сметка"
                />
                <p className="mt-1 text-xs text-gray-500">Специална сметка за покупки от нерегистрирани лица (0% ДДС)</p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Операция за нерегистрирани лица
                </label>
                <select
                  value={formData.nonRegisteredVatOperation}
                  onChange={(e) => setFormData({ ...formData, nonRegisteredVatOperation: e.target.value })}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                >
                  <option value="пок09">пок09 - Покупки от нерегистрирани лица</option>
                  <option value="пок19">пок19 - Освободени от ДДС</option>
                  <option value="пок20">пок20 - Нулева ставка</option>
                </select>
                <p className="mt-1 text-xs text-gray-500">Операция в дневник ДДС покупки колона 09</p>
              </div>
            </div>
          </div>
        </div>

        {/* Formatting Options */}
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Форматиране</h3>
          </div>
          <div className="px-4 py-5 sm:p-6 space-y-6">
            <div>
              <label className="block text-sm font-medium text-gray-700">
                Дължина на кодовете на сметките
              </label>
              <select
                value={formData.accountCodeLength}
                onChange={(e) => setFormData({ ...formData, accountCodeLength: parseInt(e.target.value) })}
                className="mt-1 block w-64 border border-gray-300 rounded-md px-3 py-2"
              >
                <option value="3">3 цифри (напр. 701)</option>
                <option value="4">4 цифри (напр. 7011)</option>
              </select>
              <p className="mt-1 text-xs text-gray-500">Автоматично форматиране на кодовете (напр. 701 → 7011)</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Шаблон за описание (продажби)
              </label>
              <input
                type="text"
                value={formData.salesDescriptionTemplate}
                onChange={(e) => setFormData({ ...formData, salesDescriptionTemplate: e.target.value })}
                className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                placeholder="{counterpart} - {document_number}"
              />
              <p className="mt-1 text-xs text-gray-500">
                Променливи: {'{counterpart}'} - име на контрагент, {'{document_number}'} - номер на документ
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">
                Шаблон за описание (покупки)
              </label>
              <input
                type="text"
                value={formData.purchaseDescriptionTemplate}
                onChange={(e) => setFormData({ ...formData, purchaseDescriptionTemplate: e.target.value })}
                className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                placeholder="{counterpart} - {document_number}"
              />
              <p className="mt-1 text-xs text-gray-500">
                Променливи: {'{counterpart}'} - име на контрагент, {'{document_number}'} - номер на документ
              </p>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex justify-between items-center">
          <button
            type="button"
            onClick={resetToDefaults}
            className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            🔄 Възстанови по подразбиране
          </button>
          <button
            type="submit"
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
          >
            💾 Запази настройките
          </button>
        </div>
      </form>

      {/* Info Card */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
        <h4 className="font-semibold mb-2 flex items-center gap-2">
          💡 Как работи
        </h4>
        <ul className="list-disc list-inside space-y-1 text-sm text-gray-600">
          <li>AI Scanner автоматично използва тези настройки при създаване на счетоводни записи</li>
          <li>Сметките се избират автоматично въз основа на типа на документа (продажба/покупка)</li>
          <li>Описанията се генерират от шаблоните с информация от фактурата</li>
          <li>Кодовете на сметките се форматират автоматично спрямо избраната дължина</li>
        </ul>
      </div>
    </div>
  );
}
