import { useState } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

export default function UniversalExport() {
  const [exporting, setExporting] = useState(false);
  const [exportOptions, setExportOptions] = useState({
    includeDocuments: false,
    includeJournalEntries: false
  });

  const companyId = parseInt(localStorage.getItem('currentCompanyId') || '1');

  const EXPORT_ALL_MUTATION = `
    mutation ExportUniversalJson($input: UniversalExportInput!) {
      exportUniversalJson(input: $input)
    }
  `;

  const EXPORT_ACCOUNTS_MUTATION = `
    mutation ExportChartOfAccounts($companyId: Int!) {
      exportChartOfAccounts(companyId: $companyId)
    }
  `;

  const EXPORT_VAT_RATES_MUTATION = `
    mutation ExportVatRates($companyId: Int!) {
      exportVatRates(companyId: $companyId)
    }
  `;

  const EXPORT_COUNTERPARTS_MUTATION = `
    mutation ExportCounterparts($companyId: Int!) {
      exportCounterparts(companyId: $companyId)
    }
  `;

  const downloadJsonFile = (jsonString, filename) => {
    const blob = new Blob([jsonString], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const exportAll = async () => {
    setExporting(true);
    try {
      const result = await graphqlRequest(EXPORT_ALL_MUTATION, {
        input: {
          companyId: companyId,
          includeDocuments: exportOptions.includeDocuments,
          includeJournalEntries: exportOptions.includeJournalEntries
        }
      });

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
      downloadJsonFile(result.exportUniversalJson, `universal-export-${timestamp}.json`);

      alert('Експортът е завършен успешно!');
    } catch (error) {
      alert(`Грешка при експорт: ${error.message}`);
    } finally {
      setExporting(false);
    }
  };

  const exportAccounts = async () => {
    setExporting(true);
    try {
      const result = await graphqlRequest(EXPORT_ACCOUNTS_MUTATION, {
        companyId: companyId
      });

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
      downloadJsonFile(result.exportChartOfAccounts, `chart-of-accounts-${timestamp}.json`);

      alert('Сметкопланът е експортиран успешно!');
    } catch (error) {
      alert(`Грешка при експорт: ${error.message}`);
    } finally {
      setExporting(false);
    }
  };

  const exportVatRates = async () => {
    setExporting(true);
    try {
      const result = await graphqlRequest(EXPORT_VAT_RATES_MUTATION, {
        companyId: companyId
      });

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
      downloadJsonFile(result.exportVatRates, `vat-rates-${timestamp}.json`);

      alert('ДДС ставките са експортирани успешно!');
    } catch (error) {
      alert(`Грешка при експорт: ${error.message}`);
    } finally {
      setExporting(false);
    }
  };

  const exportCounterparts = async () => {
    setExporting(true);
    try {
      const result = await graphqlRequest(EXPORT_COUNTERPARTS_MUTATION, {
        companyId: companyId
      });

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
      downloadJsonFile(result.exportCounterparts, `counterparts-${timestamp}.json`);

      alert('Контрагентите са експортирани успешно!');
    } catch (error) {
      alert(`Грешка при експорт: ${error.message}`);
    } finally {
      setExporting(false);
    }
  };

  const exportTypes = [
    {
      id: 'all',
      icon: '📦',
      title: 'Пълен експорт',
      description: 'Всички данни - сметкоплан, ДДС ставки, контрагенти',
      action: exportAll,
      color: 'blue'
    },
    {
      id: 'accounts',
      icon: '🗂️',
      title: 'Само сметкоплан',
      description: 'Експорт само на сметките от сметкоплана',
      action: exportAccounts,
      color: 'green'
    },
    {
      id: 'vat',
      icon: '📊',
      title: 'Само ДДС ставки',
      description: 'Експорт само на ДДС ставки и кодове',
      action: exportVatRates,
      color: 'purple'
    },
    {
      id: 'counterparts',
      icon: '👥',
      title: 'Само контрагенти',
      description: 'Експорт само на контрагенти',
      action: exportCounterparts,
      color: 'orange'
    }
  ];

  const features = [
    'JSON формат съгласно универсална схема v2.0',
    'Съвместимост с други инсталации',
    'Възможност за частичен експорт',
    'Автоматично име на файла с дата',
    'Валидна структура за реимпорт',
    'Експорт само на активни записи'
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start space-x-4">
        <div className="text-4xl">📤</div>
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-gray-900">
            Универсален експорт - JSON формат v2.0
          </h3>
          <p className="text-gray-600 mb-3">
            Експорт на данни в JSON формат за прехвърляне в друга инсталация
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            {features.map((feature, index) => (
              <div key={index} className="flex items-center space-x-2">
                <span className="w-2 h-2 bg-blue-400 rounded-full"></span>
                <span className="text-sm text-gray-700">{feature}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Export Options for Full Export */}
      <div className="bg-blue-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-blue-500 text-xl">⚙️</div>
          <div className="flex-1">
            <div className="font-medium text-blue-900 mb-2">Допълнителни опции за пълен експорт:</div>
            <div className="space-y-2">
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={exportOptions.includeDocuments}
                  onChange={(e) => setExportOptions({...exportOptions, includeDocuments: e.target.checked})}
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  disabled={exporting}
                />
                <span className="text-sm text-blue-800">
                  Включи документи (фактури, протоколи) - <em>в момента не е имплементирано</em>
                </span>
              </label>
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={exportOptions.includeJournalEntries}
                  onChange={(e) => setExportOptions({...exportOptions, includeJournalEntries: e.target.checked})}
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  disabled={exporting}
                />
                <span className="text-sm text-blue-800">
                  Включи счетоводни записи - <em>в момента не е имплементирано</em>
                </span>
              </label>
            </div>
          </div>
        </div>
      </div>

      {/* Export Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {exportTypes.map((type) => (
          <div
            key={type.id}
            className={`border-2 rounded-lg p-6 hover:shadow-lg transition-all bg-white border-${type.color}-200 hover:border-${type.color}-400`}
          >
            <div className="flex items-start space-x-4">
              <div className="text-4xl">{type.icon}</div>
              <div className="flex-1">
                <h4 className="text-lg font-semibold text-gray-900 mb-1">
                  {type.title}
                </h4>
                <p className="text-sm text-gray-600 mb-4">
                  {type.description}
                </p>
                <button
                  onClick={type.action}
                  disabled={exporting}
                  className={`w-full px-4 py-2 text-sm font-medium text-white bg-${type.color}-600 border border-transparent rounded-md hover:bg-${type.color}-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed`}
                >
                  {exporting ? 'Експортира се...' : `Експортирай ${type.title.toLowerCase()}`}
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Usage Instructions */}
      <div className="bg-green-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-green-500 text-xl">💡</div>
          <div className="text-sm text-green-800">
            <div className="font-medium mb-2">Как да използвам експортираните данни:</div>
            <ol className="space-y-2 list-decimal list-inside">
              <li>
                <strong>Изберете тип експорт</strong> - пълен или частичен според нуждите ви
              </li>
              <li>
                <strong>Изтеглете JSON файла</strong> - файлът ще се запази автоматично с датата в името
              </li>
              <li>
                <strong>Прехвърлете файла</strong> на друг компютър или в друга инсталация
              </li>
              <li>
                <strong>Импортирайте файла</strong> в другата инсталация чрез "Универсален импорт"
              </li>
              <li>
                <strong>Проверете резултата</strong> - системата ще покаже колко записа са импортирани
              </li>
            </ol>
            <div className="mt-3 pt-3 border-t border-green-200">
              <p className="font-medium mb-1">Важно:</p>
              <ul className="space-y-1">
                <li>• Експортират се само <strong>активни записи</strong></li>
                <li>• Форматът е съвместим с Universal Schema v2.0</li>
                <li>• Файлът може да се редактира ръчно преди импорт</li>
                <li>• При импорт дубликатите се пропускат автоматично</li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      {/* Info Box */}
      <div className="bg-yellow-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-yellow-500 text-xl">ℹ️</div>
          <div className="text-sm text-yellow-800">
            <div className="font-medium mb-1">Формат на експортирания файл:</div>
            <p>
              Експортираният JSON файл следва Universal Schema v2.0 и съдържа:
            </p>
            <ul className="mt-2 space-y-1">
              <li>• <code className="bg-yellow-100 px-1 rounded">companyInfo</code> - информация за фирмата</li>
              <li>• <code className="bg-yellow-100 px-1 rounded">chartOfAccounts</code> - масив със сметки</li>
              <li>• <code className="bg-yellow-100 px-1 rounded">vatRates</code> - масив с ДДС ставки</li>
              <li>• <code className="bg-yellow-100 px-1 rounded">counterparts</code> - масив с контрагенти</li>
              <li>• <code className="bg-yellow-100 px-1 rounded">importSettings</code> - настройки за импорт</li>
            </ul>
            <p className="mt-2">
              Файлът може директно да се импортира в друга инсталация на системата.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
