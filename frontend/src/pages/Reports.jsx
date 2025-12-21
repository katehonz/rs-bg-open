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
  const [accountCodeDepth, setAccountCodeDepth] = useState(null);
  
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

  const BG_GENERAL_LEDGER_QUERY = `
    query GetBgGeneralLedger($input: BgGeneralLedgerInput!) {
      bgGeneralLedger(input: $input) {
        companyName
        periodStart
        periodEnd
        byDebit {
          debitAccountCode
          debitAccountName
          entries {
            creditAccountCode
            creditAccountName
            amount
          }
          totalAmount
        }
        byCredit {
          creditAccountCode
          creditAccountName
          entries {
            debitAccountCode
            debitAccountName
            amount
          }
          totalAmount
        }
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

  const EXPORT_GENERAL_LEDGER_MUTATION = `
    mutation ExportGeneralLedger($input: GeneralLedgerInput!, $format: String!) {
      exportGeneralLedger(input: $input, format: $format) {
        format
        content
        filename
        mimeType
      }
    }
  `;

  const EXPORT_BG_GENERAL_LEDGER_MUTATION = `
    mutation ExportBgGeneralLedger($input: BgGeneralLedgerInput!, $format: String!) {
      exportBgGeneralLedger(input: $input, format: $format) {
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
          showZeroBalances,
          accountCodeDepth: accountCodeDepth ? parseInt(accountCodeDepth) : null
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
      } else if (reportType === 'bgGeneralLedger') {
        input = {
          companyId,
          startDate,
          endDate,
          accountId: accountId ? parseInt(accountId) : null
        };
        data = await graphqlRequest(BG_GENERAL_LEDGER_QUERY, { input });
        setReportData({ type: 'bgGeneralLedger', data: data.bgGeneralLedger });
      }
    } catch (err) {
      setError('Грешка при генериране на справката: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const exportReport = async (format) => {
    if (!reportData || (reportType !== 'turnover' && reportType !== 'chronological' && reportType !== 'generalLedger' && reportType !== 'bgGeneralLedger')) {
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
          showZeroBalances,
          accountCodeDepth: accountCodeDepth ? parseInt(accountCodeDepth) : null
        };
      } else if (reportType === 'bgGeneralLedger') {
        input = {
          companyId,
          startDate,
          endDate
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
      } else if (reportType === 'generalLedger') {
        data = await graphqlRequest(EXPORT_GENERAL_LEDGER_MUTATION, { input, format });
        exportData = data.exportGeneralLedger;
      } else if (reportType === 'bgGeneralLedger') {
        data = await graphqlRequest(EXPORT_BG_GENERAL_LEDGER_MUTATION, { input, format });
        exportData = data.exportBgGeneralLedger;
      }

      // Handle binary formats (ODT, XLSX)
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

  const printGeneralLedger = () => {
    if (!reportData || reportType !== 'generalLedger') {
      alert('Моля първо генерирайте справка "Главна книга"');
      return;
    }

    const data = reportData.data;

    // Generate HTML content
    let htmlContent = `
<!DOCTYPE html>
<html lang="bg">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Главна книга - ${data.companyName}</title>
  <style>
    body {
      font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
      margin: 20px;
      font-size: 11pt;
    }
    h1 {
      text-align: center;
      color: #1a202c;
      margin-bottom: 10px;
    }
    .header-info {
      text-align: center;
      color: #4a5568;
      margin-bottom: 30px;
    }
    .account-section {
      margin-bottom: 40px;
      page-break-inside: avoid;
      border: 1px solid #e2e8f0;
      border-radius: 8px;
      overflow: hidden;
    }
    .account-header {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: white;
      padding: 15px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .account-title {
      font-size: 14pt;
      font-weight: bold;
    }
    .account-balances {
      display: flex;
      gap: 20px;
      font-size: 10pt;
    }
    table {
      width: 100%;
      border-collapse: collapse;
    }
    th {
      background-color: #f7fafc;
      padding: 10px;
      text-align: left;
      border-bottom: 2px solid #cbd5e0;
      font-weight: 600;
      color: #2d3748;
      font-size: 9pt;
      text-transform: uppercase;
    }
    td {
      padding: 8px 10px;
      border-bottom: 1px solid #e2e8f0;
      font-size: 10pt;
    }
    tr:hover {
      background-color: #f7fafc;
    }
    .opening-balance-row {
      background-color: #ebf8ff;
      font-weight: 500;
    }
    .summary-row {
      background-color: #f7fafc;
      font-weight: 600;
      border-top: 2px solid #4a5568;
    }
    .amount {
      text-align: right;
      font-family: 'Courier New', monospace;
    }
    .text-right {
      text-align: right;
    }
    @media print {
      body { margin: 0; }
      .account-section { page-break-inside: avoid; }
    }
  </style>
</head>
<body>
  <h1>Главна книга</h1>
  <div class="header-info">
    <strong>${data.companyName}</strong><br>
    Период: ${data.periodStart} - ${data.periodEnd}<br>
    Генерирана на: ${new Date(data.generatedAt).toLocaleString('bg-BG')}
  </div>
`;

    // Iterate through accounts
    if (data.accounts && data.accounts.length > 0) {
      data.accounts.forEach(account => {
        htmlContent += `
  <div class="account-section">
    <div class="account-header">
      <div class="account-title">
        ${account.accountCode} - ${account.accountName}
      </div>
      <div class="account-balances">
        <span>Начално: ${formatCurrency(account.openingBalance)}</span>
        <span>Крайно: ${formatCurrency(account.closingBalance)}</span>
      </div>
    </div>
    <table>
      <thead>
        <tr>
          <th>Дата</th>
          <th>Документ</th>
          <th>Описание</th>
          <th class="text-right">Дебит</th>
          <th class="text-right">Кредит</th>
          <th class="text-right">Салдо</th>
          <th>Контрагент</th>
        </tr>
      </thead>
      <tbody>
`;

        // Opening balance row
        if (account.openingBalance !== 0) {
          htmlContent += `
        <tr class="opening-balance-row">
          <td>${new Date(data.periodStart).toLocaleDateString('bg-BG')}</td>
          <td>Начално салдо</td>
          <td>Салдо към началото на периода</td>
          <td class="amount">${account.openingBalance > 0 ? formatCurrency(account.openingBalance) : '-'}</td>
          <td class="amount">${account.openingBalance < 0 ? formatCurrency(Math.abs(account.openingBalance)) : '-'}</td>
          <td class="amount">${formatCurrency(account.openingBalance)}</td>
          <td>-</td>
        </tr>
`;
        }

        // Transaction entries
        account.entries.forEach(entry => {
          htmlContent += `
        <tr>
          <td>${new Date(entry.date).toLocaleDateString('bg-BG')}</td>
          <td>
            ${entry.entryNumber}
            ${entry.documentNumber ? '<br><small>№ ' + entry.documentNumber + '</small>' : ''}
          </td>
          <td>${entry.description}</td>
          <td class="amount">${parseFloat(entry.debitAmount) > 0 ? formatCurrency(entry.debitAmount) : '-'}</td>
          <td class="amount">${parseFloat(entry.creditAmount) > 0 ? formatCurrency(entry.creditAmount) : '-'}</td>
          <td class="amount">${formatCurrency(entry.balance)}</td>
          <td>${entry.counterpartName || '-'}</td>
        </tr>
`;
        });

        // Summary row
        htmlContent += `
        <tr class="summary-row">
          <td colspan="3">Общо за сметката:</td>
          <td class="amount">${formatCurrency(account.totalDebits)}</td>
          <td class="amount">${formatCurrency(account.totalCredits)}</td>
          <td class="amount">${formatCurrency(account.closingBalance)}</td>
          <td></td>
        </tr>
`;

        htmlContent += `
      </tbody>
    </table>
  </div>
`;
      });
    } else {
      htmlContent += '<p style="text-align: center; color: #718096; padding: 20px;">Няма данни за избрания период.</p>';
    }

    htmlContent += `
</body>
</html>
`;

    // Open in new tab
    const printWindow = window.open('', '_blank');
    printWindow.document.write(htmlContent);
    printWindow.document.close();
  };

  const printBgGeneralLedger = () => {
    if (!reportData || reportType !== 'bgGeneralLedger') {
      alert('Моля първо генерирайте справка "Главна книга (БГ вариант)"');
      return;
    }

    const data = reportData.data;

    // Generate HTML content
    let htmlContent = `
<!DOCTYPE html>
<html lang="bg">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Главна книга (БГ вариант) - ${data.companyName}</title>
  <style>
    body {
      font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
      margin: 20px;
      font-size: 12pt;
    }
    h1 {
      text-align: center;
      color: #1a202c;
      margin-bottom: 10px;
    }
    .header-info {
      text-align: center;
      color: #4a5568;
      margin-bottom: 30px;
    }
    h2 {
      color: #2d3748;
      border-bottom: 3px solid #e2e8f0;
      padding-bottom: 10px;
      margin-top: 40px;
    }
    .account-group {
      margin-bottom: 30px;
      page-break-inside: avoid;
    }
    .account-header {
      background-color: #edf2f7;
      padding: 12px;
      margin-bottom: 10px;
      border-left: 5px solid #4299e1;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .account-header.credit {
      border-left-color: #48bb78;
      background-color: #f0fff4;
    }
    .account-code {
      font-weight: bold;
      color: #2b6cb0;
      font-size: 14pt;
    }
    .account-header.credit .account-code {
      color: #22543d;
    }
    .account-total {
      font-weight: 600;
      color: #2d3748;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }
    th {
      background-color: #f7fafc;
      padding: 10px;
      text-align: left;
      border: 1px solid #cbd5e0;
      font-weight: 600;
      color: #2d3748;
    }
    td {
      padding: 8px 10px;
      border: 1px solid #e2e8f0;
    }
    tr:hover {
      background-color: #f7fafc;
    }
    .amount {
      text-align: right;
      font-family: 'Courier New', monospace;
    }
    .section-title {
      font-size: 16pt;
      margin-top: 40px;
    }
    @media print {
      body { margin: 0; }
      .account-group { page-break-inside: avoid; }
      h2 { page-break-after: avoid; }
    }
  </style>
</head>
<body>
  <h1>Главна книга (БГ вариант)</h1>
  <div class="header-info">
    <strong>${data.companyName}</strong><br>
    Период: ${data.periodStart} - ${data.periodEnd}<br>
    Генерирана на: ${new Date(data.generatedAt).toLocaleString('bg-BG')}
  </div>
`;

    // By Debit section
    htmlContent += '<h2 class="section-title">Главна книга по Дебит</h2>';
    if (data.byDebit && data.byDebit.length > 0) {
      data.byDebit.forEach(debitGroup => {
        htmlContent += `
  <div class="account-group">
    <div class="account-header">
      <div>
        <span class="account-code">${debitGroup.debitAccountCode}</span>
        - ${debitGroup.debitAccountName}
      </div>
      <div class="account-total">Общо: ${formatCurrency(debitGroup.totalAmount)}</div>
    </div>
    <table>
      <thead>
        <tr>
          <th>Кредит сметка</th>
          <th style="text-align: right;">Стойност</th>
        </tr>
      </thead>
      <tbody>
`;
        debitGroup.entries.forEach(entry => {
          htmlContent += `
        <tr>
          <td><strong>${entry.creditAccountCode}</strong> - ${entry.creditAccountName}</td>
          <td class="amount">${formatCurrency(entry.amount)}</td>
        </tr>
`;
        });
        htmlContent += `
      </tbody>
    </table>
  </div>
`;
      });
    } else {
      htmlContent += '<p style="text-align: center; color: #718096; padding: 20px;">Няма данни за дебитни операции в избрания период.</p>';
    }

    // By Credit section
    htmlContent += '<h2 class="section-title">Главна книга по Кредит</h2>';
    if (data.byCredit && data.byCredit.length > 0) {
      data.byCredit.forEach(creditGroup => {
        htmlContent += `
  <div class="account-group">
    <div class="account-header credit">
      <div>
        <span class="account-code">${creditGroup.creditAccountCode}</span>
        - ${creditGroup.creditAccountName}
      </div>
      <div class="account-total">Общо: ${formatCurrency(creditGroup.totalAmount)}</div>
    </div>
    <table>
      <thead>
        <tr>
          <th>Дебит сметка</th>
          <th style="text-align: right;">Стойност</th>
        </tr>
      </thead>
      <tbody>
`;
        creditGroup.entries.forEach(entry => {
          htmlContent += `
        <tr>
          <td><strong>${entry.debitAccountCode}</strong> - ${entry.debitAccountName}</td>
          <td class="amount">${formatCurrency(entry.amount)}</td>
        </tr>
`;
        });
        htmlContent += `
      </tbody>
    </table>
  </div>
`;
      });
    } else {
      htmlContent += '<p style="text-align: center; color: #718096; padding: 20px;">Няма данни за кредитни операции в избрания период.</p>';
    }

    htmlContent += `
</body>
</html>
`;

    // Open in new tab
    const printWindow = window.open('', '_blank');
    printWindow.document.write(htmlContent);
    printWindow.document.close();
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
              <option value="bgGeneralLedger">Главна книга (БГ вариант)</option>
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
            <div className="flex items-center gap-4">
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

              <div className="flex items-center gap-2">
                <label htmlFor="accountCodeDepth" className="text-sm text-gray-700">
                  Дълбочина на сметките:
                </label>
                <select
                  id="accountCodeDepth"
                  value={accountCodeDepth || ''}
                  onChange={(e) => setAccountCodeDepth(e.target.value || null)}
                  className="px-3 py-1 border border-gray-300 rounded text-sm"
                >
                  <option value="">Всички (пълен детайл)</option>
                  <option value="3">3 цифри (синтетични)</option>
                  <option value="4">4 цифри</option>
                  <option value="5">5 цифри</option>
                </select>
              </div>
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
                onClick={() => exportReport('ODT')}
                disabled={loading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 text-sm"
              >
                ODT (LibreOffice)
              </button>
            </div>
          )}

          {reportData && reportType === 'generalLedger' && (
            <div className="flex items-center gap-2">
              <button
                onClick={printGeneralLedger}
                disabled={loading}
                className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:bg-gray-400 text-sm"
              >
                Печат (HTML)
              </button>
              <span className="text-sm text-gray-600">Експорт:</span>
              <button
                onClick={() => exportReport('XLSX')}
                disabled={loading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 text-sm"
              >
                Excel
              </button>
              <button
                onClick={() => exportReport('ODT')}
                disabled={loading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 text-sm"
              >
                ODT (LibreOffice)
              </button>
            </div>
          )}

          {reportData && reportType === 'bgGeneralLedger' && (
            <div className="flex items-center gap-2">
              <button
                onClick={printBgGeneralLedger}
                disabled={loading}
                className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:bg-gray-400 text-sm"
              >
                Печат (HTML)
              </button>
              <span className="text-sm text-gray-600">Експорт:</span>
              <button
                onClick={() => exportReport('XLSX')}
                disabled={loading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 text-sm"
              >
                Excel
              </button>
              <button
                onClick={() => exportReport('ODT')}
                disabled={loading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 text-sm"
              >
                ODT (LibreOffice)
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
               reportType === 'generalLedger' ? 'Главна книга' :
               reportType === 'bgGeneralLedger' ? 'Главна книга (БГ вариант)' :
               'Хронологичен регистър'}
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

          {reportType === 'bgGeneralLedger' && reportData?.data?.byDebit && reportData?.data?.byCredit && (
            <div className="space-y-8">
              {/* Главна книга по Дебит */}
              <div>
                <h4 className="text-xl font-semibold text-gray-900 mb-4 px-6 pt-4">
                  Главна книга по Дебит
                </h4>
                {reportData.data.byDebit.length > 0 ? (
                  reportData.data.byDebit.map((debitGroup, idx) => (
                  <div key={idx} className="mb-6 border-b border-gray-200 pb-4">
                    <div className="bg-blue-50 px-6 py-3 flex justify-between items-center">
                      <div className="font-semibold text-gray-900">
                        <span className="text-blue-700">{debitGroup.debitAccountCode}</span>
                        {' - '}
                        {debitGroup.debitAccountName}
                      </div>
                      <div className="text-sm font-medium text-gray-600">
                        Общо: {formatCurrency(debitGroup.totalAmount)}
                      </div>
                    </div>
                    <table className="min-w-full">
                      <thead className="bg-gray-50">
                        <tr>
                          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                            Кредит сметка
                          </th>
                          <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                            Стойност
                          </th>
                        </tr>
                      </thead>
                      <tbody className="bg-white divide-y divide-gray-200">
                        {debitGroup.entries.map((entry, entryIdx) => (
                          <tr key={entryIdx} className="hover:bg-gray-50">
                            <td className="px-6 py-4 text-sm text-gray-900">
                              <span className="font-medium text-gray-700">{entry.creditAccountCode}</span>
                              {' - '}
                              {entry.creditAccountName}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                              {formatCurrency(entry.amount)}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                  ))
                ) : (
                  <p className="text-gray-500 text-center py-8 px-6">Няма данни за дебитни операции в избрания период.</p>
                )}
              </div>

              {/* Главна книга по Кредит */}
              <div>
                <h4 className="text-xl font-semibold text-gray-900 mb-4 px-6 pt-4 border-t-4 border-gray-300">
                  Главна книга по Кредит
                </h4>
                {reportData.data.byCredit.length > 0 ? (
                  reportData.data.byCredit.map((creditGroup, idx) => (
                  <div key={idx} className="mb-6 border-b border-gray-200 pb-4">
                    <div className="bg-green-50 px-6 py-3 flex justify-between items-center">
                      <div className="font-semibold text-gray-900">
                        <span className="text-green-700">{creditGroup.creditAccountCode}</span>
                        {' - '}
                        {creditGroup.creditAccountName}
                      </div>
                      <div className="text-sm font-medium text-gray-600">
                        Общо: {formatCurrency(creditGroup.totalAmount)}
                      </div>
                    </div>
                    <table className="min-w-full">
                      <thead className="bg-gray-50">
                        <tr>
                          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                            Дебит сметка
                          </th>
                          <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                            Стойност
                          </th>
                        </tr>
                      </thead>
                      <tbody className="bg-white divide-y divide-gray-200">
                        {creditGroup.entries.map((entry, entryIdx) => (
                          <tr key={entryIdx} className="hover:bg-gray-50">
                            <td className="px-6 py-4 text-sm text-gray-900">
                              <span className="font-medium text-gray-700">{entry.debitAccountCode}</span>
                              {' - '}
                              {entry.debitAccountName}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                              {formatCurrency(entry.amount)}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                  ))
                ) : (
                  <p className="text-gray-500 text-center py-8 px-6">Няма данни за кредитни операции в избрания период.</p>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
