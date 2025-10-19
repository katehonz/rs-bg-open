import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';
import html2pdf from 'html2pdf.js';
import * as XLSX from 'xlsx';

export default function DepreciationReport() {
  const currentYear = new Date().getFullYear();
  const currentMonth = new Date().getMonth() + 1;

  const [year, setYear] = useState(currentYear);
  const [month, setMonth] = useState(currentMonth);
  const [reportData, setReportData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [companyInfo, setCompanyInfo] = useState(null);

  const REPORT_QUERY = `
    query GetDepreciationReport($companyId: Int!, $year: Int!, $month: Int!) {
      depreciationJournal(companyId: $companyId, year: $year, month: $month) {
        id
        fixedAssetId
        period
        accountingDepreciationAmount
        accountingBookValueBefore
        accountingBookValueAfter
        taxDepreciationAmount
        taxBookValueBefore
        taxBookValueAfter
        isPosted
      }

      fixedAssets(companyId: $companyId) {
        id
        inventoryNumber
        name
        categoryId
        acquisitionCost
        acquisitionDate
        accountingUsefulLife
        accountingDepreciationRate
        accountingAccumulatedDepreciation
        taxDepreciationRate
        taxAccumulatedDepreciation
        status
      }

      fixedAssetCategories {
        id
        name
        code
        taxCategory
      }

      companies {
        id
        name
        eik
        vatNumber
        address
        city
      }
    }
  `;

  const monthNames = [
    'Януари', 'Февруари', 'Март', 'Април', 'Май', 'Юни',
    'Юли', 'Август', 'Септември', 'Октомври', 'Ноември', 'Декември'
  ];

  const loadReport = async () => {
    try {
      setLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const response = await graphqlRequest(REPORT_QUERY, {
        companyId,
        year,
        month
      });

      // Find current company info
      const company = response.companies?.find(c => c.id === companyId);
      setCompanyInfo(company);

      // Create maps for lookups
      const assetsMap = {};
      const categoriesMap = {};

      response.fixedAssets?.forEach(asset => {
        assetsMap[asset.id] = asset;
      });

      response.fixedAssetCategories?.forEach(cat => {
        categoriesMap[cat.id] = cat;
      });

      // Process depreciation data
      const processedData = response.depreciationJournal?.map(entry => {
        const asset = assetsMap[entry.fixedAssetId];
        return {
          ...entry,
          asset,
          category: categoriesMap[asset?.categoryId],
          // Use accumulated depreciation from assets table
          accountingAccumulatedDepreciation: asset?.accountingAccumulatedDepreciation || 0,
          taxAccumulatedDepreciation: asset?.taxAccumulatedDepreciation || 0
        };
      });

      setReportData(processedData);
    } catch (err) {
      console.error('Error loading report:', err);
      alert('Грешка при зареждане на справката');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadReport();
  }, [year, month]);

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount || 0);
  };

  const formatDate = (date) => {
    return new Date(date).toLocaleDateString('bg-BG');
  };

  const calculateTotals = () => {
    if (!reportData) return {};

    return reportData.reduce((acc, entry) => ({
      accountingDepreciation: (acc.accountingDepreciation || 0) + parseFloat(entry.accountingDepreciationAmount),
      taxDepreciation: (acc.taxDepreciation || 0) + parseFloat(entry.taxDepreciationAmount),
      acquisitionCost: (acc.acquisitionCost || 0) + parseFloat(entry.asset?.acquisitionCost || 0),
      accountingBookValue: (acc.accountingBookValue || 0) + parseFloat(entry.accountingBookValueAfter),
      taxBookValue: (acc.taxBookValue || 0) + parseFloat(entry.taxBookValueAfter)
    }), {});
  };

  const exportToPDF = () => {
    const element = document.getElementById('depreciation-report');
    const opt = {
      margin: 10,
      filename: `амортизации_${year}_${String(month).padStart(2, '0')}.pdf`,
      image: { type: 'jpeg', quality: 0.98 },
      html2canvas: { scale: 2 },
      jsPDF: { unit: 'mm', format: 'a4', orientation: 'landscape' }
    };

    html2pdf().set(opt).from(element).save();
  };

  const exportToExcel = () => {
    if (!reportData) return;

    const wsData = [
      // Header
      ['СПРАВКА ЗА АМОРТИЗАЦИИТЕ'],
      [`${companyInfo?.name || 'Фирма'} - ЕИК: ${companyInfo?.eik || ''}`],
      [`Период: ${monthNames[month - 1]} ${year}`],
      [],
      // Column headers
      [
        'Инв. №',
        'Наименование',
        'Категория',
        'Отчетна стойност',
        'Дата придобиване',
        'Счет. амортизация',
        'Счет. натрупана',
        'Счет. балансова',
        'Данъчна амортизация',
        'Данъчна натрупана',
        'Данъчна балансова',
        'Разлика'
      ]
    ];

    // Add data rows
    reportData.forEach(entry => {
      wsData.push([
        entry.asset?.inventoryNumber || '',
        entry.asset?.name || '',
        entry.category?.name || '',
        formatCurrency(entry.asset?.acquisitionCost || 0),
        formatDate(entry.asset?.acquisitionDate),
        formatCurrency(entry.accountingDepreciationAmount),
        formatCurrency(entry.accountingAccumulatedDepreciation),
        formatCurrency(entry.accountingBookValueAfter),
        formatCurrency(entry.taxDepreciationAmount),
        formatCurrency(entry.taxAccumulatedDepreciation),
        formatCurrency(entry.taxBookValueAfter),
        formatCurrency(parseFloat(entry.taxDepreciationAmount) - parseFloat(entry.accountingDepreciationAmount))
      ]);
    });

    // Add totals
    const totals = calculateTotals();
    wsData.push([]);
    wsData.push([
      'ОБЩО',
      '',
      '',
      formatCurrency(totals.acquisitionCost),
      '',
      formatCurrency(totals.accountingDepreciation),
      '',
      formatCurrency(totals.accountingBookValue),
      formatCurrency(totals.taxDepreciation),
      '',
      formatCurrency(totals.taxBookValue),
      formatCurrency(totals.taxDepreciation - totals.accountingDepreciation)
    ]);

    const wb = XLSX.utils.book_new();
    const ws = XLSX.utils.aoa_to_sheet(wsData);

    // Set column widths
    ws['!cols'] = [
      { wch: 10 }, // Инв. №
      { wch: 30 }, // Наименование
      { wch: 15 }, // Категория
      { wch: 15 }, // Отчетна стойност
      { wch: 12 }, // Дата
      { wch: 15 }, // Счет. аморт
      { wch: 15 }, // Счет. натрупана
      { wch: 15 }, // Счет. балансова
      { wch: 15 }, // Данъчна аморт
      { wch: 15 }, // Данъчна натрупана
      { wch: 15 }, // Данъчна балансова
      { wch: 12 }  // Разлика
    ];

    XLSX.utils.book_append_sheet(wb, ws, 'Амортизации');
    XLSX.writeFile(wb, `амортизации_${year}_${String(month).padStart(2, '0')}.xlsx`);
  };

  const totals = calculateTotals();

  return (
    <div className="space-y-6">
      {/* Controls */}
      <div className="bg-white rounded-lg shadow p-4">
        <div className="flex items-end gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Година
            </label>
            <select
              value={year}
              onChange={(e) => setYear(parseInt(e.target.value))}
              className="block w-32 px-3 py-2 border border-gray-300 rounded-md"
            >
              {[currentYear - 2, currentYear - 1, currentYear, currentYear + 1].map(y => (
                <option key={y} value={y}>{y}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Месец
            </label>
            <select
              value={month}
              onChange={(e) => setMonth(parseInt(e.target.value))}
              className="block w-40 px-3 py-2 border border-gray-300 rounded-md"
            >
              {monthNames.map((name, idx) => (
                <option key={idx + 1} value={idx + 1}>
                  {name}
                </option>
              ))}
            </select>
          </div>

          <button
            onClick={loadReport}
            disabled={loading}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Зарежда...' : 'Генерирай'}
          </button>

          <div className="flex-1"></div>

          <button
            onClick={exportToPDF}
            disabled={!reportData || loading}
            className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
          >
            <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            PDF
          </button>

          <button
            onClick={exportToExcel}
            disabled={!reportData || loading}
            className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
          >
            <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            Excel
          </button>
        </div>
      </div>

      {/* Report */}
      <div id="depreciation-report" className="bg-white rounded-lg shadow">
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-xl font-bold text-center">СПРАВКА ЗА АМОРТИЗАЦИИТЕ</h2>
          <p className="text-center text-gray-600 mt-2">
            {companyInfo?.name} - ЕИК: {companyInfo?.eik}
          </p>
          <p className="text-center text-gray-600">
            Период: {monthNames[month - 1]} {year}
          </p>
        </div>

        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Инв. №
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Наименование
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Категория
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                  Отчетна ст-ст
                </th>
                <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase">
                  Дата придоб.
                </th>
                <th colSpan="3" className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase bg-blue-50">
                  Счетоводна амортизация
                </th>
                <th colSpan="3" className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase bg-green-50">
                  Данъчна амортизация
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                  Разлика
                </th>
              </tr>
              <tr>
                <th colSpan="5"></th>
                <th className="px-4 py-2 text-right text-xs font-medium text-gray-500 bg-blue-50">
                  За периода
                </th>
                <th className="px-4 py-2 text-right text-xs font-medium text-gray-500 bg-blue-50">
                  Натрупана
                </th>
                <th className="px-4 py-2 text-right text-xs font-medium text-gray-500 bg-blue-50">
                  Балансова
                </th>
                <th className="px-4 py-2 text-right text-xs font-medium text-gray-500 bg-green-50">
                  За периода
                </th>
                <th className="px-4 py-2 text-right text-xs font-medium text-gray-500 bg-green-50">
                  Натрупана
                </th>
                <th className="px-4 py-2 text-right text-xs font-medium text-gray-500 bg-green-50">
                  Балансова
                </th>
                <th></th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {reportData?.map((entry, idx) => {
                const difference = parseFloat(entry.taxDepreciationAmount) - parseFloat(entry.accountingDepreciationAmount);
                return (
                  <tr key={entry.id} className={idx % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                    <td className="px-4 py-2 text-sm text-gray-900">
                      {entry.asset?.inventoryNumber}
                    </td>
                    <td className="px-4 py-2 text-sm text-gray-900">
                      {entry.asset?.name}
                    </td>
                    <td className="px-4 py-2 text-sm text-gray-600">
                      {entry.category?.name}
                    </td>
                    <td className="px-4 py-2 text-sm text-right text-gray-900">
                      {formatCurrency(entry.asset?.acquisitionCost)}
                    </td>
                    <td className="px-4 py-2 text-sm text-center text-gray-600">
                      {formatDate(entry.asset?.acquisitionDate)}
                    </td>
                    <td className="px-4 py-2 text-sm text-right text-gray-900 bg-blue-50/50">
                      {formatCurrency(entry.accountingDepreciationAmount)}
                    </td>
                    <td className="px-4 py-2 text-sm text-right text-gray-900 bg-blue-50/50">
                      {formatCurrency(entry.accountingAccumulatedDepreciation)}
                    </td>
                    <td className="px-4 py-2 text-sm text-right text-gray-900 bg-blue-50/50">
                      {formatCurrency(entry.accountingBookValueAfter)}
                    </td>
                    <td className="px-4 py-2 text-sm text-right text-gray-900 bg-green-50/50">
                      {formatCurrency(entry.taxDepreciationAmount)}
                    </td>
                    <td className="px-4 py-2 text-sm text-right text-gray-900 bg-green-50/50">
                      {formatCurrency(entry.taxAccumulatedDepreciation)}
                    </td>
                    <td className="px-4 py-2 text-sm text-right text-gray-900 bg-green-50/50">
                      {formatCurrency(entry.taxBookValueAfter)}
                    </td>
                    <td className={`px-4 py-2 text-sm text-right font-medium ${
                      difference > 0 ? 'text-green-600' : difference < 0 ? 'text-red-600' : 'text-gray-500'
                    }`}>
                      {difference !== 0 ? formatCurrency(Math.abs(difference)) : '-'}
                    </td>
                  </tr>
                );
              })}
            </tbody>
            <tfoot className="bg-gray-100">
              <tr className="font-bold">
                <td colSpan="3" className="px-4 py-3 text-sm">ОБЩО:</td>
                <td className="px-4 py-3 text-sm text-right">
                  {formatCurrency(totals.acquisitionCost)}
                </td>
                <td></td>
                <td className="px-4 py-3 text-sm text-right bg-blue-100">
                  {formatCurrency(totals.accountingDepreciation)}
                </td>
                <td className="px-4 py-3 text-sm text-right bg-blue-100"></td>
                <td className="px-4 py-3 text-sm text-right bg-blue-100">
                  {formatCurrency(totals.accountingBookValue)}
                </td>
                <td className="px-4 py-3 text-sm text-right bg-green-100">
                  {formatCurrency(totals.taxDepreciation)}
                </td>
                <td className="px-4 py-3 text-sm text-right bg-green-100"></td>
                <td className="px-4 py-3 text-sm text-right bg-green-100">
                  {formatCurrency(totals.taxBookValue)}
                </td>
                <td className="px-4 py-3 text-sm text-right">
                  {formatCurrency(totals.taxDepreciation - totals.accountingDepreciation)}
                </td>
              </tr>
            </tfoot>
          </table>
        </div>

        <div className="p-4 border-t border-gray-200 text-xs text-gray-500">
          <p>Дата на изготвяне: {new Date().toLocaleDateString('bg-BG')}</p>
        </div>
      </div>
    </div>
  );
}