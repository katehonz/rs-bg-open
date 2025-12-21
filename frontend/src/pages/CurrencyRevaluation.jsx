import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';
import AccountSelectModal from '../components/AccountSelectModal';

// GraphQL Queries
const GET_REVALUATION_SETTINGS = `
  query GetRevaluationSettings($companyId: Int!) {
    revaluationSettings(companyId: $companyId) {
      id
      companyId
      expenseAccountId
      revenueAccountId
    }
  }
`;

const SAVE_REVALUATION_SETTINGS = `
  mutation SaveRevaluationSettings($input: RevaluationSettingsInput!) {
    saveRevaluationSettings(input: $input) {
      id
      companyId
      expenseAccountId
      revenueAccountId
    }
  }
`;

const GET_ACCOUNTS = `
  query GetAccounts($companyId: Int!) {
    accountHierarchy(companyId: $companyId) {
      id
      code
      name
      accountType
      accountClass
      isActive
      isAnalytical
    }
  }
`;

const GET_EXCHANGE_RATES = `
  query GetExchangeRatesForDate($date: NaiveDate!) {
    exchangeRatesWithCurrencies(date: $date) {
      exchangeRate {
        id
        rate
        validDate
        rateSource
      }
      fromCurrency {
        code
        nameBg
      }
      isUpToDate
      ageDescription
    }
  }
`;

const GET_REVALUATION_PREVIEW = `
  query GetRevaluationPreview($companyId: Int!, $revaluationDate: NaiveDate!) {
    currencyRevaluationPreview(companyId: $companyId, revaluationDate: $revaluationDate) {
      items {
        accountId
        accountCode
        accountName
        counterpartId
        counterpartName
        currencyCode
        foreignQuantity
        debitBgn
        creditBgn
        balanceBgn
        weightedAvgRate
        newRate
        newBalanceBgn
        revaluationDifference
      }
      totalItems
      totalGain
      totalLoss
      netDifference
    }
  }
`;

const RUN_REVALUATION = `
  mutation RunCurrencyRevaluation($companyId: Int!, $revaluationDate: NaiveDate!) {
    runCurrencyRevaluation(companyId: $companyId, revaluationDate: $revaluationDate) {
      success
      message
      createdEntriesCount
      totalRevaluationAmount
    }
  }
`;

export default function CurrencyRevaluation() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [success, setSuccess] = useState(null);
  const [accounts, setAccounts] = useState([]);
  const [settings, setSettings] = useState({
    expenseAccountId: null,
    revenueAccountId: null
  });
  const [expenseAccount, setExpenseAccount] = useState(null);
  const [revenueAccount, setRevenueAccount] = useState(null);

  // Revaluation execution state
  const [revaluationDate, setRevaluationDate] = useState(new Date().toISOString().split('T')[0]);
  const [exchangeRates, setExchangeRates] = useState([]);
  const [running, setRunning] = useState(false);

  // Preview state
  const [preview, setPreview] = useState(null);
  const [loadingPreview, setLoadingPreview] = useState(false);

  // Modal states
  const [showAccountModal, setShowAccountModal] = useState(false);
  const [accountModalType, setAccountModalType] = useState(null); // 'expense' or 'revenue'

  useEffect(() => {
    loadData();
  }, [companyId]);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      // Load accounts
      const accountsData = await graphqlRequest(GET_ACCOUNTS, { companyId });
      const activeAccounts = (accountsData.accountHierarchy || []).filter(a => a.isActive);
      setAccounts(activeAccounts);

      // Load settings
      try {
        const settingsData = await graphqlRequest(GET_REVALUATION_SETTINGS, { companyId });
        if (settingsData.revaluationSettings) {
          setSettings({
            expenseAccountId: settingsData.revaluationSettings.expenseAccountId,
            revenueAccountId: settingsData.revaluationSettings.revenueAccountId
          });

          // Find account objects from loaded accounts
          const expenseAcc = activeAccounts.find(a => a.id === settingsData.revaluationSettings.expenseAccountId);
          const revenueAcc = activeAccounts.find(a => a.id === settingsData.revaluationSettings.revenueAccountId);

          if (expenseAcc) {
            setExpenseAccount(expenseAcc);
          }
          if (revenueAcc) {
            setRevenueAccount(revenueAcc);
          }
        }
      } catch (err) {
        // Settings might not exist yet - that's OK
        console.log('No revaluation settings found, will create new');
      }

      // Load exchange rates for today
      await loadExchangeRates(revaluationDate);
    } catch (err) {
      setError('Грешка при зареждане: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const loadExchangeRates = async (date) => {
    try {
      const ratesData = await graphqlRequest(GET_EXCHANGE_RATES, { date });
      setExchangeRates(ratesData.exchangeRatesWithCurrencies || []);
    } catch (err) {
      console.error('Грешка при зареждане на курсове:', err);
    }
  };

  const loadPreview = async () => {
    if (!settings.expenseAccountId || !settings.revenueAccountId) {
      setError('Моля първо настройте сметките за разходи и приходи!');
      return;
    }

    setLoadingPreview(true);
    setError(null);
    try {
      const previewData = await graphqlRequest(GET_REVALUATION_PREVIEW, {
        companyId,
        revaluationDate
      });
      setPreview(previewData.currencyRevaluationPreview);
    } catch (err) {
      setError('Грешка при зареждане на справка: ' + err.message);
    } finally {
      setLoadingPreview(false);
    }
  };

  const handleSaveSettings = async () => {
    if (!settings.expenseAccountId || !settings.revenueAccountId) {
      setError('Моля изберете и двете сметки (разходи и приходи)!');
      return;
    }

    try {
      setError(null);
      await graphqlRequest(SAVE_REVALUATION_SETTINGS, {
        input: {
          companyId,
          expenseAccountId: settings.expenseAccountId,
          revenueAccountId: settings.revenueAccountId
        }
      });
      setSuccess('Настройките са запазени успешно!');
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError('Грешка при запазване: ' + err.message);
    }
  };

  const handleAccountSelect = (account) => {
    if (accountModalType === 'expense') {
      setSettings({ ...settings, expenseAccountId: account.id });
      setExpenseAccount(account);
    } else if (accountModalType === 'revenue') {
      setSettings({ ...settings, revenueAccountId: account.id });
      setRevenueAccount(account);
    }
    setShowAccountModal(false);
    setAccountModalType(null);
  };

  const handleRunRevaluation = async () => {
    if (!settings.expenseAccountId || !settings.revenueAccountId) {
      setError('Моля първо настройте сметките за разходи и приходи!');
      return;
    }

    if (!window.confirm(
      `Сигурни ли сте, че искате да стартирате валутна преоценка за ${revaluationDate}?\n\n` +
      `Това ще създаде автоматични записи за всички отворени позиции в чужда валута.`
    )) {
      return;
    }

    try {
      setRunning(true);
      setError(null);
      setSuccess(null);

      const result = await graphqlRequest(RUN_REVALUATION, {
        companyId,
        revaluationDate
      });

      if (result.runCurrencyRevaluation.success) {
        setSuccess(
          `Преоценката е завършена успешно!\n` +
          `Създадени записи: ${result.runCurrencyRevaluation.createdEntriesCount}\n` +
          `Обща валутна разлика: ${result.runCurrencyRevaluation.totalRevaluationAmount.toFixed(2)} лв.`
        );
      } else {
        setError(result.runCurrencyRevaluation.message);
      }
    } catch (err) {
      setError('Грешка при изпълнение на преоценка: ' + err.message);
    } finally {
      setRunning(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
        <span className="ml-2 text-gray-600">Зарежда се...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Валутна преоценка</h1>
          <p className="mt-1 text-sm text-gray-500">
            Автоматична преоценка на вземания и задължения в чужда валута
          </p>
        </div>
      </div>

      {/* Messages */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center">
            <div className="text-red-400 mr-3">⚠️</div>
            <div className="text-sm text-red-800 whitespace-pre-line">{error}</div>
          </div>
        </div>
      )}

      {success && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex items-center">
            <div className="text-green-400 mr-3">✓</div>
            <div className="text-sm text-green-800 whitespace-pre-line">{success}</div>
          </div>
        </div>
      )}

      {/* Settings Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 flex items-center">
            <span className="w-2 h-2 bg-blue-400 rounded-full mr-2"></span>
            Настройки
          </h3>
          <button
            onClick={handleSaveSettings}
            className="px-4 py-2 bg-blue-600 text-white rounded-md text-sm hover:bg-blue-700"
          >
            Запази настройки
          </button>
        </div>

        <div className="space-y-4">
          {/* Expense Account */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Сметка 624 - Разходи по валутни операции *
            </label>
            {expenseAccount ? (
              <div className="flex items-center justify-between p-3 border border-gray-300 rounded-md bg-gray-50">
                <div>
                  <div className="font-mono font-medium text-gray-900">{expenseAccount.code}</div>
                  <div className="text-sm text-gray-600">{expenseAccount.name}</div>
                </div>
                <button
                  onClick={() => {
                    setExpenseAccount(null);
                    setSettings({ ...settings, expenseAccountId: null });
                  }}
                  className="text-red-600 hover:text-red-800"
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            ) : (
              <button
                onClick={() => {
                  setAccountModalType('expense');
                  setShowAccountModal(true);
                }}
                className="w-full p-3 border border-gray-300 border-dashed rounded-md text-gray-500 hover:border-gray-400 hover:text-gray-600 text-left"
              >
                Избери сметка 624...
              </button>
            )}
          </div>

          {/* Revenue Account */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Сметка 724 - Приходи от валутни операции *
            </label>
            {revenueAccount ? (
              <div className="flex items-center justify-between p-3 border border-gray-300 rounded-md bg-gray-50">
                <div>
                  <div className="font-mono font-medium text-gray-900">{revenueAccount.code}</div>
                  <div className="text-sm text-gray-600">{revenueAccount.name}</div>
                </div>
                <button
                  onClick={() => {
                    setRevenueAccount(null);
                    setSettings({ ...settings, revenueAccountId: null });
                  }}
                  className="text-red-600 hover:text-red-800"
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            ) : (
              <button
                onClick={() => {
                  setAccountModalType('revenue');
                  setShowAccountModal(true);
                }}
                className="w-full p-3 border border-gray-300 border-dashed rounded-md text-gray-500 hover:border-gray-400 hover:text-gray-600 text-left"
              >
                Избери сметка 724...
              </button>
            )}
          </div>
        </div>

        <div className="mt-4 p-4 bg-blue-50 rounded-md">
          <p className="text-sm text-blue-800">
            <strong>Важно:</strong> Сметките 624 и 724 се използват за автоматично генериране на записи за валутни разлики при преоценка на вземания и задължения.
          </p>
        </div>
      </div>

      {/* Exchange Rates Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <span className="w-2 h-2 bg-green-400 rounded-full mr-2"></span>
          Валутни курсове
        </h3>

        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Дата на преоценка
          </label>
          <input
            type="date"
            value={revaluationDate}
            onChange={(e) => {
              setRevaluationDate(e.target.value);
              loadExchangeRates(e.target.value);
            }}
            className="px-3 py-2 border border-gray-300 rounded-md text-sm"
          />
        </div>

        {exchangeRates.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Валута</th>
                  <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">Курс към BGN</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Дата</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Статус</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {exchangeRates.map((rate, idx) => (
                  <tr key={idx}>
                    <td className="px-4 py-3 text-sm font-medium text-gray-900">
                      {rate.fromCurrency.code} - {rate.fromCurrency.nameBg}
                    </td>
                    <td className="px-4 py-3 text-sm text-right font-mono">
                      {parseFloat(rate.exchangeRate.rate).toFixed(4)}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-600">
                      {rate.exchangeRate.validDate}
                    </td>
                    <td className="px-4 py-3 text-sm">
                      {rate.isUpToDate ? (
                        <span className="px-2 py-1 bg-green-100 text-green-800 rounded text-xs">Актуален</span>
                      ) : (
                        <span className="px-2 py-1 bg-yellow-100 text-yellow-800 rounded text-xs">
                          {rate.ageDescription}
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-sm text-gray-500">Няма налични валутни курсове за избраната дата</p>
        )}
      </div>

      {/* Preview Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <span className="w-2 h-2 bg-blue-400 rounded-full mr-2"></span>
          Справка за преоценка
        </h3>

        <div className="space-y-4">
          <div className="p-4 bg-gray-50 rounded-md">
            <h4 className="font-medium text-gray-900 mb-2">Какво показва справката?</h4>
            <ul className="text-sm text-gray-600 space-y-1 list-disc list-inside">
              <li>Всички сметки с валутни салда (каса, банки, клиенти, доставчици, заеми)</li>
              <li>Салдо във валута и стойността в BGN по среден курс</li>
              <li>Изчислена стойност по нов курс към дата на преоценка</li>
              <li>Валутна разлика за преоценка (приход или разход)</li>
            </ul>
          </div>

          <div className="flex items-center justify-end">
            <button
              onClick={loadPreview}
              disabled={loadingPreview || !settings.expenseAccountId || !settings.revenueAccountId}
              className="px-6 py-3 bg-blue-600 text-white rounded-md font-medium hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed flex items-center"
            >
              {loadingPreview ? (
                <>
                  <div className="animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full mr-2"></div>
                  Зарежда се...
                </>
              ) : (
                <>
                  <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  Покажи справка
                </>
              )}
            </button>
          </div>

          {/* Preview Table */}
          {preview && preview.items && preview.items.length > 0 && (
            <div className="mt-6">
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">Сметка</th>
                      <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">Контрагент</th>
                      <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">Валута</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Количество</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Дебит (BGN)</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Кредит (BGN)</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Стойност (BGN)</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Среден курс</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Нов курс</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Нова стойност</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase">Разлика</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {preview.items.map((item, idx) => (
                      <tr key={idx} className="hover:bg-gray-50">
                        <td className="px-3 py-2 text-sm">
                          <div className="font-mono font-medium text-gray-900">{item.accountCode}</div>
                          <div className="text-xs text-gray-500">{item.accountName}</div>
                        </td>
                        <td className="px-3 py-2 text-sm text-gray-600">
                          {item.counterpartName || '-'}
                        </td>
                        <td className="px-3 py-2 text-sm font-medium text-gray-900">{item.currencyCode}</td>
                        <td className="px-3 py-2 text-sm text-right font-mono">{item.foreignQuantity.toFixed(2)}</td>
                        <td className="px-3 py-2 text-sm text-right font-mono">{item.debitBgn.toFixed(2)}</td>
                        <td className="px-3 py-2 text-sm text-right font-mono">{item.creditBgn.toFixed(2)}</td>
                        <td className="px-3 py-2 text-sm text-right font-mono font-medium">{item.balanceBgn.toFixed(2)}</td>
                        <td className="px-3 py-2 text-sm text-right font-mono">{item.weightedAvgRate.toFixed(4)}</td>
                        <td className="px-3 py-2 text-sm text-right font-mono">{item.newRate.toFixed(4)}</td>
                        <td className="px-3 py-2 text-sm text-right font-mono font-medium">{item.newBalanceBgn.toFixed(2)}</td>
                        <td className={`px-3 py-2 text-sm text-right font-mono font-bold ${item.revaluationDifference > 0 ? 'text-green-600' : item.revaluationDifference < 0 ? 'text-red-600' : 'text-gray-600'}`}>
                          {item.revaluationDifference > 0 ? '+' : ''}{item.revaluationDifference.toFixed(2)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                  <tfoot className="bg-gray-100 font-semibold">
                    <tr>
                      <td colSpan="10" className="px-3 py-2 text-sm text-right">Общо печалби:</td>
                      <td className="px-3 py-2 text-sm text-right font-mono text-green-600">+{preview.totalGain.toFixed(2)}</td>
                    </tr>
                    <tr>
                      <td colSpan="10" className="px-3 py-2 text-sm text-right">Общо загуби:</td>
                      <td className="px-3 py-2 text-sm text-right font-mono text-red-600">-{preview.totalLoss.toFixed(2)}</td>
                    </tr>
                    <tr className="border-t-2 border-gray-300">
                      <td colSpan="10" className="px-3 py-2 text-sm text-right font-bold">Нетна разлика:</td>
                      <td className={`px-3 py-2 text-sm text-right font-mono font-bold ${preview.netDifference > 0 ? 'text-green-600' : preview.netDifference < 0 ? 'text-red-600' : 'text-gray-900'}`}>
                        {preview.netDifference > 0 ? '+' : ''}{preview.netDifference.toFixed(2)}
                      </td>
                    </tr>
                  </tfoot>
                </table>
              </div>
            </div>
          )}

          {preview && preview.items && preview.items.length === 0 && (
            <div className="mt-4 p-4 bg-yellow-50 border border-yellow-200 rounded-md">
              <p className="text-sm text-yellow-800">Няма отворени позиции във валута за преоценка.</p>
            </div>
          )}
        </div>
      </div>

      {/* Run Revaluation Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <span className="w-2 h-2 bg-purple-400 rounded-full mr-2"></span>
          Изпълнение на преоценка
        </h3>

        <div className="space-y-4">
          <div className="p-4 bg-orange-50 border border-orange-200 rounded-md">
            <h4 className="font-medium text-orange-900 mb-2">⚠️ Внимание!</h4>
            <p className="text-sm text-orange-800">
              Преди да стартирате преоценката, прегледайте справката по-горе за да видите какви записи ще бъдат създадени.
              След изпълнението, записите ще бъдат създадени в неразнесено състояние за преглед.
            </p>
          </div>

          <div className="flex items-center justify-end space-x-3">
            <button
              onClick={handleRunRevaluation}
              disabled={running || !settings.expenseAccountId || !settings.revenueAccountId || !preview || preview.items.length === 0}
              className="px-6 py-3 bg-purple-600 text-white rounded-md font-medium hover:bg-purple-700 disabled:bg-gray-400 disabled:cursor-not-allowed flex items-center"
            >
              {running ? (
                <>
                  <div className="animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full mr-2"></div>
                  Изпълнява се...
                </>
              ) : (
                <>
                  <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  Стартирай преоценка
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Account Select Modal */}
      <AccountSelectModal
        show={showAccountModal}
        accounts={accounts}
        currentAccountId={accountModalType === 'expense' ? settings.expenseAccountId : settings.revenueAccountId}
        onSelect={handleAccountSelect}
        onClose={() => {
          setShowAccountModal(false);
          setAccountModalType(null);
        }}
      />
    </div>
  );
}
