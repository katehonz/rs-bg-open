import { useState, useEffect, useMemo } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

const BANK_PROFILES_QUERY = `
  query ActiveBankProfiles($companyId: Int!, $activeOnly: Boolean) {
    bankProfiles(companyId: $companyId, activeOnly: $activeOnly) {
      id
      name
      accountId
      bufferAccountId
      currencyCode
      importFormat
      isActive
    }
  }
`;

const ACCOUNTS_QUERY = `
  query AccountOptions($companyId: Int!, $isAnalytical: Boolean) {
    accounts(companyId: $companyId, isAnalytical: $isAnalytical) {
      id
      code
      name
      isAnalytical
    }
  }
`;

const IMPORT_BANK_STATEMENT = `
  mutation ImportBankStatement($input: ImportBankStatementInput!) {
    importBankStatement(input: $input) {
      bankImport {
        id
        fileName
        importFormat
        status
        importedAt
        transactionsCount
        totalDebit
        totalCredit
      }
      transactions
      totalDebit
      totalCredit
      journalEntryIds
    }
  }
`;

const importFormatConfig = {
  UNICREDIT_MT940: {
    label: 'UniCredit MT940',
    extensions: ['.mt940', '.txt'],
    icon: '🏦',
    description: 'SWIFT MT940 извлечения от UniCredit в текстов формат.',
  },
  WISE_CAMT053: {
    label: 'Wise CAMT.053 XML',
    extensions: ['.xml'],
    icon: '🌐',
    description: 'ISO 20022 CAMT.053 извлечения, експортирани от Wise.',
  },
  REVOLUT_CAMT053: {
    label: 'Revolut CAMT.053 XML',
    extensions: ['.xml'],
    icon: '💳',
    description: 'ISO 20022 CAMT.053 извлечения от Revolut Business.',
  },
  PAYSERA_CAMT053: {
    label: 'Paysera CAMT.053 XML',
    extensions: ['.xml'],
    icon: '💼',
    description: 'ISO 20022 CAMT.053 извлечения, генерирани през Paysera.',
  },
  POSTBANK_XML: {
    label: 'Postbank XML',
    extensions: ['.xml'],
    icon: '🏛️',
    description: 'XML извлечения от системата на Пощенска банка.',
  },
  OBB_XML: {
    label: 'OBB XML',
    extensions: ['.xml'],
    icon: '🏦',
    description: 'XML извлечения от Обединена Българска Банка (ОББ).',
  },
  CCB_CSV: {
    label: 'ЦКБ CSV',
    extensions: ['.csv'],
    icon: '🏦',
    description: 'CSV извлечения от ЦКБ в Windows-1251 формат.',
  },
};

const baseFeatures = [
  'Автоматично създаване на журнални записи',
  'Дебит/Кредит по банковата и буферната сметка',
  'Запазване на импорта в историята с референция',
];

const formatSpecificFeatures = {
  UNICREDIT_MT940: ['Парсване на SWIFT тагове :61: и :86:', 'Разпознаване на дебит/кредит по MT940'],
  WISE_CAMT053: ['Импорт на CAMT.053 statement entries', 'Поддръжка на много валути и IBAN-и'],
  REVOLUT_CAMT053: ['Разпознаване на картови плащания и fee операции', 'CAMT.053 с допълнителна информация за карти'],
  PAYSERA_CAMT053: ['Обработка на Paysera inbound/outbound потоци', 'Съхранение на bordero номера'],
  POSTBANK_XML: ['Импорт на XML, предоставени от Postbank API', 'Разпознаване на контрагенти и бордеро номер'],
  OBB_XML: ['Импорт на XML от ОББ', 'Разпознаване на картови плащания, бордеро и референции'],
  CCB_CSV: ['Импорт на CSV извлечения от ЦКБ', 'Автоматично разпознаване по полетата Приход/Разход'],
};

const defaultFormatConfig = {
  label: 'Банков файл',
  extensions: ['.xml', '.mt940', '.txt', '.csv'],
  icon: '📄',
  description: 'Поддържани са MT940, CAMT.053 и специфични банкови XML формати.',
};

function formatBytes(bytes) {
  if (!bytes) {
    return '0 B';
  }
  const units = ['B', 'KB', 'MB', 'GB'];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / Math.pow(1024, exponent);
  return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

function mapAccountLabel(accountsMap, accountId) {
  if (!accountId) {
    return '—';
  }
  const account = accountsMap.get(accountId);
  return account ? `${account.code} · ${account.name}` : `ID ${accountId}`;
}

function mergeFeatures(importFormat) {
  return [...baseFeatures, ...(formatSpecificFeatures[importFormat] ?? [])];
}

function fileToBase64(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result;
      if (typeof result !== 'string') {
        reject(new Error('Неуспешно конвертиране на файла в base64.'));
        return;
      }
      const [, base64] = result.split(',');
      resolve(base64 || result);
    };
    reader.onerror = () => reject(new Error('Грешка при прочитане на файла.'));
    reader.readAsDataURL(file);
  });
}

export default function BankImport() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [bankProfiles, setBankProfiles] = useState([]);
  const [accounts, setAccounts] = useState([]);
  const [selectedBankId, setSelectedBankId] = useState(null);
  const [dragOver, setDragOver] = useState(false);
  const [uploadedFiles, setUploadedFiles] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [alert, setAlert] = useState(null);
  const [isImporting, setIsImporting] = useState(false);

  useEffect(() => {
    let mounted = true;

    const loadData = async () => {
      setLoading(true);
      try {
        const [profilesData, accountsData] = await Promise.all([
          graphqlRequest(BANK_PROFILES_QUERY, { companyId, activeOnly: true }),
          graphqlRequest(ACCOUNTS_QUERY, { companyId, isAnalytical: true }),
        ]);

        if (!mounted) {
          return;
        }

        const profiles = profilesData?.bankProfiles ?? [];
        setBankProfiles(profiles);
        setAccounts(accountsData?.accounts ?? []);
        setSelectedBankId((prev) => {
          if (prev && profiles.some((profile) => profile.id === prev)) {
            return prev;
          }
          return profiles.length > 0 ? profiles[0].id : null;
        });
        setError(null);
      } catch (err) {
        console.error('Грешка при зареждане на банкови профили:', err);
        if (mounted) {
          setError(err.message);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    loadData();

    return () => {
      mounted = false;
    };
  }, [companyId]);

  const accountsMap = useMemo(() => {
    const map = new Map();
    accounts.forEach((account) => {
      map.set(account.id, account);
    });
    return map;
  }, [accounts]);

  const selectedProfile = useMemo(
    () => bankProfiles.find((profile) => profile.id === selectedBankId) ?? null,
    [bankProfiles, selectedBankId],
  );

  const formatConfig = selectedProfile
    ? importFormatConfig[selectedProfile.importFormat] ?? defaultFormatConfig
    : defaultFormatConfig;

  const features = mergeFeatures(selectedProfile?.importFormat);

  const allowedExtensions = formatConfig.extensions.map((ext) => ext.toLowerCase());

  const handleFilesAdded = (fileList) => {
    if (!selectedProfile) {
      setAlert({ type: 'warning', message: 'Моля изберете банка преди да качите файл.' });
      return;
    }

    const filesArray = Array.from(fileList);
    const accepted = [];
    const rejected = [];

    filesArray.forEach((file) => {
      const lowerName = file.name.toLowerCase();
      const isValid = allowedExtensions.some((ext) => lowerName.endsWith(ext));
      if (isValid) {
        accepted.push(file);
      } else {
        rejected.push(file.name);
      }
    });

    if (rejected.length > 0) {
      setAlert({
        type: 'warning',
        message: `Следните файлове са с неподдържан формат: ${rejected.join(', ')}`,
      });
    }

    if (accepted.length > 0) {
      setUploadedFiles((prev) => [
        ...prev,
        ...accepted.map((file) => ({
          name: file.name,
          size: file.size,
          file,
          status: 'pending',
          error: null,
          summary: null,
        })),
      ]);
    }
  };

  const handleDrop = (event) => {
    event.preventDefault();
    event.stopPropagation();
    setDragOver(false);
    handleFilesAdded(event.dataTransfer.files);
  };

  const handleDragOver = (event) => {
    event.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = (event) => {
    event.preventDefault();
    setDragOver(false);
  };

  const removeFile = (index) => {
    setUploadedFiles((prev) => prev.filter((_, idx) => idx !== index));
  };

  const clearFiles = () => {
    setUploadedFiles([]);
  };

  const startImport = async () => {
    if (!selectedProfile) {
      setAlert({ type: 'warning', message: 'Изберете банкова сметка за импортиране.' });
      return;
    }

    if (uploadedFiles.length === 0) {
      setAlert({ type: 'warning', message: 'Добавете поне един файл за импорт.' });
      return;
    }

    setIsImporting(true);
    setAlert(null);

    const filesToProcess = [...uploadedFiles];

    for (let i = 0; i < filesToProcess.length; i += 1) {
      const entry = filesToProcess[i];
      setUploadedFiles((prev) => prev.map((file, idx) => (
        idx === i ? { ...file, status: 'processing', error: null } : file
      )));

      try {
        const base64content = await fileToBase64(entry.file);
        const response = await graphqlRequest(IMPORT_BANK_STATEMENT, {
          input: {
            bankProfileId: selectedProfile.id,
            companyId,
            fileName: entry.name,
            fileBase64: base64content,
          },
        });

        const result = response?.importBankStatement;
        if (!result) {
          throw new Error('Сървърът не върна резултат от импорта.');
        }

        setUploadedFiles((prev) => prev.map((file, idx) => (
          idx === i
            ? {
                ...file,
                status: 'completed',
                summary: result,
                error: null,
              }
            : file
        )));
      } catch (err) {
        console.error('Грешка при импорт на банков файл:', err);
        setUploadedFiles((prev) => prev.map((file, idx) => (
          idx === i
            ? {
                ...file,
                status: 'error',
                error: err.message,
              }
            : file
        )));
        setAlert({ type: 'error', message: err.message });
      }
    }

    setIsImporting(false);
    setAlert((prev) => prev ?? {
      type: 'success',
      message: 'Импортът на банковите извлечения приключи.',
    });
  };

  if (loading) {
    return (
      <div className="bg-white border border-gray-200 rounded-lg p-6 text-center text-gray-500">
        Зареждане на банковите профили...
      </div>
    );
  }

  if (!selectedProfile) {
    return (
      <div className="space-y-4">
        {error && (
          <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
            {error}
          </div>
        )}
        <div className="bg-yellow-50 border border-yellow-200 text-yellow-800 px-4 py-3 rounded">
          Няма активни банкови профили. Добавете банка от меню "Банки", за да започнете импорт.
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Какво е необходимо?</h3>
          <ul className="list-disc list-inside text-sm text-gray-600 space-y-1">
            <li>Дефинирайте банкова сметка, валута и формат на импорта в раздел "Банки".</li>
            <li>Задайте аналитична сметка (503...) и буферна сметка за разпределение на плащанията.</li>
            <li>След добавяне на профил се върнете тук, за да импортирате банковия файл.</li>
          </ul>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}

      <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between lg:space-x-6 space-y-4 lg:space-y-0">
        <div className="flex items-start space-x-4">
          <div className="text-4xl">{formatConfig.icon}</div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900">Импорт на банкови извлечения</h3>
            <p className="text-gray-600 mb-3">
              Файлът ще бъде обработен според настройките на избраната банка. Получените журнални записи ще бъдат създадени в чернова.
            </p>
            <div className="flex items-center space-x-3">
              <label className="text-sm font-medium text-gray-700">Банков профил:</label>
              <select
                value={selectedBankId ?? ''}
                onChange={(event) => setSelectedBankId(Number(event.target.value))}
                className="border border-gray-300 rounded-md px-3 py-1.5 text-sm focus:border-blue-500 focus:ring-blue-500"
              >
                {bankProfiles.map((profile) => (
                  <option key={profile.id} value={profile.id}>
                    {profile.name} ({profile.currencyCode})
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4 text-sm text-gray-600 w-full max-w-md">
          <div className="font-semibold text-gray-900 mb-2">Текуща конфигурация</div>
          <dl className="space-y-1">
            <div className="flex justify-between">
              <dt className="text-gray-500">Банкова сметка</dt>
              <dd className="text-gray-900">{mapAccountLabel(accountsMap, selectedProfile.accountId)}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Буферна сметка</dt>
              <dd className="text-gray-900">{mapAccountLabel(accountsMap, selectedProfile.bufferAccountId)}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Валута</dt>
              <dd className="text-gray-900">{selectedProfile.currencyCode}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Формат</dt>
              <dd className="text-gray-900">{formatConfig.label}</dd>
            </div>
          </dl>
        </div>
      </div>

      {alert && (
        <div
          className={`${
            alert.type === 'success'
              ? 'bg-green-50 border-green-200 text-green-700'
              : alert.type === 'warning'
              ? 'bg-yellow-50 border-yellow-200 text-yellow-700'
              : 'bg-red-50 border-red-200 text-red-700'
          } border px-4 py-3 rounded`}
        >
          {alert.message}
        </div>
      )}

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <div className="xl:col-span-2 space-y-6">
          <div
            className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
              dragOver ? 'border-blue-500 bg-blue-50' : 'border-gray-300 hover:border-gray-400'
            }`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <div className="space-y-4">
              <div className="text-4xl">{formatConfig.icon}</div>
              <div>
                <p className="text-lg font-medium text-gray-900">
                  Пуснете файла ({formatConfig.label}) тук или
                </p>
                <label className="cursor-pointer">
                  <span className="text-blue-600 hover:text-blue-700 font-medium">
                    изберете файл
                  </span>
                  <input
                    type="file"
                    multiple
                    accept={allowedExtensions.join(',')}
                    onChange={(event) => {
                      handleFilesAdded(event.target.files);
                      event.target.value = '';
                    }}
                    className="hidden"
                  />
                </label>
              </div>
              <p className="text-sm text-gray-500">
                Поддържани разширения: {allowedExtensions.join(', ')}
              </p>
              <p className="text-xs text-gray-400">Препоръчително е файловете да са до 20MB всеки.</p>
            </div>
          </div>

          {uploadedFiles.length > 0 && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h4 className="text-sm font-medium text-gray-900">
                  Качени файлове ({uploadedFiles.length})
                </h4>
                <div className="flex items-center space-x-3">
                  <button
                    onClick={clearFiles}
                    className="px-3 py-1.5 text-xs font-medium text-gray-600 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                  >
                    Изчисти всички
                  </button>
                  <button
                    onClick={startImport}
                    disabled={isImporting}
                    className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:bg-gray-400"
                  >
                    {isImporting ? 'Импортиране...' : 'Стартирай импорта'}
                  </button>
                </div>
              </div>

              <div className="space-y-3">
                {uploadedFiles.map((file, index) => (
                  <div key={`${file.name}-${index}`} className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="flex flex-col md:flex-row md:items-center md:justify-between space-y-3 md:space-y-0">
                      <div className="flex items-center space-x-3">
                        <div className="text-2xl">{formatConfig.icon}</div>
                        <div>
                          <div className="text-sm font-medium text-gray-900">{file.name}</div>
                          <div className="text-xs text-gray-500">{formatBytes(file.size)}</div>
                        </div>
                      </div>
                      <div className="flex items-center space-x-3">
                        <span
                          className={`px-2 py-1 text-xs rounded-full font-medium ${
                            file.status === 'pending'
                              ? 'bg-yellow-100 text-yellow-800'
                              : file.status === 'processing'
                              ? 'bg-blue-100 text-blue-800'
                              : file.status === 'completed'
                              ? 'bg-green-100 text-green-800'
                              : 'bg-red-100 text-red-800'
                          }`}
                        >
                          {file.status === 'pending' && 'Чака'}
                          {file.status === 'processing' && 'Обработва се'}
                          {file.status === 'completed' && 'Готов'}
                          {file.status === 'error' && 'Грешка'}
                        </span>
                        <button
                          onClick={() => removeFile(index)}
                          className="p-1 text-gray-400 hover:text-red-600 transition-colors"
                          disabled={file.status === 'processing'}
                        >
                          ✕
                        </button>
                      </div>
                    </div>
                    {file.error && (
                      <div className="mt-2 text-sm text-red-600">
                        {file.error}
                      </div>
                    )}
                    {file.summary && (
                      <div className="mt-3 bg-green-50 border border-green-100 rounded-md p-3 text-sm text-green-800">
                        <div className="font-medium text-green-900 mb-1">Импортът е успешен</div>
                        <div>Журнални записи: {file.summary.transactions}</div>
                        <div>Сума Дт: {file.summary.totalDebit}</div>
                        <div>Сума Кт: {file.summary.totalCredit}</div>
                        {file.summary.journalEntryIds?.length > 0 && (
                          <div className="mt-1 text-xs text-green-700">
                            ID на журнални записи: {file.summary.journalEntryIds.join(', ')}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="space-y-4">
          <div className="bg-white border border-gray-200 rounded-lg p-5">
            <h4 className="text-sm font-semibold text-gray-900 mb-3">Как работи импортът</h4>
            <p className="text-sm text-gray-600 mb-3">
              {formatConfig.description}
            </p>
            <ul className="space-y-2 text-sm text-gray-600">
              {features.map((feature, idx) => (
                <li key={idx} className="flex items-start space-x-2">
                  <span className="text-green-500 text-base">•</span>
                  <span>{feature}</span>
                </li>
              ))}
            </ul>
          </div>

          <div className="bg-blue-50 border border-blue-100 rounded-lg p-5 text-sm text-blue-800">
            <div className="font-medium text-blue-900 mb-2">Напомняне</div>
            <ul className="space-y-2">
              <li>След импорта журналните записи са в чернова за допълнителен преглед.</li>
              <li>Буферната сметка служи за последващо разпределение по контрагенти или фактури.</li>
              <li>Документният номер може да е транзакцията от файла или автоматично генериран.</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
