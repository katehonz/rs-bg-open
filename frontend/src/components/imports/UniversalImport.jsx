import { useState, useCallback, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

export default function UniversalImport() {
  const [dragOver, setDragOver] = useState(false);
  const [uploadedFiles, setUploadedFiles] = useState([]);
  const [importing, setImporting] = useState(false);
  const [importResults, setImportResults] = useState(null);
  const [currentCompanyId, setCurrentCompanyId] = useState(
    parseInt(localStorage.getItem('currentCompanyId') || '1')
  );

  // Update company ID when localStorage changes
  useEffect(() => {
    const handleStorageChange = () => {
      const newCompanyId = parseInt(localStorage.getItem('currentCompanyId') || '1');
      setCurrentCompanyId(newCompanyId);
    };

    window.addEventListener('storage', handleStorageChange);
    // Also check periodically for same-window changes
    const interval = setInterval(handleStorageChange, 1000);

    return () => {
      window.removeEventListener('storage', handleStorageChange);
      clearInterval(interval);
    };
  }, []);

  const IMPORT_MUTATION = `
    mutation ImportUniversalJson($input: UniversalImportInput!) {
      importUniversalJson(input: $input) {
        success
        accountsImported
        vatRatesImported
        counterpartsImported
        documentsImported
        journalEntriesImported
        errors
        warnings
      }
    }
  `;

  const IMPORT_CHART_MUTATION = `
    mutation ImportChartOfAccounts($companyId: Int!, $jsonData: String!) {
      importChartOfAccounts(companyId: $companyId, jsonData: $jsonData) {
        success
        accountsImported
        vatRatesImported
        counterpartsImported
        documentsImported
        journalEntriesImported
        errors
        warnings
      }
    }
  `;

  const features = [
    'JSON формат съгласно универсална схема v2.0',
    'Импорт на сметкоплан',
    'Импорт на ДДС ставки и кодове',
    'Импорт на контрагенти',
    'Валидация и проверка за дубликати',
    'Детайлен отчет за импорта'
  ];

  const handleDragOver = useCallback((e) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e) => {
    e.preventDefault();
    setDragOver(false);
  }, []);

  const handleDrop = useCallback((e) => {
    e.preventDefault();
    setDragOver(false);

    const files = Array.from(e.dataTransfer.files);
    const jsonFiles = files.filter(file => file.name.toLowerCase().endsWith('.json'));

    if (jsonFiles.length > 0) {
      setUploadedFiles(prev => [...prev, ...jsonFiles.map(file => ({
        name: file.name,
        size: file.size,
        status: 'pending',
        file: file
      }))]);
    } else {
      alert('Моля качете само JSON файлове');
    }
  }, []);

  const handleFileSelect = useCallback((e) => {
    const files = Array.from(e.target.files);
    setUploadedFiles(prev => [...prev, ...files.map(file => ({
      name: file.name,
      size: file.size,
      status: 'pending',
      file: file
    }))]);
  }, []);

  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const removeFile = (index) => {
    setUploadedFiles(prev => prev.filter((_, i) => i !== index));
  };

  const startImport = async () => {
    if (uploadedFiles.length === 0) {
      alert('Моля качете файл за импорт');
      return;
    }

    setImporting(true);
    setImportResults(null);
    setUploadedFiles(prev => prev.map(file => ({ ...file, status: 'processing' })));

    try {
      // Process each file sequentially
      const results = [];

      for (const fileInfo of uploadedFiles) {
        try {
          const fileContent = await fileInfo.file.text();

          // Validate JSON and detect format
          let parsedData;
          try {
            parsedData = JSON.parse(fileContent);
          } catch (parseErr) {
            throw new Error(`Невалиден JSON формат: ${parseErr.message}`);
          }

          // Check if it's an array (chart of accounts only) or full universal format
          let result;
          if (Array.isArray(parsedData)) {
            // It's a chart of accounts array
            result = await graphqlRequest(IMPORT_CHART_MUTATION, {
              companyId: currentCompanyId,
              jsonData: fileContent
            });

            results.push({
              fileName: fileInfo.name,
              ...result.importChartOfAccounts
            });

            // Update file status
            setUploadedFiles(prev => prev.map(f =>
              f.name === fileInfo.name
                ? { ...f, status: result.importChartOfAccounts.success ? 'completed' : 'error' }
                : f
            ));
          } else {
            // It's the full universal format
            result = await graphqlRequest(IMPORT_MUTATION, {
              input: {
                companyId: currentCompanyId,
                jsonData: fileContent
              }
            });

            results.push({
              fileName: fileInfo.name,
              ...result.importUniversalJson
            });

            // Update file status
            setUploadedFiles(prev => prev.map(f =>
              f.name === fileInfo.name
                ? { ...f, status: result.importUniversalJson.success ? 'completed' : 'error' }
                : f
            ));
          }

        } catch (error) {
          results.push({
            fileName: fileInfo.name,
            success: false,
            errors: [error.message],
            warnings: [],
            accountsImported: 0,
            vatRatesImported: 0,
            counterpartsImported: 0,
            documentsImported: 0,
            journalEntriesImported: 0
          });

          setUploadedFiles(prev => prev.map(f =>
            f.name === fileInfo.name ? { ...f, status: 'error' } : f
          ));
        }
      }

      setImportResults(results);

    } catch (error) {
      alert(`Грешка при импорт: ${error.message}`);
    } finally {
      setImporting(false);
    }
  };

  const downloadExampleJson = () => {
    const example = {
      version: "2.0",
      exportDate: new Date().toISOString(),
      companyInfo: {
        name: "Примерна фирма ООД",
        eik: "123456789",
        vatNumber: "BG123456789",
        address: "ул. Примерна 1",
        city: "София",
        country: "BG"
      },
      chartOfAccounts: [
        {
          code: "101",
          name: "Каса в BGN",
          accountType: "ASSET",
          isAnalytic: false,
          requiresCounterpart: false,
          isActive: true,
          description: "Каса в български лева"
        },
        {
          code: "411",
          name: "Клиенти",
          accountType: "ASSET",
          isAnalytic: true,
          requiresCounterpart: true,
          isActive: true,
          description: "Вземания от клиенти"
        }
      ],
      vatRates: [
        {
          code: "02",
          name: "Доставки в страната облагаеми с 20%",
          rate: 20,
          registerType: 2,
          isDeductible: true,
          deductionPercent: 100,
          accountCode: "4531",
          isActive: true,
          description: "Стандартна ДДС ставка за продажби"
        },
        {
          code: "11",
          name: "Покупки с 20% ДДС",
          rate: 20,
          registerType: 1,
          isDeductible: true,
          deductionPercent: 100,
          accountCode: "4532",
          isActive: true,
          description: "Стандартна ДДС ставка за покупки"
        }
      ],
      counterparts: [
        {
          name: "Примерен доставчик ЕООД",
          eik: "987654321",
          vatNumber: "BG987654321",
          address: "бул. Бизнес 10",
          city: "Пловдив",
          country: "BG",
          phone: "+359888123456",
          email: "office@example.com",
          contactPerson: "Иван Иванов",
          isVatRegistered: true,
          isActive: true
        }
      ],
      importSettings: {
        validateBeforeImport: true,
        autoCreateCounterparts: true,
        autoCreateAccounts: false,
        skipDuplicates: true,
        skipExistingAccounts: true,
        updateExisting: false,
        importOnlyActive: true,
        deleteExistingAccounts: false
      }
    };

    const blob = new Blob([JSON.stringify(example, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'universal-import-example.json';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const downloadSchemaDoc = () => {
    const schemaUrl = '/z-import-format/universal-schema.json';
    const a = document.createElement('a');
    a.href = schemaUrl;
    a.download = 'universal-schema.json';
    a.click();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start space-x-4">
        <div className="text-4xl">📊</div>
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-gray-900">
            Универсален импорт - JSON формат v2.0
          </h3>
          <p className="text-gray-600 mb-1">
            Импорт на сметкоплан, ДДС ставки, контрагенти и други данни от JSON файл
          </p>
          <div className="mb-3 px-3 py-2 bg-blue-100 border border-blue-300 rounded-md inline-block">
            <span className="text-sm font-medium text-blue-900">
              Импортът ще се направи във фирма с ID: {currentCompanyId}
            </span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            {features.map((feature, index) => (
              <div key={index} className="flex items-center space-x-2">
                <span className="w-2 h-2 bg-green-400 rounded-full"></span>
                <span className="text-sm text-gray-700">{feature}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Download Templates */}
      <div className="bg-blue-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-blue-500 text-xl">📥</div>
          <div className="flex-1">
            <div className="font-medium text-blue-900 mb-2">Изтегляне на шаблони и документация:</div>
            <div className="flex flex-wrap gap-2">
              <button
                onClick={downloadExampleJson}
                className="px-4 py-2 text-sm font-medium text-blue-700 bg-white border border-blue-300 rounded-md hover:bg-blue-50 transition-colors"
              >
                📄 Примерен JSON файл
              </button>
              <button
                onClick={downloadSchemaDoc}
                className="px-4 py-2 text-sm font-medium text-blue-700 bg-white border border-blue-300 rounded-md hover:bg-blue-50 transition-colors"
              >
                📋 JSON Schema документация
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Upload Area */}
      <div
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          dragOver
            ? 'border-green-500 bg-green-50'
            : 'border-gray-300 hover:border-gray-400'
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="space-y-4">
          <div className="text-4xl">📊</div>
          <div>
            <p className="text-lg font-medium text-gray-900">
              Пуснете JSON файл за импорт тук или
            </p>
            <label className="cursor-pointer">
              <span className="text-green-600 hover:text-green-700 font-medium">
                изберете файл
              </span>
              <input
                type="file"
                accept=".json"
                onChange={handleFileSelect}
                className="hidden"
                multiple
              />
            </label>
          </div>
          <p className="text-sm text-gray-500">
            Поддържан формат: .json (Universal Schema v2.0)
          </p>
        </div>
      </div>

      {/* Uploaded Files */}
      {uploadedFiles.length > 0 && (
        <div className="space-y-4">
          <h4 className="text-sm font-medium text-gray-900">
            Качени файлове ({uploadedFiles.length})
          </h4>
          <div className="space-y-2">
            {uploadedFiles.map((file, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 bg-gray-50 rounded-lg border"
              >
                <div className="flex items-center space-x-3">
                  <div className="text-xl">📋</div>
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      {file.name}
                    </p>
                    <p className="text-xs text-gray-500">
                      {formatFileSize(file.size)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <span
                    className={`px-2 py-1 text-xs rounded-full font-medium ${
                      file.status === 'pending'
                        ? 'bg-yellow-100 text-yellow-800'
                        : file.status === 'processing'
                        ? 'bg-blue-100 text-blue-800 animate-pulse'
                        : file.status === 'completed'
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}
                  >
                    {file.status === 'pending' && 'Чака'}
                    {file.status === 'processing' && 'Импортира се...'}
                    {file.status === 'completed' && '✓ Готов'}
                    {file.status === 'error' && '✗ Грешка'}
                  </span>
                  <button
                    onClick={() => removeFile(index)}
                    disabled={importing}
                    className="p-1 text-gray-400 hover:text-red-600 transition-colors disabled:opacity-50"
                  >
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
            ))}
          </div>

          <div className="flex justify-end space-x-3">
            <button
              onClick={() => {
                setUploadedFiles([]);
                setImportResults(null);
              }}
              disabled={importing}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Изчисти всички
            </button>
            <button
              onClick={startImport}
              disabled={importing}
              className="px-4 py-2 text-sm font-medium text-white bg-green-600 border border-transparent rounded-md hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {importing ? 'Импортира се...' : 'Започни импорт'}
            </button>
          </div>
        </div>
      )}

      {/* Import Results */}
      {importResults && (
        <div className="space-y-4">
          <h4 className="text-sm font-medium text-gray-900">Резултати от импорта:</h4>
          {importResults.map((result, index) => (
            <div key={index} className={`rounded-lg p-4 border-2 ${
              result.success ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'
            }`}>
              <div className="flex items-start space-x-3">
                <div className="text-2xl">
                  {result.success ? '✅' : '❌'}
                </div>
                <div className="flex-1">
                  <p className="font-medium text-gray-900 mb-2">
                    {result.fileName}
                  </p>

                  {result.success && (
                    <div className="space-y-1 text-sm">
                      <p className="text-green-700">
                        ✓ Сметки: {result.accountsImported}
                      </p>
                      <p className="text-green-700">
                        ✓ ДДС ставки: {result.vatRatesImported}
                      </p>
                      <p className="text-green-700">
                        ✓ Контрагенти: {result.counterpartsImported}
                      </p>
                      {result.documentsImported > 0 && (
                        <p className="text-green-700">
                          ✓ Документи: {result.documentsImported}
                        </p>
                      )}
                      {result.journalEntriesImported > 0 && (
                        <p className="text-green-700">
                          ✓ Счетоводни записи: {result.journalEntriesImported}
                        </p>
                      )}
                    </div>
                  )}

                  {result.warnings && result.warnings.length > 0 && (
                    <div className="mt-2">
                      <p className="font-medium text-yellow-800 text-sm mb-1">Предупреждения:</p>
                      <ul className="text-xs text-yellow-700 space-y-1">
                        {result.warnings.map((warning, i) => (
                          <li key={i}>⚠ {warning}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  {result.errors && result.errors.length > 0 && (
                    <div className="mt-2">
                      <p className="font-medium text-red-800 text-sm mb-1">Грешки:</p>
                      <ul className="text-xs text-red-700 space-y-1">
                        {result.errors.map((error, i) => (
                          <li key={i}>✗ {error}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Format Info */}
      <div className="bg-green-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-green-500 text-xl">💡</div>
          <div className="text-sm text-green-800">
            <div className="font-medium mb-2">Формат на универсалния импорт (v2.0):</div>
            <ul className="space-y-1">
              <li>• <strong>JSON файл</strong> съдържащ данни за компанията, сметкоплан, ДДС ставки и контрагенти</li>
              <li>• <strong>chartOfAccounts</strong> - масив със сметки (код, име, тип, родител)</li>
              <li>• <strong>vatRates</strong> - масив с ДДС ставки (код, име, процент, регистър)</li>
              <li>• <strong>counterparts</strong> - масив с контрагенти (име, БУЛСТАТ, ДДС, адрес)</li>
              <li>• <strong>Валидация</strong> - автоматична проверка за дубликати и грешки</li>
              <li>• <strong>Настройки</strong> - контрол над създаване/обновяване на записи</li>
              <li>• <strong>deleteExistingAccounts</strong> - изтриване на стари сметки преди импорт (за нова фирма)</li>
            </ul>
            <div className="mt-3 text-xs">
              Изтеглете примерния файл за да видите точния формат и структура!
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
