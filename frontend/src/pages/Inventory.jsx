import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

export default function Inventory() {
  const [activeTab, setActiveTab] = useState('turnover');
  const [startDate, setStartDate] = useState('2025-01-01');
  const [endDate, setEndDate] = useState(new Date().toISOString().split('T')[0]);
  const [turnoverData, setTurnoverData] = useState([]);
  const [loading, setLoading] = useState(false);

  // Corrections tab state
  const [accounts, setAccounts] = useState([]);
  const [selectedAccount, setSelectedAccount] = useState(null);
  const [checkDate, setCheckDate] = useState(new Date().toISOString().split('T')[0]);
  const [corrections, setCorrections] = useState([]);
  const [correctionsLoading, setCorrectionsLoading] = useState(false);
  const [accountsLoading, setAccountsLoading] = useState(false);

  const QUANTITY_TURNOVER_QUERY = `
    query GetQuantityTurnover($companyId: Int!, $fromDate: NaiveDate!, $toDate: NaiveDate!) {
      getQuantityTurnover(companyId: $companyId, fromDate: $fromDate, toDate: $toDate) {
        accountId
        accountCode
        accountName
        openingQuantity
        openingAmount
        receiptQuantity
        receiptAmount
        issueQuantity
        issueAmount
        closingQuantity
        closingAmount
      }
    }
  `;

  const ACCOUNTS_QUERY = `
    query GetMaterialAccounts($companyId: Int!) {
      accountHierarchy(companyId: $companyId) {
        id
        code
        name
        accountClass
        supportsQuantities
        isActive
      }
    }
  `;

  const CORRECTIONS_QUERY = `
    query CheckCorrections($companyId: Int!, $accountId: Int!, $newEntryDate: NaiveDate!) {
      checkRetroactiveCorrections(
        companyId: $companyId,
        accountId: $accountId,
        newEntryDate: $newEntryDate
      ) {
        movementId
        movementDate
        materialAccountId
        expenseAccountId
        quantity
        oldAverageCost
        newAverageCost
        correctionAmount
        description
      }
    }
  `;

  const ACCOUNT_NAME_QUERY = `
    query GetAccountName($companyId: Int!, $accountId: Int!) {
      accountHierarchy(companyId: $companyId) {
        id
        code
        name
      }
    }
  `;

  const EXPORT_QUANTITY_TURNOVER_MUTATION = `
    mutation ExportQuantityTurnover(
      $companyId: Int!,
      $fromDate: NaiveDate!,
      $toDate: NaiveDate!,
      $format: String!
    ) {
      exportQuantityTurnover(
        companyId: $companyId,
        fromDate: $fromDate,
        toDate: $toDate,
        format: $format
      ) {
        format
        content
        filename
        mimeType
      }
    }
  `;

  useEffect(() => {
    if (activeTab === 'corrections') {
      loadMaterialAccounts();
    }
  }, [activeTab]);

  const handleLoadTurnover = async () => {
    try {
      setLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const response = await graphqlRequest(QUANTITY_TURNOVER_QUERY, {
        companyId,
        fromDate: startDate,
        toDate: endDate
      });

      setTurnoverData(response.getQuantityTurnover || []);
    } catch (err) {
      alert('Грешка при зареждане: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      style: 'currency',
      currency: 'BGN',
      minimumFractionDigits: 2
    }).format(amount || 0);
  };

  const formatQuantity = (qty) => {
    return new Intl.NumberFormat('bg-BG', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 6
    }).format(qty || 0);
  };

  const loadMaterialAccounts = async () => {
    try {
      setAccountsLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const data = await graphqlRequest(ACCOUNTS_QUERY, { companyId });

      const materialAccounts = (data.accountHierarchy || []).filter(acc =>
        acc.isActive && (
          acc.accountClass === 3 ||
          acc.supportsQuantities ||
          (acc.code && acc.code.startsWith('3'))
        )
      );

      setAccounts(materialAccounts);
    } catch (err) {
      alert('Грешка при зареждане на сметките: ' + err.message);
    } finally {
      setAccountsLoading(false);
    }
  };

  const checkCorrections = async () => {
    if (!selectedAccount) {
      alert('Моля изберете материална сметка');
      return;
    }

    try {
      setCorrectionsLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const response = await graphqlRequest(CORRECTIONS_QUERY, {
        companyId,
        accountId: selectedAccount.id,
        newEntryDate: checkDate
      });

      const correctionsData = response.checkRetroactiveCorrections || [];

      if (correctionsData.length > 0) {
        const accountsData = await graphqlRequest(ACCOUNT_NAME_QUERY, { companyId });
        const accountsMap = {};
        (accountsData.accountHierarchy || []).forEach(acc => {
          accountsMap[acc.id] = acc;
        });

        const enrichedCorrections = correctionsData.map(corr => ({
          ...corr,
          materialAccountName: accountsMap[corr.materialAccountId]?.name || 'N/A',
          materialAccountCode: accountsMap[corr.materialAccountId]?.code || 'N/A',
          expenseAccountName: accountsMap[corr.expenseAccountId]?.name || 'N/A',
          expenseAccountCode: accountsMap[corr.expenseAccountId]?.code || 'N/A',
        }));

        setCorrections(enrichedCorrections);
      } else {
        setCorrections([]);
      }
    } catch (err) {
      alert('Грешка при проверка: ' + err.message);
    } finally {
      setCorrectionsLoading(false);
    }
  };

  const getTotalCorrection = () => {
    return corrections.reduce((sum, corr) => sum + parseFloat(corr.correctionAmount), 0);
  };

  const exportTurnover = async (format) => {
    try {
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const response = await graphqlRequest(EXPORT_QUANTITY_TURNOVER_MUTATION, {
        companyId,
        fromDate: startDate,
        toDate: endDate,
        format
      });

      const exportData = response.exportQuantityTurnover;

      // Decode base64 content
      const byteCharacters = atob(exportData.content);
      const byteNumbers = new Array(byteCharacters.length);
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i);
      }
      const byteArray = new Uint8Array(byteNumbers);

      // Create blob and download
      const blob = new Blob([byteArray], { type: exportData.mimeType });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.setAttribute('download', exportData.filename);
      document.body.appendChild(link);
      link.click();
      link.parentNode.removeChild(link);
    } catch (err) {
      alert('Грешка при експорт: ' + err.message);
    }
  };

  const handlePrint = () => {
    window.print();
  };

  return (
    <div className="container mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">Материални запаси</h1>

      {/* Tabs */}
      <div className="bg-white rounded-lg shadow mb-6">
        <div className="border-b border-gray-200">
          <nav className="flex">
            <button
              onClick={() => setActiveTab('turnover')}
              className={`px-6 py-3 text-sm font-medium border-b-2 ${
                activeTab === 'turnover'
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              📊 Количествена оборотна ведомост
            </button>
            <button
              onClick={() => setActiveTab('corrections')}
              className={`px-6 py-3 text-sm font-medium border-b-2 ${
                activeTab === 'corrections'
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              ⚠️ Корекции на СПЦ
            </button>
          </nav>
        </div>
      </div>

      {/* Turnover Tab */}
      {activeTab === 'turnover' && (
        <>
          {/* Period Selection */}
          <div className="bg-white rounded-lg shadow p-6 mb-6">
            <h2 className="text-xl font-semibold mb-4">Количествена оборотна ведомост</h2>

        <div className="flex items-end gap-4 mb-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              От дата
            </label>
            <input
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              До дата
            </label>
            <input
              type="date"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md"
            />
          </div>

          <button
            onClick={handleLoadTurnover}
            disabled={loading}
            className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Зареждане...' : 'Покажи'}
          </button>
        </div>
      </div>

      {/* Turnover Table */}
      {turnoverData.length > 0 && (
        <div className="bg-white rounded-lg shadow overflow-hidden">
          {/* Export buttons */}
          <div className="px-6 py-4 bg-gray-50 border-b border-gray-200 flex justify-end gap-2">
            <button
              onClick={handlePrint}
              className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 flex items-center gap-2"
            >
              🖨️ Печат
            </button>
            <button
              onClick={() => exportTurnover('XLSX')}
              className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 flex items-center gap-2"
            >
              📊 Excel
            </button>
          </div>

          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th rowSpan="2" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-r">
                    Сметка
                  </th>
                  <th colSpan="2" className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider border-r">
                    Начално салдо
                  </th>
                  <th colSpan="2" className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider border-r">
                    Постъпване (Дт)
                  </th>
                  <th colSpan="2" className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider border-r">
                    Разход (Кт)
                  </th>
                  <th colSpan="2" className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Крайно салдо
                  </th>
                </tr>
                <tr>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase">Кол-во</th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase border-r">Стойност</th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase">Кол-во</th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase border-r">Стойност</th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase">Кол-во</th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase border-r">Стойност</th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase">Кол-во</th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase">Стойност</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {turnoverData.map((row, idx) => (
                  <tr key={idx} className="hover:bg-gray-50">
                    <td className="px-6 py-4 text-sm font-medium text-gray-900 border-r">
                      <div className="font-semibold">{row.accountCode}</div>
                      <div className="text-xs text-gray-600">{row.accountName}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                      {formatQuantity(row.openingQuantity)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 border-r">
                      {formatCurrency(row.openingAmount)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-green-600 font-medium">
                      {formatQuantity(row.receiptQuantity)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-green-600 font-medium border-r">
                      {formatCurrency(row.receiptAmount)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-red-600 font-medium">
                      {formatQuantity(row.issueQuantity)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-red-600 font-medium border-r">
                      {formatCurrency(row.issueAmount)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-semibold">
                      {formatQuantity(row.closingQuantity)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-semibold">
                      {formatCurrency(row.closingAmount)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {turnoverData.length === 0 && !loading && (
        <div className="bg-white rounded-lg shadow p-12 text-center text-gray-500">
          Няма данни за показване. Изберете период и натиснете "Покажи".
        </div>
      )}
        </>
      )}

      {/* Corrections Tab */}
      {activeTab === 'corrections' && (
        <>
          <div className="bg-white rounded-lg shadow p-6 mb-6">
            <h2 className="text-xl font-semibold mb-4">Проверка за корекции</h2>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Материална сметка
                </label>
                <select
                  value={selectedAccount?.id || ''}
                  onChange={(e) => {
                    const acc = accounts.find(a => a.id === parseInt(e.target.value));
                    setSelectedAccount(acc);
                  }}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md"
                  disabled={accountsLoading}
                >
                  <option value="">Изберете сметка...</option>
                  {accounts.map(acc => (
                    <option key={acc.id} value={acc.id}>
                      {acc.code} - {acc.name}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Проверка за записи след дата
                </label>
                <input
                  type="date"
                  value={checkDate}
                  onChange={(e) => setCheckDate(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md"
                />
              </div>

              <div className="flex items-end">
                <button
                  onClick={checkCorrections}
                  disabled={correctionsLoading || !selectedAccount}
                  className="w-full px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
                >
                  {correctionsLoading ? 'Проверка...' : 'Провери за корекции'}
                </button>
              </div>
            </div>

            <div className="bg-blue-50 border border-blue-200 rounded-md p-4 text-sm text-blue-800">
              <strong>Как работи:</strong> Изберете материална сметка и дата. Системата ще провери дали има изписвания
              СЛЕД тази дата, които трябва да бъдат коригирани заради промяна в средно претеглената цена.
            </div>
          </div>

          {/* Results */}
          {corrections.length > 0 && (
            <div className="bg-white rounded-lg shadow overflow-hidden mb-6">
              <div className="px-6 py-4 bg-yellow-50 border-b border-yellow-200">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-lg font-semibold text-yellow-900">
                      ⚠️ Открити {corrections.length} операции за корекция
                    </h3>
                    <p className="text-sm text-yellow-800 mt-1">
                      Следните изписвания са направени с невярна СПЦ и трябва да бъдат коригирани
                    </p>
                  </div>
                  <div className="text-right">
                    <div className="text-sm text-gray-600">Обща корекция:</div>
                    <div className={`text-2xl font-bold ${getTotalCorrection() >= 0 ? 'text-red-600' : 'text-green-600'}`}>
                      {formatCurrency(Math.abs(getTotalCorrection()))}
                      {getTotalCorrection() >= 0 ? ' ↑' : ' ↓'}
                    </div>
                  </div>
                </div>
              </div>

              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Дата
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Кореспонденция
                      </th>
                      <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Количество
                      </th>
                      <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Стара СПЦ
                      </th>
                      <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Нова СПЦ
                      </th>
                      <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Корекция
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Коригиращ запис
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {corrections.map((corr, idx) => {
                      const isPositive = parseFloat(corr.correctionAmount) >= 0;
                      return (
                        <tr key={idx} className="hover:bg-gray-50">
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {corr.movementDate}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-900">
                            <div className="font-medium">
                              Дт {corr.expenseAccountCode} / Кт {corr.materialAccountCode}
                            </div>
                            <div className="text-xs text-gray-600">
                              {corr.expenseAccountName}
                            </div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                            {formatQuantity(corr.quantity)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                            {formatCurrency(corr.oldAverageCost)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-right font-medium text-blue-600">
                            {formatCurrency(corr.newAverageCost)}
                          </td>
                          <td className={`px-6 py-4 whitespace-nowrap text-sm text-right font-bold ${
                            isPositive ? 'text-red-600' : 'text-green-600'
                          }`}>
                            {formatCurrency(Math.abs(corr.correctionAmount))}
                            {isPositive ? ' ↑' : ' ↓'}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-900">
                            <div className={`font-mono ${isPositive ? 'text-red-600' : 'text-green-600'}`}>
                              {isPositive ? (
                                <>
                                  <div>Дт {corr.expenseAccountCode}</div>
                                  <div>Кт {corr.materialAccountCode}</div>
                                  <div className="font-bold">{formatCurrency(corr.correctionAmount)}</div>
                                </>
                              ) : (
                                <>
                                  <div>Дт {corr.expenseAccountCode} (-)</div>
                                  <div>Кт {corr.materialAccountCode} (-)</div>
                                  <div className="font-bold">{formatCurrency(Math.abs(corr.correctionAmount))} сторно</div>
                                </>
                              )}
                            </div>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>

              <div className="px-6 py-4 bg-gray-50 border-t border-gray-200">
                <div className="flex items-center justify-between">
                  <div className="text-sm text-gray-600">
                    <strong>Забележка:</strong> Коригиращите записи трябва да се направят ръчно в дневника.
                    Използвайте показаната кореспонденция и суми.
                  </div>
                  <button
                    onClick={() => window.print()}
                    className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700"
                  >
                    🖨️ Печат
                  </button>
                </div>
              </div>
            </div>
          )}

          {corrections.length === 0 && !correctionsLoading && selectedAccount && (
            <div className="bg-white rounded-lg shadow p-12 text-center">
              <div className="text-6xl mb-4">✅</div>
              <div className="text-xl font-semibold text-green-600 mb-2">
                Няма нужда от корекции
              </div>
              <div className="text-gray-600">
                Всички изписвания след {checkDate} са с правилна СПЦ
              </div>
            </div>
          )}

          {!selectedAccount && !correctionsLoading && (
            <div className="bg-white rounded-lg shadow p-12 text-center text-gray-500">
              Изберете материална сметка и дата за проверка
            </div>
          )}
        </>
      )}
    </div>
  );
}
