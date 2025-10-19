import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';
import html2pdf from 'html2pdf.js';
import * as XLSX from 'xlsx';

export default function FixedAssetsReports() {
  const currentYear = new Date().getFullYear();
  const currentMonth = new Date().getMonth() + 1;

  const [reportType, setReportType] = useState('ac-dma'); // ac-dma, tax-dma, opis, sap, dap
  const [year, setYear] = useState(currentYear);
  const [month, setMonth] = useState(currentMonth);
  const [reportData, setReportData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [companyInfo, setCompanyInfo] = useState(null);

  const REPORTS_QUERY = `
    query GetReportsData($companyId: Int!) {
      fixedAssets(companyId: $companyId, status: "active") {
        id
        inventoryNumber
        name
        categoryId
        acquisitionCost
        acquisitionDate
        putIntoServiceDate
        accountingUsefulLife
        accountingDepreciationRate
        accountingAccumulatedDepreciation
        accountingBookValue
        taxDepreciationRate
        taxAccumulatedDepreciation
        taxBookValue
        status
        category {
          id
          name
          code
        }
      }

      fixedAssetCategories {
        id
        name
        code
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

  const reportTypes = [
    { id: 'ac-dma', name: 'Счетоводни амортизационни квоти' },
    { id: 'tax-dma', name: 'Данъчни амортизационни квоти' },
    { id: 'opis', name: 'Инвентаризационен опис' },
    { id: 'sap', name: 'Счетоводен амортизационен план (SAP)' },
    { id: 'dap', name: 'Данъчен амортизационен план (DAP)' }
  ];

  useEffect(() => {
    loadReportData();
  }, []);

  const loadReportData = async () => {
    try {
      setLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const response = await graphqlRequest(REPORTS_QUERY, { companyId });

      const company = response.companies?.find(c => c.id === companyId);
      setCompanyInfo(company);
      setReportData(response.fixedAssets || []);
    } catch (err) {
      console.error('Error loading report data:', err);
      alert('Грешка при зареждане на данните');
    } finally {
      setLoading(false);
    }
  };

  const calculateMonthlyDepreciation = (asset, isAccounting) => {
    const rate = isAccounting ? asset.accountingDepreciationRate : asset.taxDepreciationRate;
    const annualAmount = (asset.acquisitionCost * rate) / 100;
    return annualAmount / 12;
  };

  const calculateDepreciationSchedule = (asset, isAccounting) => {
    const monthly = calculateMonthlyDepreciation(asset, isAccounting);
    const startDate = new Date(asset.putIntoServiceDate || asset.acquisitionDate);
    const startYear = startDate.getFullYear();
    const startMonth = startDate.getMonth() + 1; // 1-based

    const rate = isAccounting ? asset.accountingDepreciationRate : asset.taxDepreciationRate;
    const totalMonths = Math.ceil((100 / rate) * 12);

    const schedule = [];
    let currentDate = new Date(startYear, startMonth - 1, 1);
    let remaining = asset.acquisitionCost;

    for (let i = 0; i < totalMonths && remaining > 0.01; i++) {
      const year = currentDate.getFullYear();
      const month = currentDate.getMonth() + 1;

      let amount = monthly;
      if (remaining < monthly) {
        amount = remaining;
      }

      schedule.push({ year, month, amount });
      remaining -= amount;

      currentDate.setMonth(currentDate.getMonth() + 1);
    }

    return schedule;
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount || 0);
  };

  const formatDate = (date) => {
    return new Date(date).toLocaleDateString('bg-BG');
  };

  const exportToPDF = () => {
    const element = document.getElementById('report-content');
    const reportName = reportTypes.find(r => r.id === reportType)?.name || 'справка';

    // Use A3 landscape for depreciation schedules (wider), A4 for others
    let format = 'a4';
    let orientation = 'portrait';

    if (reportType === 'ac-dma' || reportType === 'tax-dma') {
      format = 'a3'; // Wider page for monthly schedules
      orientation = 'landscape';
    } else if (reportType === 'sap' || reportType === 'dap') {
      format = 'a4';
      orientation = 'landscape';
    }

    const opt = {
      margin: [5, 5, 5, 5], // Smaller margins for more content
      filename: `${reportType}_${year}_${String(month).padStart(2, '0')}.pdf`,
      image: { type: 'jpeg', quality: 0.98 },
      html2canvas: {
        scale: 2,
        useCORS: true,
        letterRendering: true
      },
      jsPDF: {
        unit: 'mm',
        format: format,
        orientation: orientation,
        compress: true
      },
      pagebreak: { mode: ['avoid-all', 'css', 'legacy'] }
    };

    html2pdf().set(opt).from(element).save();
  };

  const exportToExcel = () => {
    if (!reportData) return;

    const wb = XLSX.utils.book_new();
    let wsData = [];

    switch (reportType) {
      case 'ac-dma':
        wsData = generateAccountingScheduleExcel();
        break;
      case 'tax-dma':
        wsData = generateTaxScheduleExcel();
        break;
      case 'opis':
        wsData = generateInventoryExcel();
        break;
      case 'sap':
        wsData = generateAccountingPlanExcel();
        break;
      case 'dap':
        wsData = generateTaxPlanExcel();
        break;
    }

    const ws = XLSX.utils.aoa_to_sheet(wsData);

    // Auto-size columns
    const colWidths = wsData[0]?.map((_, colIndex) => {
      const maxLength = Math.max(
        ...wsData.map(row => {
          const cell = row[colIndex];
          return cell ? cell.toString().length : 0;
        })
      );
      return { wch: Math.min(maxLength + 2, 50) };
    });
    ws['!cols'] = colWidths;

    XLSX.utils.book_append_sheet(wb, ws, reportTypes.find(r => r.id === reportType)?.name.substring(0, 31));
    XLSX.writeFile(wb, `${reportType}_${year}_${String(month).padStart(2, '0')}.xlsx`);
  };

  const generateAccountingScheduleExcel = () => {
    const data = [
      ['Счетоводни амортизационни квоти'],
      [`На фирма: ${companyInfo?.name}`],
      []
    ];

    reportData.forEach(asset => {
      const schedule = calculateDepreciationSchedule(asset, true);
      const yearGroups = {};

      schedule.forEach(entry => {
        if (!yearGroups[entry.year]) {
          yearGroups[entry.year] = Array(12).fill(0);
        }
        yearGroups[entry.year][entry.month - 1] = entry.amount;
      });

      data.push([
        'Инв. номер', 'Име', 'Дата на въвеждане', 'Година',
        'Януари', 'Февруари', 'Март', 'Април', 'Май', 'Юни',
        'Юли', 'Август', 'Септември', 'Октомври', 'Ноември', 'Декември', 'Общо'
      ]);

      Object.entries(yearGroups).forEach(([yearKey, months]) => {
        const yearTotal = months.reduce((sum, val) => sum + val, 0);
        data.push([
          asset.inventoryNumber,
          asset.name,
          formatDate(asset.putIntoServiceDate || asset.acquisitionDate),
          yearKey,
          ...months.map(m => formatCurrency(m)),
          formatCurrency(yearTotal)
        ]);
      });

      const total = schedule.reduce((sum, entry) => sum + entry.amount, 0);
      data.push(['', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', `Общо: ${formatCurrency(total)}`]);
      data.push([]);
    });

    const grandTotal = reportData.reduce((sum, asset) => {
      const schedule = calculateDepreciationSchedule(asset, true);
      return sum + schedule.reduce((s, e) => s + e.amount, 0);
    }, 0);
    data.push(['', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', `Общо за всички активи: ${formatCurrency(grandTotal)}`]);

    return data;
  };

  const generateTaxScheduleExcel = () => {
    const data = [
      ['Данъчни амортизационни квоти'],
      [`На фирма: ${companyInfo?.name}`],
      []
    ];

    reportData.forEach(asset => {
      const schedule = calculateDepreciationSchedule(asset, false);
      const yearGroups = {};

      schedule.forEach(entry => {
        if (!yearGroups[entry.year]) {
          yearGroups[entry.year] = Array(12).fill(0);
        }
        yearGroups[entry.year][entry.month - 1] = entry.amount;
      });

      data.push([
        'Инв. номер', 'Име', 'Дата на въвеждане', 'Година',
        'Януари', 'Февруари', 'Март', 'Април', 'Май', 'Юни',
        'Юли', 'Август', 'Септември', 'Октомври', 'Ноември', 'Декември', 'Общо'
      ]);

      Object.entries(yearGroups).forEach(([yearKey, months]) => {
        const yearTotal = months.reduce((sum, val) => sum + val, 0);
        data.push([
          asset.inventoryNumber,
          asset.name,
          formatDate(asset.putIntoServiceDate || asset.acquisitionDate),
          yearKey,
          ...months.map(m => formatCurrency(m)),
          formatCurrency(yearTotal)
        ]);
      });

      const total = schedule.reduce((sum, entry) => sum + entry.amount, 0);
      data.push(['', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', `Общо: ${formatCurrency(total)}`]);
      data.push([]);
    });

    const grandTotal = reportData.reduce((sum, asset) => {
      const schedule = calculateDepreciationSchedule(asset, false);
      return sum + schedule.reduce((s, e) => s + e.amount, 0);
    }, 0);
    data.push(['', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', `Общо за всички активи: ${formatCurrency(grandTotal)}`]);

    return data;
  };

  const generateInventoryExcel = () => {
    const data = [
      ['Инвентаризационен опис на ДМА'],
      [`На фирма: ${companyInfo?.name}`],
      [`Към дата: ${new Date().toLocaleDateString('bg-BG')}`],
      ['за всички дълготрайни активи'],
      [`Съгласно заповед No: от дата: ${new Date().toLocaleDateString('bg-BG')}`],
      [],
      [
        'Инв. номер', 'Наименование', 'Сметка', 'Отчетна стойност',
        'Начислена амортизация', 'Балансова стойност', 'Дата на въвеждане',
        'Налично при инвентаризация', 'Констатирано - Липса', 'Констатирано - Излишък', 'Забележка'
      ]
    ];

    reportData.forEach(asset => {
      data.push([
        asset.inventoryNumber,
        asset.name,
        asset.category?.code || '206',
        formatCurrency(asset.acquisitionCost),
        formatCurrency(asset.accountingAccumulatedDepreciation),
        formatCurrency(asset.accountingBookValue),
        formatDate(asset.acquisitionDate),
        '', '', '', ''
      ]);
    });

    const totalBookValue = reportData.reduce((sum, asset) => sum + parseFloat(asset.accountingBookValue), 0);
    data.push([]);
    data.push(['', '', '', '', '', `Общо: ${formatCurrency(totalBookValue)}`]);

    return data;
  };

  const generateAccountingPlanExcel = () => {
    const data = [
      ['Счетоводен амортизационен план'],
      [`На фирма: ${companyInfo?.name}`],
      [`Към месец: ${String(month).padStart(2, '0')}-${year}`],
      [],
      [
        'Инв. номер', 'Наименование', 'Сметка', 'Категории', 'Дата на въвеждане',
        'Дата на изхабяване', 'Години Месеци', 'Амортизуема стойност',
        'Начислени амортизации', 'Остатъчна стойност', 'Год. аморт. норма', 'Начислени за годината'
      ]
    ];

    reportData.forEach(asset => {
      const startDate = new Date(asset.putIntoServiceDate || asset.acquisitionDate);
      const yearsMonths = `${Math.floor(asset.accountingUsefulLife / 12)}г. ${asset.accountingUsefulLife % 12}м.`;
      const endDate = new Date(startDate);
      endDate.setMonth(endDate.getMonth() + asset.accountingUsefulLife);
      const yearlyDepreciation = (parseFloat(asset.acquisitionCost) * parseFloat(asset.accountingDepreciationRate)) / 100;

      data.push([
        asset.inventoryNumber,
        asset.name,
        asset.category?.code || '206',
        asset.category?.name,
        formatDate(asset.putIntoServiceDate || asset.acquisitionDate),
        formatDate(endDate),
        yearsMonths,
        formatCurrency(asset.acquisitionCost),
        formatCurrency(asset.accountingAccumulatedDepreciation),
        formatCurrency(asset.accountingBookValue),
        `${formatCurrency(asset.accountingDepreciationRate)}%`,
        formatCurrency(yearlyDepreciation)
      ]);
    });

    const totals = reportData.reduce((acc, asset) => ({
      acquisitionCost: acc.acquisitionCost + parseFloat(asset.acquisitionCost),
      accumulatedDepreciation: acc.accumulatedDepreciation + parseFloat(asset.accountingAccumulatedDepreciation),
      bookValue: acc.bookValue + parseFloat(asset.accountingBookValue),
      yearlyDepreciation: acc.yearlyDepreciation + (parseFloat(asset.acquisitionCost) * parseFloat(asset.accountingDepreciationRate) / 100)
    }), { acquisitionCost: 0, accumulatedDepreciation: 0, bookValue: 0, yearlyDepreciation: 0 });

    data.push([]);
    data.push([
      'Общо:', '', '', '', '', '', '',
      formatCurrency(totals.acquisitionCost),
      formatCurrency(totals.accumulatedDepreciation),
      formatCurrency(totals.bookValue),
      '',
      formatCurrency(totals.yearlyDepreciation)
    ]);

    return data;
  };

  const generateTaxPlanExcel = () => {
    const data = [
      ['Данъчен амортизационен план'],
      [`На фирма: ${companyInfo?.name}`],
      [`Към месец: ${String(month).padStart(2, '0')}-${year}`],
      [],
      [
        'Инв. номер', 'Наименование', 'Сметка', 'Категории', 'Дата на въвеждане',
        'Дата на изхабяване', 'Години Месеци', 'Амортизуема стойност',
        'Начислени амортизации', 'Остатъчна стойност', 'Год. аморт. норма', 'Начислени за годината'
      ]
    ];

    reportData.forEach(asset => {
      const startDate = new Date(asset.putIntoServiceDate || asset.acquisitionDate);
      const taxLife = Math.ceil((100 / parseFloat(asset.taxDepreciationRate)) * 12);
      const yearsMonths = `${Math.floor(taxLife / 12)}г. ${taxLife % 12}м.`;
      const endDate = new Date(startDate);
      endDate.setMonth(endDate.getMonth() + taxLife);
      const yearlyDepreciation = (parseFloat(asset.acquisitionCost) * parseFloat(asset.taxDepreciationRate)) / 100;

      data.push([
        asset.inventoryNumber,
        asset.name,
        asset.category?.code || '206',
        asset.category?.name,
        formatDate(asset.putIntoServiceDate || asset.acquisitionDate),
        formatDate(endDate),
        yearsMonths,
        formatCurrency(asset.acquisitionCost),
        formatCurrency(asset.taxAccumulatedDepreciation),
        formatCurrency(asset.taxBookValue),
        `${formatCurrency(asset.taxDepreciationRate)}%`,
        formatCurrency(yearlyDepreciation)
      ]);
    });

    const totals = reportData.reduce((acc, asset) => ({
      acquisitionCost: acc.acquisitionCost + parseFloat(asset.acquisitionCost),
      accumulatedDepreciation: acc.accumulatedDepreciation + parseFloat(asset.taxAccumulatedDepreciation),
      bookValue: acc.bookValue + parseFloat(asset.taxBookValue),
      yearlyDepreciation: acc.yearlyDepreciation + (parseFloat(asset.acquisitionCost) * parseFloat(asset.taxDepreciationRate) / 100)
    }), { acquisitionCost: 0, accumulatedDepreciation: 0, bookValue: 0, yearlyDepreciation: 0 });

    data.push([]);
    data.push([
      'Общо:', '', '', '', '', '', '',
      formatCurrency(totals.acquisitionCost),
      formatCurrency(totals.accumulatedDepreciation),
      formatCurrency(totals.bookValue),
      '',
      formatCurrency(totals.yearlyDepreciation)
    ]);

    return data;
  };

  const renderAccountingDepreciationSchedule = () => {
    if (!reportData) return null;

    return (
      <div id="report-content" className="bg-white p-8">
        <h2 className="text-xl font-bold text-center mb-2">Счетоводни амортизационни квоти</h2>
        <p className="text-center mb-1">На фирма: {companyInfo?.name}</p>
        <div className="space-y-8">
          {reportData.map(asset => {
            const schedule = calculateDepreciationSchedule(asset, true);
            const yearGroups = {};

            schedule.forEach(entry => {
              if (!yearGroups[entry.year]) {
                yearGroups[entry.year] = Array(12).fill(0);
              }
              yearGroups[entry.year][entry.month - 1] = entry.amount;
            });

            const total = schedule.reduce((sum, entry) => sum + entry.amount, 0);

            return (
              <div key={asset.id} className="mb-6">
                <table className="min-w-full border-collapse border border-gray-300 text-xs">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="border border-gray-300 px-2 py-1 text-left">Инв.<br/>номер</th>
                      <th className="border border-gray-300 px-2 py-1 text-left">Име</th>
                      <th className="border border-gray-300 px-2 py-1 text-left">Дата на<br/>въвеждане в<br/>експлоатация</th>
                      <th className="border border-gray-300 px-2 py-1">Година</th>
                      <th className="border border-gray-300 px-2 py-1">Януари</th>
                      <th className="border border-gray-300 px-2 py-1">Февруари</th>
                      <th className="border border-gray-300 px-2 py-1">Март</th>
                      <th className="border border-gray-300 px-2 py-1">Април</th>
                      <th className="border border-gray-300 px-2 py-1">Май</th>
                      <th className="border border-gray-300 px-2 py-1">Юни</th>
                      <th className="border border-gray-300 px-2 py-1">Юли</th>
                      <th className="border border-gray-300 px-2 py-1">Август</th>
                      <th className="border border-gray-300 px-2 py-1">Септември</th>
                      <th className="border border-gray-300 px-2 py-1">Октомври</th>
                      <th className="border border-gray-300 px-2 py-1">Ноември</th>
                      <th className="border border-gray-300 px-2 py-1">Декември</th>
                      <th className="border border-gray-300 px-2 py-1">Общо</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(yearGroups).map(([yearKey, months]) => {
                      const yearTotal = months.reduce((sum, val) => sum + val, 0);
                      return (
                        <tr key={`${asset.id}-${yearKey}`}>
                          <td className="border border-gray-300 px-2 py-1">{asset.inventoryNumber}</td>
                          <td className="border border-gray-300 px-2 py-1">{asset.name}</td>
                          <td className="border border-gray-300 px-2 py-1 text-center">
                            {formatDate(asset.putIntoServiceDate || asset.acquisitionDate)}
                          </td>
                          <td className="border border-gray-300 px-2 py-1 text-center">{yearKey}</td>
                          {months.map((amount, idx) => (
                            <td key={idx} className="border border-gray-300 px-2 py-1 text-right">
                              {formatCurrency(amount)}
                            </td>
                          ))}
                          <td className="border border-gray-300 px-2 py-1 text-right font-semibold">
                            {formatCurrency(yearTotal)}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
                <p className="text-right font-bold mt-1">Общо: {formatCurrency(total)}</p>
              </div>
            );
          })}

          <p className="text-right font-bold text-lg mt-4">
            Общо за всички активи: {formatCurrency(reportData.reduce((sum, asset) => {
              const schedule = calculateDepreciationSchedule(asset, true);
              return sum + schedule.reduce((s, e) => s + e.amount, 0);
            }, 0))}
          </p>
        </div>
      </div>
    );
  };

  const renderTaxDepreciationSchedule = () => {
    if (!reportData) return null;

    return (
      <div id="report-content" className="bg-white p-8">
        <h2 className="text-xl font-bold text-center mb-2">Данъчни амортизационни квоти</h2>
        <p className="text-center mb-1">На фирма: {companyInfo?.name}</p>
        <div className="space-y-8">
          {reportData.map(asset => {
            const schedule = calculateDepreciationSchedule(asset, false);
            const yearGroups = {};

            schedule.forEach(entry => {
              if (!yearGroups[entry.year]) {
                yearGroups[entry.year] = Array(12).fill(0);
              }
              yearGroups[entry.year][entry.month - 1] = entry.amount;
            });

            const total = schedule.reduce((sum, entry) => sum + entry.amount, 0);

            return (
              <div key={asset.id} className="mb-6">
                <table className="min-w-full border-collapse border border-gray-300 text-xs">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="border border-gray-300 px-2 py-1 text-left">Инв.<br/>номер</th>
                      <th className="border border-gray-300 px-2 py-1 text-left">Име</th>
                      <th className="border border-gray-300 px-2 py-1 text-left">Дата на<br/>въвеждане в<br/>експлоатация</th>
                      <th className="border border-gray-300 px-2 py-1">Година</th>
                      <th className="border border-gray-300 px-2 py-1">Януари</th>
                      <th className="border border-gray-300 px-2 py-1">Февруари</th>
                      <th className="border border-gray-300 px-2 py-1">Март</th>
                      <th className="border border-gray-300 px-2 py-1">Април</th>
                      <th className="border border-gray-300 px-2 py-1">Май</th>
                      <th className="border border-gray-300 px-2 py-1">Юни</th>
                      <th className="border border-gray-300 px-2 py-1">Юли</th>
                      <th className="border border-gray-300 px-2 py-1">Август</th>
                      <th className="border border-gray-300 px-2 py-1">Септември</th>
                      <th className="border border-gray-300 px-2 py-1">Октомври</th>
                      <th className="border border-gray-300 px-2 py-1">Ноември</th>
                      <th className="border border-gray-300 px-2 py-1">Декември</th>
                      <th className="border border-gray-300 px-2 py-1">Общо</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(yearGroups).map(([yearKey, months]) => {
                      const yearTotal = months.reduce((sum, val) => sum + val, 0);
                      return (
                        <tr key={`${asset.id}-${yearKey}`}>
                          <td className="border border-gray-300 px-2 py-1">{asset.inventoryNumber}</td>
                          <td className="border border-gray-300 px-2 py-1">{asset.name}</td>
                          <td className="border border-gray-300 px-2 py-1 text-center">
                            {formatDate(asset.putIntoServiceDate || asset.acquisitionDate)}
                          </td>
                          <td className="border border-gray-300 px-2 py-1 text-center">{yearKey}</td>
                          {months.map((amount, idx) => (
                            <td key={idx} className="border border-gray-300 px-2 py-1 text-right">
                              {formatCurrency(amount)}
                            </td>
                          ))}
                          <td className="border border-gray-300 px-2 py-1 text-right font-semibold">
                            {formatCurrency(yearTotal)}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
                <p className="text-right font-bold mt-1">Общо: {formatCurrency(total)}</p>
              </div>
            );
          })}

          <p className="text-right font-bold text-lg mt-4">
            Общо за всички активи: {formatCurrency(reportData.reduce((sum, asset) => {
              const schedule = calculateDepreciationSchedule(asset, false);
              return sum + schedule.reduce((s, e) => s + e.amount, 0);
            }, 0))}
          </p>
        </div>
      </div>
    );
  };

  const renderInventoryList = () => {
    if (!reportData) return null;

    const totalBookValue = reportData.reduce((sum, asset) => sum + parseFloat(asset.accountingBookValue), 0);

    return (
      <div id="report-content" className="bg-white p-8">
        <h2 className="text-xl font-bold text-center mb-2">Инвентаризационен опис на ДМА</h2>
        <p className="text-center mb-1">На фирма: {companyInfo?.name}</p>
        <p className="text-center mb-1">Към дата: {new Date().toLocaleDateString('bg-BG')}</p>
        <p className="text-sm mb-4">за всички дълготрайни активи</p>
        <p className="text-sm mb-4">Съгласно заповед No: от дата: {new Date().toLocaleDateString('bg-BG')}</p>

        <table className="min-w-full border-collapse border border-gray-300 text-xs">
          <thead className="bg-gray-50">
            <tr>
              <th className="border border-gray-300 px-2 py-1 text-left">Инв.<br/>номер</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Наименование</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Сметка</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Отчетна<br/>стойност</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Начислена<br/>амортизация</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Балансова<br/>стойност</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Дата на<br/>въвеждане</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Налично при<br/>инвентаризация</th>
              <th className="border border-gray-300 px-2 py-1 text-center" colSpan="2">Констатирано</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Забележка</th>
            </tr>
            <tr>
              <th colSpan="8"></th>
              <th className="border border-gray-300 px-2 py-1 text-center">Липса</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Излишък</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {reportData.map(asset => (
              <tr key={asset.id}>
                <td className="border border-gray-300 px-2 py-1">{asset.inventoryNumber}</td>
                <td className="border border-gray-300 px-2 py-1">{asset.name}</td>
                <td className="border border-gray-300 px-2 py-1">{asset.category?.code || '206'}</td>
                <td className="border border-gray-300 px-2 py-1 text-right">
                  {formatCurrency(asset.acquisitionCost)}
                </td>
                <td className="border border-gray-300 px-2 py-1 text-right">
                  {formatCurrency(asset.accountingAccumulatedDepreciation)}
                </td>
                <td className="border border-gray-300 px-2 py-1 text-right">
                  {formatCurrency(asset.accountingBookValue)}
                </td>
                <td className="border border-gray-300 px-2 py-1 text-center">
                  {formatDate(asset.acquisitionDate)}
                </td>
                <td className="border border-gray-300 px-2 py-1"></td>
                <td className="border border-gray-300 px-2 py-1"></td>
                <td className="border border-gray-300 px-2 py-1"></td>
                <td className="border border-gray-300 px-2 py-1"></td>
              </tr>
            ))}
          </tbody>
        </table>

        <p className="text-right font-bold mt-4">Общо: {formatCurrency(totalBookValue)}</p>

        <div className="mt-8 flex justify-between">
          <div>
            <p className="text-sm">Име, презиме, фамилия</p>
            <p className="text-sm mt-8">_____________________</p>
          </div>
          <div>
            <p className="text-sm">Длъжност:</p>
            <p className="text-sm mt-8">_____________________</p>
          </div>
          <div>
            <p className="text-sm">Подпис:</p>
            <p className="text-sm mt-8">_____________________</p>
          </div>
        </div>
      </div>
    );
  };

  const renderAccountingPlan = () => {
    if (!reportData) return null;

    const totals = reportData.reduce((acc, asset) => ({
      acquisitionCost: acc.acquisitionCost + parseFloat(asset.acquisitionCost),
      accumulatedDepreciation: acc.accumulatedDepreciation + parseFloat(asset.accountingAccumulatedDepreciation),
      bookValue: acc.bookValue + parseFloat(asset.accountingBookValue),
      yearlyDepreciation: acc.yearlyDepreciation + (parseFloat(asset.acquisitionCost) * parseFloat(asset.accountingDepreciationRate) / 100)
    }), { acquisitionCost: 0, accumulatedDepreciation: 0, bookValue: 0, yearlyDepreciation: 0 });

    return (
      <div id="report-content" className="bg-white p-8">
        <h2 className="text-xl font-bold text-center mb-2">Счетоводен амортизационен план</h2>
        <p className="text-center mb-1">На фирма: {companyInfo?.name}</p>
        <p className="text-center mb-4">Към месец: {String(month).padStart(2, '0')}-{year}</p>

        <table className="min-w-full border-collapse border border-gray-300 text-xs">
          <thead className="bg-blue-50">
            <tr>
              <th className="border border-gray-300 px-2 py-1 text-left">Инв.<br/>номер</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Наименование</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Сметка</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Категории</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Дата на<br/>въвеждане</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Дата на<br/>изхабяване</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Години<br/>Месеци</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Амортизуема<br/>стойност</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Начислени<br/>амортизации</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Остатъчна<br/>стойност</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Год.<br/>аморт.<br/>норма</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Начислени<br/>за<br/>годината</th>
            </tr>
          </thead>
          <tbody>
            {reportData.map(asset => {
              const startDate = new Date(asset.putIntoServiceDate || asset.acquisitionDate);
              const yearsMonths = `${Math.floor(asset.accountingUsefulLife / 12)}г. ${asset.accountingUsefulLife % 12}м.`;
              const endDate = new Date(startDate);
              endDate.setMonth(endDate.getMonth() + asset.accountingUsefulLife);

              const yearlyDepreciation = (parseFloat(asset.acquisitionCost) * parseFloat(asset.accountingDepreciationRate)) / 100;

              return (
                <tr key={asset.id}>
                  <td className="border border-gray-300 px-2 py-1">{asset.inventoryNumber}</td>
                  <td className="border border-gray-300 px-2 py-1">{asset.name}</td>
                  <td className="border border-gray-300 px-2 py-1">{asset.category?.code || '206'}</td>
                  <td className="border border-gray-300 px-2 py-1">{asset.category?.name}</td>
                  <td className="border border-gray-300 px-2 py-1 text-center">
                    {formatDate(asset.putIntoServiceDate || asset.acquisitionDate)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-center">
                    {formatDate(endDate)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-center">{yearsMonths}</td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(asset.acquisitionCost)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(asset.accountingAccumulatedDepreciation)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(asset.accountingBookValue)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-center">
                    {formatCurrency(asset.accountingDepreciationRate)}%
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(yearlyDepreciation)}
                  </td>
                </tr>
              );
            })}
          </tbody>
          <tfoot className="bg-gray-100 font-bold">
            <tr>
              <td colSpan="7" className="border border-gray-300 px-2 py-1">Общо:</td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.acquisitionCost)}
              </td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.accumulatedDepreciation)}
              </td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.bookValue)}
              </td>
              <td></td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.yearlyDepreciation)}
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    );
  };

  const renderTaxPlan = () => {
    if (!reportData) return null;

    const totals = reportData.reduce((acc, asset) => ({
      acquisitionCost: acc.acquisitionCost + parseFloat(asset.acquisitionCost),
      accumulatedDepreciation: acc.accumulatedDepreciation + parseFloat(asset.taxAccumulatedDepreciation),
      bookValue: acc.bookValue + parseFloat(asset.taxBookValue),
      yearlyDepreciation: acc.yearlyDepreciation + (parseFloat(asset.acquisitionCost) * parseFloat(asset.taxDepreciationRate) / 100)
    }), { acquisitionCost: 0, accumulatedDepreciation: 0, bookValue: 0, yearlyDepreciation: 0 });

    return (
      <div id="report-content" className="bg-white p-8">
        <h2 className="text-xl font-bold text-center mb-2">Данъчен амортизационен план</h2>
        <p className="text-center mb-1">На фирма: {companyInfo?.name}</p>
        <p className="text-center mb-4">Към месец: {String(month).padStart(2, '0')}-{year}</p>

        <table className="min-w-full border-collapse border border-gray-300 text-xs">
          <thead className="bg-green-50">
            <tr>
              <th className="border border-gray-300 px-2 py-1 text-left">Инв.<br/>номер</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Наименование</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Сметка</th>
              <th className="border border-gray-300 px-2 py-1 text-left">Категории</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Дата на<br/>въвеждане</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Дата на<br/>изхабяване</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Години<br/>Месеци</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Амортизуема<br/>стойност</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Начислени<br/>амортизации</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Остатъчна<br/>стойност</th>
              <th className="border border-gray-300 px-2 py-1 text-center">Год.<br/>аморт.<br/>норма</th>
              <th className="border border-gray-300 px-2 py-1 text-right">Начислени<br/>за<br/>годината</th>
            </tr>
          </thead>
          <tbody>
            {reportData.map(asset => {
              const startDate = new Date(asset.putIntoServiceDate || asset.acquisitionDate);
              const taxLife = Math.ceil((100 / parseFloat(asset.taxDepreciationRate)) * 12);
              const yearsMonths = `${Math.floor(taxLife / 12)}г. ${taxLife % 12}м.`;
              const endDate = new Date(startDate);
              endDate.setMonth(endDate.getMonth() + taxLife);

              const yearlyDepreciation = (parseFloat(asset.acquisitionCost) * parseFloat(asset.taxDepreciationRate)) / 100;

              return (
                <tr key={asset.id}>
                  <td className="border border-gray-300 px-2 py-1">{asset.inventoryNumber}</td>
                  <td className="border border-gray-300 px-2 py-1">{asset.name}</td>
                  <td className="border border-gray-300 px-2 py-1">{asset.category?.code || '206'}</td>
                  <td className="border border-gray-300 px-2 py-1">{asset.category?.name}</td>
                  <td className="border border-gray-300 px-2 py-1 text-center">
                    {formatDate(asset.putIntoServiceDate || asset.acquisitionDate)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-center">
                    {formatDate(endDate)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-center">{yearsMonths}</td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(asset.acquisitionCost)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(asset.taxAccumulatedDepreciation)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(asset.taxBookValue)}
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-center">
                    {formatCurrency(asset.taxDepreciationRate)}%
                  </td>
                  <td className="border border-gray-300 px-2 py-1 text-right">
                    {formatCurrency(yearlyDepreciation)}
                  </td>
                </tr>
              );
            })}
          </tbody>
          <tfoot className="bg-gray-100 font-bold">
            <tr>
              <td colSpan="7" className="border border-gray-300 px-2 py-1">Общо:</td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.acquisitionCost)}
              </td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.accumulatedDepreciation)}
              </td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.bookValue)}
              </td>
              <td></td>
              <td className="border border-gray-300 px-2 py-1 text-right">
                {formatCurrency(totals.yearlyDepreciation)}
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    );
  };

  const renderReport = () => {
    switch (reportType) {
      case 'ac-dma':
        return renderAccountingDepreciationSchedule();
      case 'tax-dma':
        return renderTaxDepreciationSchedule();
      case 'opis':
        return renderInventoryList();
      case 'sap':
        return renderAccountingPlan();
      case 'dap':
        return renderTaxPlan();
      default:
        return null;
    }
  };

  return (
    <div className="space-y-6">
      {/* Controls */}
      <div className="bg-white rounded-lg shadow p-4">
        <div className="flex items-end gap-4 flex-wrap">
          <div className="flex-1 min-w-[250px]">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Тип справка
            </label>
            <select
              value={reportType}
              onChange={(e) => setReportType(e.target.value)}
              className="block w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              {reportTypes.map(type => (
                <option key={type.id} value={type.id}>{type.name}</option>
              ))}
            </select>
          </div>

          {(reportType === 'sap' || reportType === 'dap') && (
            <>
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
            </>
          )}

          <button
            onClick={loadReportData}
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
            Изтегли PDF
          </button>

          <button
            onClick={exportToExcel}
            disabled={!reportData || loading}
            className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
          >
            <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            Изтегли Excel
          </button>
        </div>
      </div>

      {/* Report Content */}
      <div className="bg-white rounded-lg shadow overflow-auto">
        {loading ? (
          <div className="flex items-center justify-center h-64">
            <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
          </div>
        ) : (
          renderReport()
        )}
      </div>
    </div>
  );
}
