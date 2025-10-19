import { useEffect, useState } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

const MONTHLY_STATS_QUERY = `
  query MonthlyTransactionStats($companyId: Int!, $fromYear: Int!, $fromMonth: Int!, $toYear: Int!, $toMonth: Int!) {
    monthlyTransactionStats(
      input: {
        companyId: $companyId
        fromYear: $fromYear
        fromMonth: $fromMonth
        toYear: $toYear
        toMonth: $toMonth
      }
    ) {
      year
      month
      monthName
      totalEntries
      postedEntries
      totalEntryLines
      postedEntryLines
      totalAmount
      vatAmount
    }
  }
`;

const EXPORT_MONTHLY_STATS_MUTATION = `
  mutation ExportMonthlyStats($companyId: Int!, $fromYear: Int!, $fromMonth: Int!, $toYear: Int!, $toMonth: Int!) {
    exportMonthlyStats(
      input: {
        companyId: $companyId
        fromYear: $fromYear
        fromMonth: $fromMonth
        toYear: $toYear
        toMonth: $toMonth
      }
    ) {
      format
      content
      filename
      mimeType
    }
  }
`;

function formatCurrency(amount) {
  const value = Number(amount) || 0;
  return new Intl.NumberFormat('bg-BG', {
    style: 'currency',
    currency: 'BGN',
    maximumFractionDigits: 2,
  }).format(value);
}

export default function MonthlyStats() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [stats, setStats] = useState([]);
  const [exporting, setExporting] = useState(false);

  // Default to last 12 months
  const now = new Date();
  const [fromYear, setFromYear] = useState(now.getFullYear() - 1);
  const [fromMonth, setFromMonth] = useState(now.getMonth() + 1);
  const [toYear, setToYear] = useState(now.getFullYear());
  const [toMonth, setToMonth] = useState(now.getMonth() + 1);

  useEffect(() => {
    fetchStats();
  }, [companyId]);

  const fetchStats = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await graphqlRequest(MONTHLY_STATS_QUERY, {
        companyId,
        fromYear,
        fromMonth,
        toYear,
        toMonth,
      });

      setStats(response.monthlyTransactionStats || []);
    } catch (err) {
      console.error('Error loading monthly stats:', err);
      setError(err.message || 'Възникна грешка при зареждане на данните.');
    } finally {
      setLoading(false);
    }
  };

  const handleExportPDF = async () => {
    setExporting(true);
    try {
      const response = await graphqlRequest(EXPORT_MONTHLY_STATS_MUTATION, {
        companyId,
        fromYear,
        fromMonth,
        toYear,
        toMonth,
      });

      const { content, filename, mimeType } = response.exportMonthlyStats;

      // Decode base64 and create download link
      const byteCharacters = atob(content);
      const byteNumbers = new Array(byteCharacters.length);
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i);
      }
      const byteArray = new Uint8Array(byteNumbers);
      const blob = new Blob([byteArray], { type: mimeType });

      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Error exporting PDF:', err);
      setError('Грешка при експортиране на PDF файла.');
    } finally {
      setExporting(false);
    }
  };

  // Calculate totals
  const totals = stats.reduce(
    (acc, s) => ({
      totalEntries: acc.totalEntries + s.totalEntries,
      postedEntries: acc.postedEntries + s.postedEntries,
      totalEntryLines: acc.totalEntryLines + s.totalEntryLines,
      postedEntryLines: acc.postedEntryLines + s.postedEntryLines,
      totalAmount: acc.totalAmount + Number(s.totalAmount || 0),
      vatAmount: acc.vatAmount + Number(s.vatAmount || 0),
    }),
    {
      totalEntries: 0,
      postedEntries: 0,
      totalEntryLines: 0,
      postedEntryLines: 0,
      totalAmount: 0,
      vatAmount: 0,
    }
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Месечна статистика на транзакции</h1>
          <p className="mt-1 text-sm text-gray-500">
            Справка за ценообразуване - групирани данни по месеци
          </p>
        </div>
        <button
          onClick={handleExportPDF}
          disabled={exporting || stats.length === 0}
          className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
        >
          {exporting ? 'Експортиране...' : 'Експорт PDF'}
        </button>
      </div>

      {/* Filter form */}
      <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700">От година</label>
            <input
              type="number"
              value={fromYear}
              onChange={(e) => setFromYear(parseInt(e.target.value))}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700">От месец</label>
            <select
              value={fromMonth}
              onChange={(e) => setFromMonth(parseInt(e.target.value))}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
            >
              {Array.from({ length: 12 }, (_, i) => i + 1).map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700">До година</label>
            <input
              type="number"
              value={toYear}
              onChange={(e) => setToYear(parseInt(e.target.value))}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700">До месец</label>
            <select
              value={toMonth}
              onChange={(e) => setToMonth(parseInt(e.target.value))}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
            >
              {Array.from({ length: 12 }, (_, i) => i + 1).map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </div>
        </div>
        <div className="mt-4">
          <button
            onClick={fetchStats}
            disabled={loading}
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Зареждане...' : 'Обнови'}
          </button>
        </div>
      </div>

      {/* Pricing info */}
      <div className="bg-blue-50 border-l-4 border-blue-400 p-4">
        <div className="flex">
          <div className="flex-1">
            <p className="text-sm text-blue-700">
              <strong>Модел на ценообразуване:</strong> Базова цена (ангажимент) + допълнително
              заплащане по брой транзакции и счетоводни редове (Дт/Кт).
            </p>
            <p className="mt-1 text-xs text-blue-600">
              Важно: Отчитат се само приключени (posted) записи.
            </p>
          </div>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-700">
          ⚠️ {error}
        </div>
      )}

      {/* Table */}
      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full" />
        </div>
      ) : stats.length === 0 ? (
        <div className="bg-white p-12 rounded-lg shadow-sm border border-gray-200 text-center">
          <p className="text-gray-500">Няма данни за избрания период.</p>
        </div>
      ) : (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Период
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Документи<br />(общо)
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Документи<br />(приключени)
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Редове Дт/Кт<br />(общо)
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Редове Дт/Кт<br />(приключени)
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Оборот
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    ДДС
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {stats.map((stat, index) => (
                  <tr key={index} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {stat.monthName} {stat.year}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                      {stat.totalEntries}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                      {stat.postedEntries}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                      {stat.totalEntryLines}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                      {stat.postedEntryLines}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                      {formatCurrency(stat.totalAmount)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                      {formatCurrency(stat.vatAmount)}
                    </td>
                  </tr>
                ))}
                <tr className="bg-blue-50 font-bold">
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">ОБЩО</td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                    {totals.totalEntries}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                    {totals.postedEntries}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                    {totals.totalEntryLines}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                    {totals.postedEntryLines}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                    {formatCurrency(totals.totalAmount)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 font-mono">
                    {formatCurrency(totals.vatAmount)}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
