import { useState, useEffect, useMemo, useCallback } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';
import BankStatementsReview from '../components/banks/BankStatementsReview';

const BANK_PROFILES_QUERY = `
  query BankProfiles($companyId: Int!, $activeOnly: Boolean) {
    bankProfiles(companyId: $companyId, activeOnly: $activeOnly) {
      id
      name
      iban
      accountId
      bufferAccountId
      currencyCode
      importFormat
      isActive
      createdAt
      updatedAt
    }
  }
`;

const ACCOUNTS_QUERY = `
  query AccountHierarchy($companyId: Int!) {
    accountHierarchy(companyId: $companyId) {
      id
      code
      name
      isAnalytical
      isActive
    }
  }
`;

const CURRENCIES_QUERY = `
  query ActiveCurrencies {
    currencies(filter: { isActive: true }) {
      code
      nameBg
      name
    }
  }
`;

const CREATE_BANK_PROFILE = `
  mutation CreateBankProfile($input: CreateBankProfileInput!) {
    createBankProfile(input: $input) {
      id
    }
  }
`;

const UPDATE_BANK_PROFILE = `
  mutation UpdateBankProfile($id: Int!, $input: UpdateBankProfileInput!) {
    updateBankProfile(id: $id, input: $input) {
      id
    }
  }
`;

const TOGGLE_BANK_PROFILE_STATUS = `
  mutation ToggleBankProfileStatus($id: Int!, $isActive: Boolean!) {
    setBankProfileStatus(id: $id, isActive: $isActive) {
      id
      isActive
    }
  }
`;

const importFormatOptions = [
  { value: 'UNICREDIT_MT940', label: 'UniCredit MT940 (SWIFT/TXT)' },
  { value: 'WISE_CAMT053', label: 'Wise CAMT.053 XML' },
  { value: 'REVOLUT_CAMT053', label: 'Revolut CAMT.053 XML' },
  { value: 'PAYSERA_CAMT053', label: 'Paysera CAMT.053 XML' },
  { value: 'POSTBANK_XML', label: 'Postbank XML' },
  { value: 'OBB_XML', label: 'OBB XML' },
  { value: 'CCB_CSV', label: 'ЦКБ CSV' },
];

const initialFormState = {
  name: '',
  iban: '',
  accountId: '',
  bufferAccountId: '',
  currencyCode: 'BGN',
  importFormat: 'UNICREDIT_MT940',
  isActive: true,
};

function formatAccount(accountsMap, accountId) {
  if (!accountId) {
    return '—';
  }

  const account = accountsMap.get(accountId);
  if (!account) {
    return `ID ${accountId}`;
  }

  return `${account.code} · ${account.name}`;
}

export default function Banks() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [bankProfiles, setBankProfiles] = useState([]);
  const [accounts, setAccounts] = useState([]);
  const [currencies, setCurrencies] = useState([]);
  const [loading, setLoading] = useState(true);
  const [referenceLoading, setReferenceLoading] = useState(true);
  const [error, setError] = useState(null);
  const [formState, setFormState] = useState(initialFormState);
  const [editingId, setEditingId] = useState(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [activeTab, setActiveTab] = useState('profiles');

  const accountsMap = useMemo(() => {
    const map = new Map();
    accounts.forEach((account) => {
      map.set(account.id, account);
    });
    return map;
  }, [accounts]);

  const currencyOptions = useMemo(() => {
    if (currencies.length === 0) {
      return [{ code: 'BGN', nameBg: 'Български лев', name: 'Bulgarian Lev' }];
    }
    return currencies;
  }, [currencies]);

  const loadBankProfiles = useCallback(async () => {
    try {
      const data = await graphqlRequest(BANK_PROFILES_QUERY, {
        companyId,
        activeOnly: false,
      });
      setBankProfiles(data?.bankProfiles ?? []);
      setError(null);
    } catch (err) {
      console.error('Грешка при зареждане на банкови профили:', err);
      setError(err.message);
    }
  }, [companyId]);

  const loadReferenceData = useCallback(async () => {
    try {
      const [accountsData, currenciesData] = await Promise.all([
        graphqlRequest(ACCOUNTS_QUERY, {
          companyId,
        }),
        graphqlRequest(CURRENCIES_QUERY),
      ]);

      const hierarchy = accountsData?.accountHierarchy ?? [];
      setAccounts(hierarchy);
      setCurrencies(currenciesData?.currencies ?? []);
    } catch (err) {
      console.error('Грешка при зареждане на справочни данни:', err);
      setError(err.message);
    }
  }, [companyId]);

  useEffect(() => {
    let mounted = true;

    const loadData = async () => {
      setLoading(true);
      setReferenceLoading(true);
      try {
        await Promise.all([
          loadBankProfiles(),
          loadReferenceData(),
        ]);
      } finally {
        if (mounted) {
          setLoading(false);
          setReferenceLoading(false);
        }
      }
    };

    loadData();

    return () => {
      mounted = false;
    };
  }, [loadBankProfiles, loadReferenceData]);

  const resetForm = () => {
    setFormState(initialFormState);
    setEditingId(null);
  };

  const handleEdit = (profile) => {
    setEditingId(profile.id);
    setFormState({
      name: profile.name || '',
      iban: profile.iban || '',
      accountId: profile.accountId ? String(profile.accountId) : '',
      bufferAccountId: profile.bufferAccountId ? String(profile.bufferAccountId) : '',
      currencyCode: profile.currencyCode || 'BGN',
      importFormat: profile.importFormat || 'UNICREDIT_MT940',
      isActive: Boolean(profile.isActive),
    });
  };

  const handleSubmit = async (event) => {
    event.preventDefault();
    setIsSubmitting(true);

    try {
      const payload = {
        name: formState.name.trim(),
        iban: formState.iban.trim() ? formState.iban.trim() : null,
        accountId: formState.accountId ? parseInt(formState.accountId, 10) : null,
        bufferAccountId: formState.bufferAccountId ? parseInt(formState.bufferAccountId, 10) : null,
        currencyCode: formState.currencyCode,
        importFormat: formState.importFormat,
        isActive: formState.isActive,
      };

      if (!payload.accountId || !payload.bufferAccountId) {
        throw new Error('Моля изберете аналитична и буферна сметка.');
      }

      if (!payload.importFormat) {
        throw new Error('Моля изберете формат на импорта.');
      }

      if (editingId) {
        await graphqlRequest(UPDATE_BANK_PROFILE, {
          id: editingId,
          input: {
            name: payload.name,
            iban: payload.iban,
            accountId: payload.accountId,
            bufferAccountId: payload.bufferAccountId,
            currencyCode: payload.currencyCode,
            importFormat: payload.importFormat,
            isActive: payload.isActive,
          },
        });
      } else {
        await graphqlRequest(CREATE_BANK_PROFILE, {
          input: {
            companyId,
            name: payload.name,
            iban: payload.iban,
            accountId: payload.accountId,
            bufferAccountId: payload.bufferAccountId,
            currencyCode: payload.currencyCode,
            importFormat: payload.importFormat,
            isActive: payload.isActive,
          },
        });
      }

      await loadBankProfiles();
      resetForm();
      setError(null);
    } catch (err) {
      console.error('Грешка при запис на банков профил:', err);
      setError(err.message);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleToggleStatus = async (profile) => {
    try {
      await graphqlRequest(TOGGLE_BANK_PROFILE_STATUS, {
        id: profile.id,
        isActive: !profile.isActive,
      });
      await loadBankProfiles();
    } catch (err) {
      console.error('Грешка при активиране/деактивиране на профил:', err);
      setError(err.message);
    }
  };

  const accountsOptions = useMemo(
    () => accounts.map((account) => ({
      value: String(account.id),
      label: `${account.code} · ${account.name}`,
      disabled: !account.isAnalytical,
    })),
    [accounts],
  );

  const sortedProfiles = useMemo(() => {
    return [...bankProfiles].sort((a, b) => {
      if (a.isActive === b.isActive) {
        return a.name.localeCompare(b.name);
      }
      return a.isActive ? -1 : 1;
    });
  }, [bankProfiles]);

  const renderProfilesTab = () => (
    <>
      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-4">
          <div className="bg-white border border-gray-200 rounded-lg shadow-sm">
            <div className="px-6 py-4 border-b border-gray-100 flex items-center justify-between">
              <div>
                <h2 className="text-lg font-semibold text-gray-900">Конфигурирани банки</h2>
                <p className="text-sm text-gray-500">
                  Аналитична сметка, валута и формат на импорта за всяка банка.
                </p>
              </div>
              {(loading || referenceLoading) && (
                <div className="text-sm text-gray-500">Зареждане...</div>
              )}
            </div>

            <div className="divide-y divide-gray-100">
              {sortedProfiles.length === 0 && !loading ? (
                <div className="px-6 py-10 text-center text-gray-500 text-sm">
                  Все още няма добавени банкови профили. Създайте първия от формата вдясно.
                </div>
              ) : (
                sortedProfiles.map((profile) => (
                  <div
                    key={profile.id}
                    className="px-6 py-5 flex flex-col lg:flex-row lg:items-center lg:justify-between space-y-4 lg:space-y-0"
                  >
                    <div className="space-y-2">
                      <div className="flex items-center space-x-3">
                        <span className="text-xl">🏦</span>
                        <div>
                          <div className="text-base font-semibold text-gray-900 flex items-center space-x-2">
                            <span>{profile.name}</span>
                            <span
                              className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                                profile.isActive ? 'bg-green-100 text-green-700' : 'bg-gray-200 text-gray-600'
                              }`}
                            >
                              {profile.isActive ? 'Активна' : 'Неактивна'}
                            </span>
                          </div>
                          {profile.iban && (
                            <div className="text-sm text-gray-500">IBAN: {profile.iban}</div>
                          )}
                        </div>
                      </div>

                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm text-gray-600">
                        <div>
                          <div className="font-medium text-gray-700">Основна сметка</div>
                          <div>{formatAccount(accountsMap, profile.accountId)}</div>
                        </div>
                        <div>
                          <div className="font-medium text-gray-700">Буферна сметка</div>
                          <div>{formatAccount(accountsMap, profile.bufferAccountId)}</div>
                        </div>
                        <div>
                          <div className="font-medium text-gray-700">Валута</div>
                          <div>{profile.currencyCode}</div>
                        </div>
                        <div>
                          <div className="font-medium text-gray-700">Формат на импорт</div>
                          <div>{profile.importFormat}</div>
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center space-x-3">
                      <button
                        type="button"
                        onClick={() => handleEdit(profile)}
                        className="px-3 py-2 text-sm border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50"
                      >
                        Редакция
                      </button>
                      <button
                        type="button"
                        onClick={() => handleToggleStatus(profile)}
                        className={`px-3 py-2 text-sm rounded-md border ${
                          profile.isActive
                            ? 'border-yellow-300 text-yellow-700 bg-yellow-50 hover:bg-yellow-100'
                            : 'border-green-300 text-green-700 bg-green-50 hover:bg-green-100'
                        }`}
                      >
                        {profile.isActive ? 'Деактивирай' : 'Активирай'}
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        <div className="bg-white border border-gray-200 rounded-lg shadow-sm p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-1">
            {editingId ? 'Редакция на банков профил' : 'Нов банков профил'}
          </h2>
          <p className="text-sm text-gray-500 mb-4">
            Задайте аналитична сметка, буферна сметка, валута и формат на импорта за банковото извлечение.
          </p>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Име на банката
              </label>
              <input
                type="text"
                value={formState.name}
                onChange={(event) => setFormState((prev) => ({ ...prev, name: event.target.value }))}
                required
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
                placeholder="Напр. UniCredit Bulbank"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">IBAN (по избор)</label>
              <input
                type="text"
                value={formState.iban}
                onChange={(event) => setFormState((prev) => ({ ...prev, iban: event.target.value }))}
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
                placeholder="BGxx XXXX XXXX XXXX"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Аналитична сметка (Дт/Кт банкова сметка)
              </label>
              <select
                value={formState.accountId}
                onChange={(event) => setFormState((prev) => ({ ...prev, accountId: event.target.value }))}
                required
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
              >
                <option value="">Изберете сметка</option>
                {accountsOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Буферна сметка (напр. 484)
              </label>
              <select
                value={formState.bufferAccountId}
                onChange={(event) => setFormState((prev) => ({ ...prev, bufferAccountId: event.target.value }))}
                required
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
              >
                <option value="">Изберете сметка</option>
                {accountsOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Валута</label>
                <select
                  value={formState.currencyCode}
                  onChange={(event) => setFormState((prev) => ({ ...prev, currencyCode: event.target.value }))}
                  required
                  className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
                >
                  {currencyOptions.map((currency) => (
                    <option key={currency.code} value={currency.code}>
                      {currency.code} · {currency.nameBg || currency.name}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Формат на импорта</label>
                <select
                  value={formState.importFormat}
                  onChange={(event) => setFormState((prev) => ({ ...prev, importFormat: event.target.value }))}
                  required
                  className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
                >
                  {importFormatOptions.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            <div className="flex items-center space-x-2">
              <input
                id="bank-is-active"
                type="checkbox"
                checked={formState.isActive}
                onChange={(event) => setFormState((prev) => ({ ...prev, isActive: event.target.checked }))}
                className="h-4 w-4 text-blue-600 border-gray-300 rounded"
              />
              <label htmlFor="bank-is-active" className="text-sm text-gray-700">
                Профилът е активен за импортиране
              </label>
            </div>

            <div className="flex items-center justify-end space-x-3 pt-2">
              {editingId && (
                <button
                  type="button"
                  onClick={resetForm}
                  className="px-4 py-2 text-sm font-medium text-gray-600 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                >
                  Отказ
                </button>
              )}
              <button
                type="submit"
                disabled={isSubmitting}
                className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:bg-gray-400"
              >
                {isSubmitting ? 'Запис...' : editingId ? 'Запази промените' : 'Създай профил'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </>
  );

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Банки</h1>
        <p className="mt-2 text-gray-600">
          Управлявайте банковите профили и преглеждайте автоматично генерираните журнални записи от извлеченията.
        </p>
      </div>

      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-6">
          <button
            type="button"
            onClick={() => setActiveTab('profiles')}
            className={`py-3 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'profiles'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Профили
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('statements')}
            className={`py-3 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'statements'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Банкови записи
          </button>
        </nav>
      </div>

      {activeTab === 'profiles' ? (
        renderProfilesTab()
      ) : (
        <BankStatementsReview
          companyId={companyId}
          bankProfiles={bankProfiles}
          accounts={accounts}
        />
      )}
    </div>
  );
}
