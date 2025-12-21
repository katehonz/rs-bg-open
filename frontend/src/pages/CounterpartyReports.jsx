import { useState, useEffect, useMemo } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

export default function CounterpartyReports() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  // Filters
  const [reportType, setReportType] = useState('turnover'); // 'turnover' or 'chronological'
  const [startDate, setStartDate] = useState(new Date().toISOString().split('T')[0]);
  const [endDate, setEndDate] = useState(new Date().toISOString().split('T')[0]);
  const [accountId, setAccountId] = useState('');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCounterpart, setSelectedCounterpart] = useState(null);
  const [showSuggestions, setShowSuggestions] = useState(false);

  // Data
  const [accounts, setAccounts] = useState([]);
  const [counterparts, setCounterparts] = useState([]);
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

  const COUNTERPARTS_QUERY = `
    query GetCounterparts($companyId: Int!) {
      counterparts(companyId: $companyId) {
        id
        name
        eik
        vatNumber
        counterpartType
        isCustomer
        isSupplier
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

  useEffect(() => {
    loadAccounts();
    loadCounterparts();
  }, [companyId]);

  // Close suggestions when clicking outside
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (!event.target.closest('.counterpart-search')) {
        setShowSuggestions(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const loadAccounts = async () => {
    try {
      const data = await graphqlRequest(ACCOUNTS_QUERY, { companyId });
      const activeAccounts = (data.accountHierarchy || []).filter(a => a.isActive);
      setAccounts(activeAccounts);
    } catch (err) {
      setError('Грешка при зареждане на сметките: ' + err.message);
    }
  };

  const loadCounterparts = async () => {
    try {
      const data = await graphqlRequest(COUNTERPARTS_QUERY, { companyId });
      setCounterparts(data.counterparts || []);
    } catch (err) {
      setError('Грешка при зареждане на контрагенти: ' + err.message);
    }
  };

  // Filter counterparts based on search term
  const filteredCounterparts = useMemo(() => {
    if (!searchTerm.trim()) return [];

    const search = searchTerm.toLowerCase();
    return counterparts.filter(cp =>
      cp.name.toLowerCase().includes(search) ||
      (cp.eik && cp.eik.toLowerCase().includes(search)) ||
      (cp.vatNumber && cp.vatNumber.toLowerCase().includes(search))
    ).slice(0, 10); // Limit to 10 suggestions
  }, [counterparts, searchTerm]);

  const generateReport = async () => {
    if (reportType === 'turnover' && !accountId) {
      setError('Моля изберете сметка за оборотна ведомост');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const input = {
        companyId,
        startDate,
        endDate,
        accountId: accountId ? parseInt(accountId) : null
      };

      const data = await graphqlRequest(TRANSACTION_LOG_QUERY, { input });

      // If chronological report - just show the entries
      if (reportType === 'chronological') {
        // Filter by selected counterpart if needed
        let entries = data.transactionLog.entries;
        if (selectedCounterpart) {
          entries = entries.filter(entry => entry.counterpartName === selectedCounterpart.name);
        }

        const selectedAccount = accountId ? accounts.find(a => a.id === parseInt(accountId)) : null;

        setReportData({
          type: 'chronological',
          companyName: data.transactionLog.companyName,
          periodStart: data.transactionLog.periodStart,
          periodEnd: data.transactionLog.periodEnd,
          accountCode: selectedAccount?.code || '',
          accountName: selectedAccount?.name || '',
          entries,
          generatedAt: data.transactionLog.generatedAt
        });
        setLoading(false);
        return;
      }

      // Process transactions to create turnover sheet by counterpart
      const counterpartMap = new Map();

      // Get entries before start date for opening balance
      const openingInput = {
        companyId,
        startDate: '1900-01-01',
        endDate: new Date(new Date(startDate).getTime() - 86400000).toISOString().split('T')[0], // Day before startDate
        accountId: parseInt(accountId)
      };

      let openingData;
      try {
        openingData = await graphqlRequest(TRANSACTION_LOG_QUERY, { input: openingInput });
      } catch (err) {
        console.log('No opening data available');
        openingData = { transactionLog: { entries: [] } };
      }

      // Calculate opening balances by counterpart
      for (const entry of openingData.transactionLog?.entries || []) {
        const cpName = entry.counterpartName || '(Без контрагент)';

        if (!counterpartMap.has(cpName)) {
          counterpartMap.set(cpName, {
            name: cpName,
            openingDebit: 0,
            openingCredit: 0,
            periodDebit: 0,
            periodCredit: 0,
            closingDebit: 0,
            closingCredit: 0
          });
        }

        const cp = counterpartMap.get(cpName);
        cp.openingDebit += parseFloat(entry.debitAmount || 0);
        cp.openingCredit += parseFloat(entry.creditAmount || 0);
      }

      // Calculate period turnovers by counterpart
      for (const entry of data.transactionLog.entries) {
        const cpName = entry.counterpartName || '(Без контрагент)';

        if (!counterpartMap.has(cpName)) {
          counterpartMap.set(cpName, {
            name: cpName,
            openingDebit: 0,
            openingCredit: 0,
            periodDebit: 0,
            periodCredit: 0,
            closingDebit: 0,
            closingCredit: 0
          });
        }

        const cp = counterpartMap.get(cpName);
        cp.periodDebit += parseFloat(entry.debitAmount || 0);
        cp.periodCredit += parseFloat(entry.creditAmount || 0);
      }

      // Calculate closing balances
      const entries = [];
      for (const [name, cp] of counterpartMap.entries()) {
        // Skip if selected counterpart doesn't match
        if (selectedCounterpart && cp.name !== selectedCounterpart.name) {
          continue;
        }

        const netClosing = (cp.openingDebit + cp.periodDebit) - (cp.openingCredit + cp.periodCredit);

        if (netClosing > 0) {
          cp.closingDebit = netClosing;
          cp.closingCredit = 0;
        } else {
          cp.closingDebit = 0;
          cp.closingCredit = Math.abs(netClosing);
        }

        entries.push(cp);
      }

      // Sort by name
      entries.sort((a, b) => a.name.localeCompare(b.name, 'bg'));

      const selectedAccount = accounts.find(a => a.id === parseInt(accountId));

      setReportData({
        type: 'turnover',
        companyName: data.transactionLog.companyName,
        periodStart: data.transactionLog.periodStart,
        periodEnd: data.transactionLog.periodEnd,
        accountCode: selectedAccount?.code || '',
        accountName: selectedAccount?.name || '',
        entries,
        generatedAt: data.transactionLog.generatedAt
      });
    } catch (err) {
      setError('Грешка при генериране на справката: ' + err.message);
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

  const exportToExcel = () => {
    if (!reportData || !reportData.entries) {
      alert('Моля първо генерирайте справка');
      return;
    }

    let headers, rows, filename;
    const accountDesc = reportData.accountCode ? `_${reportData.accountCode}` : '';
    const counterpartDesc = selectedCounterpart ? `_${selectedCounterpart.name}` : '';

    if (reportData.type === 'chronological') {
      // Chronological report
      headers = ['Дата', 'Документ №', 'Контрагент', 'Сметка', 'Име на сметка', 'Описание', 'Дебит', 'Кредит'];
      rows = reportData.entries.map(entry => [
        new Date(entry.date).toLocaleDateString('bg-BG'),
        entry.documentNumber || entry.entryNumber,
        entry.counterpartName || '-',
        entry.accountCode,
        entry.accountName,
        entry.description,
        parseFloat(entry.debitAmount || 0).toFixed(2),
        parseFloat(entry.creditAmount || 0).toFixed(2)
      ]);

      // Add totals row
      rows.push([
        '', '', '', '', 'ОБЩО:', '',
        totals.totalDebit.toFixed(2),
        totals.totalCredit.toFixed(2)
      ]);

      filename = `хронология_контрагенти${accountDesc}_${reportData.periodStart}_${reportData.periodEnd}${counterpartDesc}.csv`;
    } else {
      // Turnover report
      headers = [
        'Контрагент',
        'Начално салдо Дебит',
        'Начално салдо Кредит',
        'Обороти Дебит',
        'Обороти Кредит',
        'Крайно салдо Дебит',
        'Крайно салдо Кредит'
      ];

      rows = reportData.entries.map(entry => [
        entry.name,
        entry.openingDebit.toFixed(2),
        entry.openingCredit.toFixed(2),
        entry.periodDebit.toFixed(2),
        entry.periodCredit.toFixed(2),
        entry.closingDebit.toFixed(2),
        entry.closingCredit.toFixed(2)
      ]);

      // Add totals row
      rows.push([
        'ОБЩО:',
        totals.openingDebit.toFixed(2),
        totals.openingCredit.toFixed(2),
        totals.periodDebit.toFixed(2),
        totals.periodCredit.toFixed(2),
        totals.closingDebit.toFixed(2),
        totals.closingCredit.toFixed(2)
      ]);

      filename = `оборотна_контрагенти${accountDesc}_${reportData.periodStart}_${reportData.periodEnd}${counterpartDesc}.csv`;
    }

    const csvContent = [
      headers.join(','),
      ...rows.map(row => row.map(cell => `"${cell}"`).join(','))
    ].join('\n');

    // Create and download file
    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);

    link.setAttribute('href', url);
    link.setAttribute('download', filename);
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const printReport = () => {
    if (!reportData || !reportData.entries) {
      alert('Моля първо генерирайте справка');
      return;
    }

    const reportTitle = reportData.type === 'chronological'
      ? 'Хронологична справка по контрагенти'
      : 'Оборотна ведомост по контрагенти';

    let tableContent;

    if (reportData.type === 'chronological') {
      // Chronological table
      tableContent = `
        <table>
          <thead>
            <tr>
              <th>Дата</th>
              <th>Документ</th>
              <th>Контрагент</th>
              <th>Сметка</th>
              <th>Описание</th>
              <th>Дебит (лв.)</th>
              <th>Кредит (лв.)</th>
            </tr>
          </thead>
          <tbody>
            ${reportData.entries.map(entry => `
              <tr>
                <td>${new Date(entry.date).toLocaleDateString('bg-BG')}</td>
                <td>${entry.documentNumber || entry.entryNumber}</td>
                <td>${entry.counterpartName || '-'}</td>
                <td>${entry.accountCode} - ${entry.accountName}</td>
                <td>${entry.description}</td>
                <td class="number debit">${parseFloat(entry.debitAmount) > 0 ? parseFloat(entry.debitAmount).toFixed(2) : '-'}</td>
                <td class="number credit">${parseFloat(entry.creditAmount) > 0 ? parseFloat(entry.creditAmount).toFixed(2) : '-'}</td>
              </tr>
            `).join('')}
            <tr class="totals">
              <td colspan="5" style="text-align: right;">ОБЩО:</td>
              <td class="number">${totals.totalDebit.toFixed(2)}</td>
              <td class="number">${totals.totalCredit.toFixed(2)}</td>
            </tr>
          </tbody>
        </table>
      `;
    } else {
      // Turnover table
      tableContent = `
        <table>
          <thead>
            <tr>
              <th rowspan="2">Контрагент</th>
              <th colspan="2">Начално салдо</th>
              <th colspan="2">Обороти за периода</th>
              <th colspan="2">Крайно салдо</th>
            </tr>
            <tr>
              <th>Дебит</th>
              <th>Кредит</th>
              <th>Дебит</th>
              <th>Кредит</th>
              <th>Дебит</th>
              <th>Кредит</th>
            </tr>
          </thead>
          <tbody>
            ${reportData.entries.map(entry => `
              <tr>
                <td>${entry.name}</td>
                <td class="number">${entry.openingDebit > 0 ? entry.openingDebit.toFixed(2) : '-'}</td>
                <td class="number">${entry.openingCredit > 0 ? entry.openingCredit.toFixed(2) : '-'}</td>
                <td class="number debit">${entry.periodDebit > 0 ? entry.periodDebit.toFixed(2) : '-'}</td>
                <td class="number credit">${entry.periodCredit > 0 ? entry.periodCredit.toFixed(2) : '-'}</td>
                <td class="number">${entry.closingDebit > 0 ? entry.closingDebit.toFixed(2) : '-'}</td>
                <td class="number">${entry.closingCredit > 0 ? entry.closingCredit.toFixed(2) : '-'}</td>
              </tr>
            `).join('')}
            <tr class="totals">
              <td style="text-align: center;">ОБЩО:</td>
              <td class="number">${totals.openingDebit.toFixed(2)}</td>
              <td class="number">${totals.openingCredit.toFixed(2)}</td>
              <td class="number">${totals.periodDebit.toFixed(2)}</td>
              <td class="number">${totals.periodCredit.toFixed(2)}</td>
              <td class="number">${totals.closingDebit.toFixed(2)}</td>
              <td class="number">${totals.closingCredit.toFixed(2)}</td>
            </tr>
          </tbody>
        </table>
      `;
    }

    // Create printable HTML
    const printContent = `
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="UTF-8">
        <title>${reportTitle}</title>
        <style>
          body {
            font-family: Arial, sans-serif;
            margin: 20px;
            font-size: 11px;
          }
          .header {
            text-align: center;
            margin-bottom: 20px;
          }
          .title {
            font-size: 18px;
            font-weight: bold;
            margin-bottom: 5px;
          }
          .subtitle {
            font-size: 12px;
            color: #666;
            margin-bottom: 5px;
          }
          table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 10px;
            font-size: 10px;
          }
          th, td {
            border: 1px solid #333;
            padding: 6px 8px;
            text-align: left;
          }
          th {
            background-color: #f0f0f0;
            font-weight: bold;
            text-align: center;
          }
          .number {
            text-align: right;
          }
          .totals {
            font-weight: bold;
            background-color: #f5f5f5;
          }
          .debit { color: #059669; }
          .credit { color: #DC2626; }
        </style>
      </head>
      <body>
        <div class="header">
          <div class="title">${reportTitle}</div>
          <div class="subtitle">${reportData.companyName}</div>
          <div class="subtitle">Период: ${reportData.periodStart} - ${reportData.periodEnd}</div>
          ${reportData.accountCode ? `<div class="subtitle">Сметка: ${reportData.accountCode} - ${reportData.accountName}</div>` : ''}
          ${selectedCounterpart ? `<div class="subtitle">Контрагент: ${selectedCounterpart.name}</div>` : ''}
        </div>

        ${tableContent}

        <div style="margin-top: 20px; font-size: 9px; color: #666;">
          Генерирано на: ${new Date().toLocaleDateString('bg-BG')} ${new Date().toLocaleTimeString('bg-BG')}
        </div>
      </body>
      </html>
    `;

    // Open print dialog
    const printWindow = window.open('', '_blank');
    printWindow.document.write(printContent);
    printWindow.document.close();
    printWindow.focus();
    setTimeout(() => {
      printWindow.print();
    }, 250);
  };

  // Calculate totals
  const calculateTotals = () => {
    if (!reportData || !reportData.entries) {
      return {
        openingDebit: 0,
        openingCredit: 0,
        periodDebit: 0,
        periodCredit: 0,
        closingDebit: 0,
        closingCredit: 0,
        totalDebit: 0,
        totalCredit: 0
      };
    }

    // For chronological reports
    if (reportData.type === 'chronological') {
      const totalDebit = reportData.entries.reduce((sum, entry) => sum + parseFloat(entry.debitAmount || 0), 0);
      const totalCredit = reportData.entries.reduce((sum, entry) => sum + parseFloat(entry.creditAmount || 0), 0);

      return {
        openingDebit: 0,
        openingCredit: 0,
        periodDebit: 0,
        periodCredit: 0,
        closingDebit: 0,
        closingCredit: 0,
        totalDebit,
        totalCredit
      };
    }

    // For turnover reports
    const totals = {
      openingDebit: 0,
      openingCredit: 0,
      periodDebit: 0,
      periodCredit: 0,
      closingDebit: 0,
      closingCredit: 0,
      totalDebit: 0,
      totalCredit: 0
    };

    for (const entry of reportData.entries) {
      totals.openingDebit += entry.openingDebit || 0;
      totals.openingCredit += entry.openingCredit || 0;
      totals.periodDebit += entry.periodDebit || 0;
      totals.periodCredit += entry.periodCredit || 0;
      totals.closingDebit += entry.closingDebit || 0;
      totals.closingCredit += entry.closingCredit || 0;
    }

    return totals;
  };

  const totals = calculateTotals();

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Справки по контрагенти</h1>
            <p className="mt-1 text-sm text-gray-500">
              Хронологична справка или оборотна ведомост по контрагенти
            </p>
          </div>
        </div>

        {/* Filters */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Вид справка
            </label>
            <select
              value={reportType}
              onChange={(e) => setReportType(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="turnover">Оборотна ведомост</option>
              <option value="chronological">Хронологична справка</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Сметка {reportType === 'turnover' ? '*' : '(опц.)'}
            </label>
            <select
              value={accountId}
              onChange={(e) => setAccountId(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="">Изберете сметка</option>
              {accounts.map(account => (
                <option key={account.id} value={account.id}>
                  {account.code} - {account.name}
                </option>
              ))}
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

          <div className="relative counterpart-search">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Контрагент (опционално)
            </label>
            <div className="relative">
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => {
                  setSearchTerm(e.target.value);
                  setShowSuggestions(true);
                }}
                onFocus={() => setShowSuggestions(true)}
                placeholder="Търсене по име, ЕИК или ДДС номер..."
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm pr-20"
              />
              {selectedCounterpart && (
                <button
                  onClick={() => {
                    setSelectedCounterpart(null);
                    setSearchTerm('');
                  }}
                  className="absolute right-2 top-1/2 -translate-y-1/2 px-2 py-1 text-xs text-gray-500 hover:text-gray-700"
                >
                  ✕ Изчисти
                </button>
              )}
            </div>
            {selectedCounterpart && (
              <div className="mt-2 px-3 py-2 bg-blue-50 border border-blue-200 rounded-md text-sm">
                <span className="font-medium text-blue-900">Избран: </span>
                <span className="text-blue-700">{selectedCounterpart.name}</span>
                {selectedCounterpart.vatNumber && (
                  <span className="text-blue-600 ml-2">({selectedCounterpart.vatNumber})</span>
                )}
              </div>
            )}
            {showSuggestions && filteredCounterparts.length > 0 && (
              <div className="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg max-h-60 overflow-y-auto">
                {filteredCounterparts.map((counterpart) => (
                  <button
                    key={counterpart.id}
                    onClick={() => {
                      setSelectedCounterpart(counterpart);
                      setSearchTerm(counterpart.name);
                      setShowSuggestions(false);
                    }}
                    className="w-full text-left px-4 py-2 hover:bg-gray-100 focus:bg-gray-100 focus:outline-none"
                  >
                    <div className="font-medium text-gray-900">{counterpart.name}</div>
                    <div className="text-xs text-gray-500">
                      {counterpart.eik && `ЕИК: ${counterpart.eik}`}
                      {counterpart.eik && counterpart.vatNumber && ' • '}
                      {counterpart.vatNumber && `ДДС: ${counterpart.vatNumber}`}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="flex items-center gap-4">
          <button
            onClick={generateReport}
            disabled={loading}
            className="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400"
          >
            {loading ? 'Генерира...' : 'Генерирай справка'}
          </button>

          {reportData && reportData.entries && (
            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-600">Експорт:</span>
              <button
                onClick={exportToExcel}
                className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 text-sm"
              >
                📊 Excel
              </button>
              <button
                onClick={printReport}
                className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 text-sm"
              >
                🖨️ Печат
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

      {/* Summary Cards */}
      {reportData && reportData.entries && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="bg-green-50 p-6 rounded-lg border-2 border-green-200">
            <div className="text-sm font-medium text-green-800 mb-1">
              {reportData.type === 'chronological' ? 'Общо Дебит' : 'Обороти Дебит'}
            </div>
            <div className="text-2xl font-bold text-green-600">
              {formatCurrency(reportData.type === 'chronological' ? totals.totalDebit : totals.periodDebit)}
            </div>
          </div>
          <div className="bg-red-50 p-6 rounded-lg border-2 border-red-200">
            <div className="text-sm font-medium text-red-800 mb-1">
              {reportData.type === 'chronological' ? 'Общо Кредит' : 'Обороти Кредит'}
            </div>
            <div className="text-2xl font-bold text-red-600">
              {formatCurrency(reportData.type === 'chronological' ? totals.totalCredit : totals.periodCredit)}
            </div>
          </div>
        </div>
      )}

      {/* Report Display */}
      {reportData && (
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">
              {reportData.type === 'chronological' ? 'Хронологична справка по контрагенти' : 'Оборотна ведомост по контрагенти'}
            </h3>
            <p className="text-sm text-gray-500">
              {reportData.companyName} • {reportData.periodStart} - {reportData.periodEnd}
            </p>
            {reportData.accountCode && (
              <p className="text-sm font-medium text-blue-600 mt-1">
                Сметка: {reportData.accountCode} - {reportData.accountName}
              </p>
            )}
            {selectedCounterpart && (
              <p className="text-sm font-medium text-indigo-600 mt-1">
                Контрагент: {selectedCounterpart.name}
              </p>
            )}
            <p className="text-xs text-gray-400 mt-1">
              Общо {reportData.entries.length} {reportData.type === 'chronological' ? 'записа' : `контрагент${reportData.entries.length !== 1 ? 'а' : ''}`}
            </p>
          </div>

          <div className="overflow-x-auto">
            {reportData.type === 'chronological' ? (
              /* Chronological Table */
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дата</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Документ</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Контрагент</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Сметка</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Описание</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Дебит</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Кредит</th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {reportData.entries.length === 0 ? (
                    <tr>
                      <td colSpan="7" className="px-6 py-8 text-center text-gray-500">
                        {selectedCounterpart
                          ? 'Няма транзакции за избрания контрагент в този период'
                          : 'Няма транзакции за този период'
                        }
                      </td>
                    </tr>
                  ) : (
                    reportData.entries.map((entry, index) => (
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
                          <div className="font-medium text-gray-900">
                            {entry.counterpartName || '-'}
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm">
                          <div className="font-medium text-gray-900">{entry.accountCode}</div>
                          <div className="text-gray-500">{entry.accountName}</div>
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-900">
                          {entry.description}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-green-600 font-medium">
                          {parseFloat(entry.debitAmount) > 0 ? formatCurrency(entry.debitAmount) : '-'}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-red-600 font-medium">
                          {parseFloat(entry.creditAmount) > 0 ? formatCurrency(entry.creditAmount) : '-'}
                        </td>
                      </tr>
                    ))
                  )}
                  {reportData.entries.length > 0 && (
                    <tr className="bg-gray-100 font-semibold border-t-2 border-gray-300">
                      <td colSpan="5" className="px-6 py-4 text-sm text-gray-900 text-right">
                        Общо:
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-green-700 font-bold">
                        {formatCurrency(totals.totalDebit)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-red-700 font-bold">
                        {formatCurrency(totals.totalCredit)}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            ) : (
              /* Turnover Table */
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th rowSpan="2" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-r">
                      Контрагент
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
                  {reportData.entries.length === 0 ? (
                    <tr>
                      <td colSpan="7" className="px-6 py-8 text-center text-gray-500">
                        {selectedCounterpart
                          ? 'Няма данни за избрания контрагент в този период'
                          : 'Няма данни за контрагенти в този период'
                        }
                      </td>
                    </tr>
                  ) : (
                    reportData.entries.map((entry, index) => (
                      <tr key={index} className="hover:bg-gray-50">
                        <td className="px-6 py-4 text-sm border-r">
                          <div className="font-medium text-gray-900">{entry.name}</div>
                        </td>
                        <td className="px-3 py-4 whitespace-nowrap text-sm text-right">
                          {entry.openingDebit > 0 ? formatCurrency(entry.openingDebit) : '-'}
                        </td>
                        <td className="px-3 py-4 whitespace-nowrap text-sm text-right border-r">
                          {entry.openingCredit > 0 ? formatCurrency(entry.openingCredit) : '-'}
                        </td>
                        <td className="px-3 py-4 whitespace-nowrap text-sm text-right text-green-600 font-medium">
                          {entry.periodDebit > 0 ? formatCurrency(entry.periodDebit) : '-'}
                        </td>
                        <td className="px-3 py-4 whitespace-nowrap text-sm text-right text-red-600 font-medium border-r">
                          {entry.periodCredit > 0 ? formatCurrency(entry.periodCredit) : '-'}
                        </td>
                        <td className="px-3 py-4 whitespace-nowrap text-sm text-right">
                          {entry.closingDebit > 0 ? formatCurrency(entry.closingDebit) : '-'}
                        </td>
                        <td className="px-3 py-4 whitespace-nowrap text-sm text-right">
                          {entry.closingCredit > 0 ? formatCurrency(entry.closingCredit) : '-'}
                        </td>
                      </tr>
                    ))
                  )}
                  {reportData.entries.length > 0 && (
                    <tr className="bg-gray-100 font-semibold border-t-2 border-gray-300">
                      <td className="px-6 py-4 text-sm text-gray-900 border-r">
                        <div className="font-bold">ОБЩО:</div>
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right font-bold">
                        {formatCurrency(totals.openingDebit)}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right font-bold border-r">
                        {formatCurrency(totals.openingCredit)}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right text-green-700 font-bold">
                        {formatCurrency(totals.periodDebit)}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right text-red-700 font-bold border-r">
                        {formatCurrency(totals.periodCredit)}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right font-bold">
                        {formatCurrency(totals.closingDebit)}
                      </td>
                      <td className="px-3 py-4 whitespace-nowrap text-sm text-right font-bold">
                        {formatCurrency(totals.closingCredit)}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
