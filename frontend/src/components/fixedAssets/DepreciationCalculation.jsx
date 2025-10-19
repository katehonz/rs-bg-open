import { useState, useEffect } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

export default function DepreciationCalculation({ onRefreshStats }) {
  const currentYear = new Date().getFullYear();
  const currentMonth = new Date().getMonth() + 1;

  const [year, setYear] = useState(currentYear);
  const [month, setMonth] = useState(currentMonth);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState(null);
  const [postingResult, setPostingResult] = useState(null);
  const [calculatedPeriods, setCalculatedPeriods] = useState([]);

  const CALCULATE_MUTATION = `
    mutation CalculateDepreciation($input: CalculateDepreciationInput!) {
      calculateDepreciation(input: $input) {
        success
        calculatedCount
        errorCount
        totalAccountingAmount
        totalTaxAmount
        errors
      }
    }
  `;

  const POST_MUTATION = `
    mutation PostDepreciation($input: PostDepreciationInput!, $userId: Int!) {
      postDepreciation(input: $input, userId: $userId) {
        success
        journalEntryId
        totalAmount
        assetsCount
        message
      }
    }
  `;

  const PERIODS_QUERY = `
    query CompanyCalculatedPeriods($companyId: Int!) {
      companyCalculatedPeriods(companyId: $companyId) {
        year
        month
        periodDisplay
        isPosted
      }
    }
  `;

  useEffect(() => {
    fetchCalculatedPeriods();
  }, []);

  const fetchCalculatedPeriods = async () => {
    try {
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      const response = await graphqlRequest(PERIODS_QUERY, { companyId });
      setCalculatedPeriods(response.companyCalculatedPeriods || []);
    } catch (err) {
      console.error('Error fetching calculated periods:', err);
    }
  };

  const handleCalculate = async () => {
    try {
      setLoading(true);
      setResult(null);
      setPostingResult(null);
      
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      const input = {
        companyId,
        year: parseInt(year),
        month: parseInt(month)
      };
      
      const response = await graphqlRequest(CALCULATE_MUTATION, { input });
      setResult(response.calculateDepreciation);

      if (response.calculateDepreciation.success) {
        onRefreshStats();
        fetchCalculatedPeriods(); // Refresh calculated periods
      }
    } catch (err) {
      alert('Грешка при изчисляване: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const handlePost = async () => {
    if (!confirm('Сигурни ли сте, че искате да приключите амортизацията в главния дневник?')) {
      return;
    }
    
    try {
      setLoading(true);
      
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      const userId = 1; // TODO: Get from auth context
      
      const input = {
        companyId,
        year: parseInt(year),
        month: parseInt(month),
        reference: `АМ-${year}-${String(month).padStart(2, '0')}`
      };
      
      const response = await graphqlRequest(POST_MUTATION, { input, userId });
      setPostingResult(response.postDepreciation);

      if (response.postDepreciation.success) {
        onRefreshStats();
        fetchCalculatedPeriods(); // Refresh calculated periods
      }
    } catch (err) {
      alert('Грешка при приключване: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      style: 'currency',
      currency: 'BGN'
    }).format(amount || 0);
  };

  const monthNames = [
    'Януари', 'Февруари', 'Март', 'Април', 'Май', 'Юни',
    'Юли', 'Август', 'Септември', 'Октомври', 'Ноември', 'Декември'
  ];

  return (
    <div className="space-y-6">
      {/* Calculated Periods Display */}
      {calculatedPeriods.length > 0 && (
        <div className="bg-blue-50 rounded-lg p-6">
          <h3 className="text-lg font-medium text-blue-900 mb-4">
            Изчислени и осчетоводени периоди
          </h3>
          <div className="flex flex-wrap gap-2">
            {calculatedPeriods.map((period, idx) => (
              <div
                key={idx}
                className={`px-3 py-1 rounded-md text-sm font-medium ${
                  period.isPosted
                    ? 'bg-green-100 text-green-800 border border-green-300'
                    : 'bg-yellow-100 text-yellow-800 border border-yellow-300'
                }`}
              >
                {period.periodDisplay}
                {period.isPosted && (
                  <span className="ml-1" title="Приключен">✓</span>
                )}
              </div>
            ))}
          </div>
          <div className="mt-3 text-sm text-blue-700">
            <p>• Зелени: Изчислени и приключени в дневника</p>
            <p>• Жълти: Изчислени, но неприключени</p>
          </div>
        </div>
      )}

      {/* Period Selection */}
      <div className="bg-gray-50 rounded-lg p-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">
          Изчисляване на месечна амортизация
        </h3>
        
        <div className="flex items-end gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Година
            </label>
            <select
              value={year}
              onChange={(e) => setYear(e.target.value)}
              className="block w-32 px-3 py-2 border border-gray-300 rounded-md"
            >
              {[currentYear - 1, currentYear, currentYear + 1].map(y => (
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
              onChange={(e) => setMonth(e.target.value)}
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
            onClick={handleCalculate}
            disabled={loading}
            className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Изчисляване...' : 'Изчисли амортизация'}
          </button>
        </div>
        
        <div className="mt-4 text-sm text-gray-600">
          <p>• Изчислява се месечна амортизация за всички активни ДМА</p>
          <p>• Отделно се изчислява счетоводна и данъчна амортизация</p>
          <p>• Актуализира балансовите стойности на активите</p>
        </div>
      </div>

      {/* Calculation Result */}
      {result && (
        <div className={`rounded-lg p-6 ${result.success ? 'bg-green-50' : 'bg-red-50'}`}>
          <h3 className={`text-lg font-medium ${result.success ? 'text-green-900' : 'text-red-900'} mb-4`}>
            {result.success ? 'Успешно изчисление' : 'Грешка при изчисление'}
          </h3>
          
          {result.success && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <p className="text-sm text-gray-600">Обработени активи</p>
                  <p className="text-2xl font-semibold text-gray-900">
                    {result.calculatedCount}
                  </p>
                </div>
                
                <div>
                  <p className="text-sm text-gray-600">Счетоводна амортизация</p>
                  <p className="text-xl font-semibold text-gray-900">
                    {formatCurrency(result.totalAccountingAmount)}
                  </p>
                </div>
                
                <div>
                  <p className="text-sm text-gray-600">Данъчна амортизация</p>
                  <p className="text-xl font-semibold text-gray-900">
                    {formatCurrency(result.totalTaxAmount)}
                  </p>
                </div>
                
                <div>
                  <p className="text-sm text-gray-600">Разлика</p>
                  <p className="text-xl font-semibold text-gray-900">
                    {formatCurrency(Math.abs(result.totalTaxAmount - result.totalAccountingAmount))}
                  </p>
                </div>
              </div>
              
              {result.errorCount > 0 && (
                <div className="mt-4 p-4 bg-yellow-100 rounded-md">
                  <p className="text-sm font-medium text-yellow-800">
                    Грешки при {result.errorCount} актива:
                  </p>
                  <ul className="mt-2 text-sm text-yellow-700">
                    {result.errors.map((error, idx) => (
                      <li key={idx}>• {error}</li>
                    ))}
                  </ul>
                </div>
              )}
              
              {!postingResult && (
                <div className="mt-6 flex items-center justify-between p-4 bg-white rounded-lg border border-green-200">
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      Приключване в главния дневник
                    </p>
                    <p className="text-sm text-gray-500 mt-1">
                      Създава журнален запис: Dt 603 Разходи за амортизация / Ct 241 Натрупана амортизация
                    </p>
                  </div>
                  <button
                    onClick={handlePost}
                    disabled={loading}
                    className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
                  >
                    {loading ? 'Приключване...' : 'Приключи в дневника'}
                  </button>
                </div>
              )}
            </div>
          )}
          
          {result.errors && result.errors.length > 0 && !result.success && (
            <div className="mt-4">
              <p className="text-sm font-medium text-red-800 mb-2">Грешки:</p>
              <ul className="text-sm text-red-700">
                {result.errors.map((error, idx) => (
                  <li key={idx}>• {error}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      {/* Posting Result */}
      {postingResult && (
        <div className="bg-blue-50 rounded-lg p-6">
          <h3 className="text-lg font-medium text-blue-900 mb-4">
            Успешно приключване
          </h3>
          
          <div className="space-y-3">
            <div className="flex items-center">
              <span className="text-sm text-gray-600 w-40">Журнален запис №:</span>
              <span className="font-medium text-gray-900">
                {postingResult.journalEntryId}
              </span>
            </div>
            
            <div className="flex items-center">
              <span className="text-sm text-gray-600 w-40">Обща сума:</span>
              <span className="font-medium text-gray-900">
                {formatCurrency(postingResult.totalAmount)}
              </span>
            </div>
            
            <div className="flex items-center">
              <span className="text-sm text-gray-600 w-40">Брой активи:</span>
              <span className="font-medium text-gray-900">
                {postingResult.assetsCount}
              </span>
            </div>
            
            <div className="mt-4 p-3 bg-white rounded-md">
              <p className="text-sm text-gray-700">
                {postingResult.message}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Information Box */}
      <div className="bg-gray-50 rounded-lg p-6">
        <h4 className="text-sm font-medium text-gray-900 mb-3">
          Информация за амортизацията
        </h4>
        <div className="space-y-2 text-sm text-gray-600">
          <p>
            <strong>Счетоводна амортизация:</strong> Изчислява се според счетоводната политика на фирмата.
            Може да бъде различна от данъчната норма.
          </p>
          <p>
            <strong>Данъчна амортизация:</strong> Изчислява се според максималните норми в ЗКПО.
            Използва се за данъчни цели.
          </p>
          <p>
            <strong>Счетоводна проводка:</strong> Дебит сметка 603 (Разходи за амортизация) / 
            Кредит сметка 241 (Натрупана амортизация на ДМА)
          </p>
          <p>
            <strong>Временни разлики:</strong> Разликата между счетоводната и данъчната амортизация 
            създава временни разлики за отсрочени данъци.
          </p>
        </div>
      </div>
    </div>
  );
}