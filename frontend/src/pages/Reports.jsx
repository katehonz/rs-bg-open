import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { graphqlRequest } from '../utils/graphqlClient';

export default function Reports() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  
  // Report parameters
  const [reportType, setReportType] = useState('turnover');
  const [startDate, setStartDate] = useState(new Date().toISOString().split('T')[0]);
  const [endDate, setEndDate] = useState(new Date().toISOString().split('T')[0]);
  const [accountId, setAccountId] = useState('');
  const [showZeroBalances, setShowZeroBalances] = useState(false);
  
  const [accounts, setAccounts] = useState([]);
  const [reportData, setReportData] = useState(null);

  const ACCOUNTS_QUERY = `
    query GetAccounts($companyId: Int!) {
      accountHierarchy(companyId: $companyId) {
        id
        code
        name
        isActive
      }
    }
  `;

  const TURNOVER_SHEET_QUERY = `
    query GetTurnoverSheet($input: TurnoverReportInput!) {
      turnoverSheet(input: $input) {
        companyName
        periodStart
        periodEnd
        entries {
          accountId
          accountCode
          accountName
          openingDebit
          openingCredit
          periodDebit
          periodCredit
          closingDebit
          closingCredit
        }
        totals {
          accountCode
          accountName
          openingDebit
          openingCredit
          periodDebit
          periodCredit
          closingDebit
          closingCredit
        }
        generatedAt
      }
    }
  `;

  const TRANSACTION_LOG_QUERY = `
    query GetTransactionLog($input: TransactionLogInput!) {
      transactionLog(input: $input) {
        companyName
        periodStart
        periodEnd
        entries {
          date
          entryNumber
          documentNumber
          description
          accountCode
          accountName
          debitAmount
          creditAmount
          counterpartName
        }
        generatedAt
      }
    }
  `;

  const GENERAL_LEDGER_QUERY = `
    query GetGeneralLedger($input: GeneralLedgerInput!) {
      generalLedger(input: $input) {
        companyName
        periodStart
        periodEnd
        accounts {
          accountId
          accountCode
          accountName
          openingBalance
          closingBalance
          totalDebits
          totalCredits
          entries {
            date
            entryNumber
            documentNumber
            description
            debitAmount
            creditAmount
            balance
            counterpartName
          }
        }
        generatedAt
      }
    }
  `;

  const CHRONOLOGICAL_REPORT_QUERY = `
    query GetChronologicalReport($input: ChronologicalReportInput!) {
      chronologicalReport(input: $input) {
        companyName
        periodStart
        periodEnd
        entries {
          date
          debitAccountCode
          debitAccountName
          creditAccountCode
          creditAccountName
          amount
          debitCurrencyAmount
          debitCurrencyCode
          creditCurrencyAmount
          creditCurrencyCode
          documentType
          documentDate
          description
        }
        totalAmount
        generatedAt
      }
    }
  `;

  const EXPORT_TURNOVER_MUTATION = `
    mutation ExportTurnoverSheet($input: TurnoverReportInput!, $format: String!) {
      exportTurnoverSheet(input: $input, format: $format) {
        format
        content
        filename
        mimeType
      }
    }
  `;

  const EXPORT_CHRONOLOGICAL_MUTATION = `
    mutation ExportChronologicalReport($input: ChronologicalReportInput!, $format: String!) {
      exportChronologicalReport(input: $input, format: $format) {
        format
        content
        filename
        mimeType
      }
    }
  `;

  useEffect(() => {
    loadAccounts();
  }, [companyId]);

  const loadAccounts = async () => {
    try {
      const data = await graphqlRequest(ACCOUNTS_QUERY, { companyId });
      const activeAccounts = (data.accountHierarchy || []).filter(a => a.isActive);
      setAccounts(activeAccounts);
    } catch (err) {
      setError('Грешка при зареждане на сметките: ' + err.message);
    }
  };

  const generateReport = async () => {
    setLoading(true);
    setError(null);

    try {
      let input;
      let data;
      
      if (reportType === 'turnover') {
        input = {
          companyId,
          startDate,
          endDate,
          accountId: accountId ? parseInt(accountId) : null,
          showZeroBalances
        };
        data = await graphqlRequest(TURNOVER_SHEET_QUERY, { input });
        setReportData({ type: 'turnover', data: data.turnoverSheet });
      } else if (reportType === 'transactions') {
        input = {
          companyId,
          startDate,
          endDate,
          accountId: accountId ? parseInt(accountId) : null
          // Note: TransactionLogInput doesn't have showZeroBalances field
        };
        data = await graphqlRequest(TRANSACTION_LOG_QUERY, { input });
        setReportData({ type: 'transactions', data: data.transactionLog });
      } else if (reportType === 'generalLedger') {
        input = {
          companyId,
          startDate,
          endDate,
          accountId: accountId ? parseInt(accountId) : null
        };
        data = await graphqlRequest(GENERAL_LEDGER_QUERY, { input });
        setReportData({ type: 'generalLedger', data: data.generalLedger });
      } else if (reportType === 'chronological') {
        input = {
          companyId,
          startDate,
          endDate,
          accountId: accountId ? parseInt(accountId) : null
        };
        data = await graphqlRequest(CHRONOLOGICAL_REPORT_QUERY, { input });
        setReportData({ type: 'chronological', data: data.chronologicalReport });
      }
    } catch (err) {
      setError('Грешка при генериране на справката: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const exportReport = async (format) => {
    if (!reportData || (reportType !== 'turnover' && reportType !== 'chronological')) {
      alert('Моля първо генерирайте справка за експорт');
      return;
    }

    setLoading(true);
    try {
      let input;
      if (reportType === 'turnover') {
        input = {
          companyId,
          startDate,
          endDate,
          accountId: accountId ? parseInt(accountId) : null,
          showZeroBalances
        };
      } else {
        input = {
          companyId,
          startDate,
          endDate,
          accountId: accountId ? parseInt(accountId) : null
        };
      }

      let data, exportData;
      if (reportType === 'turnover') {
        data = await graphqlRequest(EXPORT_TURNOVER_MUTATION, { input, format });
        exportData = data.exportTurnoverSheet;
      } else if (reportType === 'chronological') {
        data = await graphqlRequest(EXPORT_CHRONOLOGICAL_MUTATION, { input, format });
        exportData = data.exportChronologicalReport;
      }

      // Handle binary formats (PDF, XLSX)
      const byteCharacters = atob(exportData.content);
      const byteNumbers = new Array(byteCharacters.length);
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i);
      }
      const byteArray = new Uint8Array(byteNumbers);
      const blob = new Blob([byteArray], { type: exportData.mimeType });
      
      const link = document.createElement('a');
      const url = URL.createObjectURL(blob);
      link.setAttribute('href', url);
      link.setAttribute('download', exportData.filename);
      link.style.visibility = 'hidden';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    } catch (err) {
      setError('Грешка при експорт: ' + err.message);
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

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Справки</h1>
            <p className="mt-1 text-sm text-gray-500">
              Генериране на счетоводни справки с възможност за експорт
            </p>
          </div>
          <div className="flex gap-2">
            <Link
              to="/reports/counterparty-turnover"
              className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
            >
              👥 Справка по контрагенти
            </Link>
            <Link
              to="/reports/monthly-stats"
              className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-purple-600 hover:bg-purple-700"
            >
              📊 Месечна статистика (ценообразуване)
            </Link>
          </div>
        </div>

        {/* Report Parameters */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Тип справка
            </label>
            <select
              value={reportType}
              onChange={(e) => setReportType(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="turnover">Оборотна ведомост</option>
              <option value="transactions">Дневник на операциите</option>
              <option value="generalLedger">Главна книга</option>
              <option value="chronological">Хронологичен регистър</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              От дата
            </label>
            <input
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              До дата
            </label>
            <input
              type="date"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Сметка (опционално)
            </label>
            <select
              value={accountId}
              onChange={(e) => setAccountId(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="">Всички сметки</option>
              {accounts.map(account => (
                <option key={account.id} value={account.id}>
                  {account.code} - {account.name}
                </option>
              ))}
            </select>
          </div>

          {reportType === 'turnover' && (
            <div className="flex items-center">
              <input
                type="checkbox"
                id="showZeroBalances"
                checked={showZeroBalances}
                onChange={(e) => setShowZeroBalances(e.target.checked)}
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <label htmlFor="showZeroBalances" className="ml-2 block text-sm text-gray-700">
                Показвай нулеви салда
              </label>
            </div>
          )}
        </div>

        <div className="flex items-center gap-4">
          <button
            onClick={generateReport}
            disabled={loading}
            className="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400"
          >
            {loading ? 'Генерира...' : 'Генерирай справка'}
          </button>

          {reportData && (reportType === 'turnover' || reportType === 'chronological') && (
            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-600">Експорт:</span>
              <button
                onClick={() => exportReport('XLSX')}
                disabled={loading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 text-sm"
              >
                Excel
              </button>
              <button
                onClick={() => exportReport('PDF')}
                disabled={loading}
                className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:bg-gray-400 text-sm"
              >
                PDF
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <h2 className="text-lg font-semibold text-red-800">Грешка</h2>
          <p className="text-red-600 mt-2">{error}</p>
        </div>
      )}

      {/* Report Display */}
      {reportData && (
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">
              {reportType === 'turnover' ? 'Оборотна ведомост' : 
               reportType === 'transactions' ? 'Дневник на операциите' : 
               reportType === 'generalLedger' ? 'Главна книга' : 'Хронологичен регистър'}
            </h3>
            <p className="text-sm text-gray-500">
              {reportData.data.companyName} • {reportData.data.periodStart} - {reportData.data.periodEnd}
            </p>
          </div>

          {reportType === 'turnover' && (
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
                      Обороти за периода
                    </th>
                    <th colSpan="2" className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Крайно салдо
                    </th>
                  </tr>
                  <tr>
                    <th className="px-3 py-2 text-xs font-medium text-gray-500 uppercase">Дебит</th>
                    <th className="px-3 py-2 text-xs font-medium text-gray-500 uppercase border-r">Кредит</th>
                    <th className="px-3 py-2 text-xs font-medium text-gray-500 uppercase">Дебит</th>
                    <th className="px-3 py-2 text-xs font-medium text-gray-500 uppercase border-r">Кредит</th>
                    <th className="px-3 py-2 text-xs font-medium text-gray-500 uppercase">Дебит</th>
                    <th className="px-3 py-2 text-xs font-medium text-gray-500 uppercase">Кредит</th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {reportData.data.entries.map((entry, index) => (
                    <tr key={index} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm border-r">
                        <div className="font-medium text-gray-900">{entry.accountCode}</div>
                        <div className="text-gray-500">{entry.accountName}</div>
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right">
                        {parseFloat(entry.openingDebit) > 0 ? formatCurrency(entry.openingDebit) : '-'}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-r">
                        {parseFloat(entry.openingCredit) > 0 ? formatCurrency(entry.openingCredit) : '-'}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right">
                        {parseFloat(entry.periodDebit) > 0 ? formatCurrency(entry.periodDebit) : '-'}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-r">
                        {parseFloat(entry.periodCredit) > 0 ? formatCurrency(entry.periodCredit) : '-'}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right">
                        {parseFloat(entry.closingDebit) > 0 ? formatCurrency(entry.closingDebit) : '-'}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right">
                        {parseFloat(entry.closingCredit) > 0 ? formatCurrency(entry.closingCredit) : '-'}
                      </td>
                    </tr>
                  ))}
                  {/* Totals Row */}
                  <tr className="bg-gray-100 font-semibold">
                    <td className="px-6 py-4 whitespace-nowrap text-sm border-r border-t-2 border-gray-300">
                      <div className="font-bold text-gray-900">{reportData.data.totals.accountCode}</div>
                    </td>
                    <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-t-2 border-gray-300">
                      {formatCurrency(reportData.data.totals.openingDebit)}
                    </td>
                    <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-r border-t-2 border-gray-300">
                      {formatCurrency(reportData.data.totals.openingCredit)}
                    </td>
                    <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-t-2 border-gray-300">
                      {formatCurrency(reportData.data.totals.periodDebit)}
                    </td>
                    <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-r border-t-2 border-gray-300">
                      {formatCurrency(reportData.data.totals.periodCredit)}
                    </td>
                    <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-t-2 border-gray-300">
                      {formatCurrency(reportData.data.totals.closingDebit)}
                    </td>
                    <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-t-2 border-gray-300">
                      {formatCurrency(reportData.data.totals.closingCredit)}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          )}

          {reportType === 'transactions' && (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дата</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Документ</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Сметка</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Описание</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Дебит</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Кредит</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Контрагент</th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {reportData.data.entries.map((entry, index) => (
                    <tr key={index} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {new Date(entry.date).toLocaleDateString('bg-BG')}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <div>{entry.entryNumber}</div>
                        {entry.documentNumber && (
                          <div className="text-xs text-gray-500">№ {entry.documentNumber}</div>
                        )}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm">
                        <div className="font-medium text-gray-900">{entry.accountCode}</div>
                        <div className="text-gray-500">{entry.accountName}</div>
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {entry.description}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                        {parseFloat(entry.debitAmount) > 0 ? formatCurrency(entry.debitAmount) : '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                        {parseFloat(entry.creditAmount) > 0 ? formatCurrency(entry.creditAmount) : '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {entry.counterpartName || '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {reportType === 'generalLedger' && (
            <div className="space-y-6">
              {reportData.data.accounts.map((account, accountIndex) => (
                <div key={accountIndex} className="border rounded-lg overflow-hidden">
                  {/* Account Header */}
                  <div className="bg-gray-100 px-6 py-4 border-b">
                    <div className="flex justify-between items-center">
                      <div>
                        <h4 className="text-lg font-medium text-gray-900">
                          {account.accountCode} - {account.accountName}
                        </h4>
                      </div>
                      <div className="flex gap-4 text-sm">
                        <span className="text-gray-600">
                          Начално салдо: <span className="font-medium">{formatCurrency(account.openingBalance)}</span>
                        </span>
                        <span className="text-gray-600">
                          Крайно салдо: <span className="font-medium">{formatCurrency(account.closingBalance)}</span>
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* Account Transactions */}
                  <div className="overflow-x-auto">
                    <table className="min-w-full divide-y divide-gray-200">
                      <thead className="bg-gray-50">
                        <tr>
                          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дата</th>
                          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Документ</th>
                          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Описание</th>
                          <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Дебит</th>
                          <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Кредит</th>
                          <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Салдо</th>
                          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Контрагент</th>
                        </tr>
                      </thead>
                      <tbody className="bg-white divide-y divide-gray-200">
                        {/* Opening Balance Row */}
                        {account.openingBalance !== 0 && (
                          <tr className="bg-blue-50">
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                              {new Date(reportData.data.periodStart).toLocaleDateString('bg-BG')}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                              Начално салдо
                            </td>
                            <td className="px-6 py-4 text-sm text-gray-500">
                              Салдо към началото на периода
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right">
                              {account.openingBalance > 0 ? formatCurrency(account.openingBalance) : '-'}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right">
                              {account.openingBalance < 0 ? formatCurrency(Math.abs(account.openingBalance)) : '-'}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right font-medium">
                              {formatCurrency(account.openingBalance)}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">-</td>
                          </tr>
                        )}

                        {/* Transaction Rows */}
                        {account.entries.map((entry, entryIndex) => (
                          <tr key={entryIndex} className="hover:bg-gray-50">
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                              {new Date(entry.date).toLocaleDateString('bg-BG')}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                              <div>{entry.entryNumber}</div>
                              {entry.documentNumber && (
                                <div className="text-xs text-gray-500">№ {entry.documentNumber}</div>
                              )}
                            </td>
                            <td className="px-6 py-4 text-sm text-gray-900">
                              {entry.description}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                              {parseFloat(entry.debitAmount) > 0 ? formatCurrency(entry.debitAmount) : '-'}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                              {parseFloat(entry.creditAmount) > 0 ? formatCurrency(entry.creditAmount) : '-'}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right font-medium">
                              {formatCurrency(entry.balance)}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                              {entry.counterpartName || '-'}
                            </td>
                          </tr>
                        ))}

                        {/* Summary Row */}
                        <tr className="bg-gray-100 font-semibold border-t-2">
                          <td colSpan="3" className="px-6 py-4 text-sm text-gray-900">
                            Общо за сметката:
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-right">
                            {formatCurrency(account.totalDebits)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-right">
                            {formatCurrency(account.totalCredits)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-right font-bold">
                            {formatCurrency(account.closingBalance)}
                          </td>
                          <td></td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
              ))}
            </div>
          )}

          {reportType === 'chronological' && (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дата</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дебит</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дебит име</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Кредит</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Кредит име</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Сума</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Дебит валутна сума</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дебит валута</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Кредит валутна сума</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Кредит валута</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Док. вид</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Док. дата</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Описание</th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {reportData.data.entries.map((entry, index) => (
                    <tr key={index} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {new Date(entry.date).toLocaleDateString('bg-BG')}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                        {entry.debitAccountCode}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {entry.debitAccountName}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                        {entry.creditAccountCode}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {entry.creditAccountName}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                        {formatCurrency(entry.amount)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                        {entry.debitCurrencyAmount ? formatCurrency(entry.debitCurrencyAmount) : '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {entry.debitCurrencyCode || '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                        {entry.creditCurrencyAmount ? formatCurrency(entry.creditCurrencyAmount) : '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {entry.creditCurrencyCode || '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {entry.documentType || '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {entry.documentDate ? new Date(entry.documentDate).toLocaleDateString('bg-BG') : '-'}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {entry.description}
                      </td>
                    </tr>
                  ))}
                  {/* Totals Row */}
                  <tr className="bg-gray-100 font-semibold border-t-2">
                    <td colSpan="5" className="px-6 py-4 text-sm text-gray-900">
                      Общо:
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right font-bold">
                      {formatCurrency(reportData.data.totalAmount)}
                    </td>
                    <td colSpan="7"></td>
                  </tr>
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
