import { useState, useEffect, useRef, useCallback } from 'react';

// GraphQL queries
const ACCOUNT_HIERARCHY_QUERY = `
  query AccountHierarchy($companyId: Int!, $includeInactive: Boolean, $limit: Int, $offset: Int) {
    accountHierarchy(companyId: $companyId, includeInactive: $includeInactive, limit: $limit, offset: $offset) {
      id
      code
      name
      accountType
      accountClass
      parentId
      level
      isVatApplicable
      vatDirection
      isActive
      isAnalytical
      supportsQuantities
      defaultUnit
      createdAt
      updatedAt
    }
  }
`;

const ACCOUNT_BALANCES_QUERY = `
  query AccountBalances($companyId: Int!, $asOfDate: NaiveDate) {
    accountBalances(companyId: $companyId, asOfDate: $asOfDate) {
      account {
        id
        code
        name
        accountType
        accountClass
        parentId
        level
        isVatApplicable
        vatDirection
        isActive
        isAnalytical
      }
      debitBalance
      creditBalance
      netBalance
    }
  }
`;

const CREATE_ACCOUNT_MUTATION = `
  mutation CreateAccount($input: CreateAccountInput!) {
    createAccount(input: $input) {
      id
      code
      name
      accountType
      accountClass
      parentId
      level
      isVatApplicable
      vatDirection
      isActive
      isAnalytical
      supportsQuantities
      defaultUnit
    }
  }
`;

// GraphQL client function
async function graphqlRequest(query, variables = {}) {
  try {
    const token = localStorage.getItem('authToken');
    const headers = {
      'Content-Type': 'application/json',
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch('/graphql', {
      method: 'POST',
      headers,
      body: JSON.stringify({
        query,
        variables,
      }),
    });

    const result = await response.json();
    
    if (result.errors) {
      throw new Error(result.errors[0].message);
    }
    
    return result.data;
  } catch (error) {
    console.error('GraphQL Error:', error);
    throw error;
  }
}

// Account type translations
const accountTypeLabels = {
  'ASSET': 'Активи',
  'LIABILITY': 'Пасиви',
  'EQUITY': 'Капитал',
  'REVENUE': 'Приходи',
  'EXPENSE': 'Разходи'
};

const accountTypeColors = {
  'ASSET': 'bg-blue-100 text-blue-800',
  'LIABILITY': 'bg-red-100 text-red-800',
  'EQUITY': 'bg-purple-100 text-purple-800',
  'REVENUE': 'bg-green-100 text-green-800',
  'EXPENSE': 'bg-yellow-100 text-yellow-800'
};

const accountClassLabels = {
  1: 'Клас 1 - Капитал и резерви',
  2: 'Клас 2 - Дълготрайни активи',
  3: 'Клас 3 - Материални запаси',
  4: 'Клас 4 - Разчети с третата лица',
  5: 'Клас 5 - Парични средства',
  6: 'Клас 6 - Разходи',
  7: 'Клас 7 - Приходи',
  8: 'Клас 8 - Извънбалансови сметки'
};

const PAGE_SIZE = 100;

function AccountRow({ account, level = 0, showBalances = false, onEdit, onToggle, isExpanded, children, expandedNodes }) {
  const hasChildren = children && children.length > 0;
  const indentLevel = Math.min(level, 5); // Max 5 levels of indentation
  
  return (
    <>
      <tr className={`hover:bg-gray-50 ${!account.is_active ? 'opacity-60' : ''}`}>
        <td className="px-6 py-3 whitespace-nowrap">
          <div className="flex items-center" style={{ paddingLeft: `${indentLevel * 20}px` }}>
            {hasChildren && (
              <button
                onClick={() => onToggle(account.id)}
                className="mr-2 text-gray-400 hover:text-gray-600"
              >
                {isExpanded ? '▼' : '▶'}
              </button>
            )}
            {!hasChildren && <div className="w-6 mr-2"></div>}
            <div className="flex items-center">
              <div className="text-sm font-medium text-gray-900">{account.code}</div>
              {account.isAnalytical && (
                <span className="ml-2 px-1.5 py-0.5 text-xs bg-gray-100 text-gray-700 rounded">А</span>
              )}
              {account.isVatApplicable && (
                <span className="ml-1 px-1.5 py-0.5 text-xs bg-blue-100 text-blue-700 rounded">ДДС</span>
              )}
            </div>
          </div>
        </td>
        <td className="px-6 py-3">
          <div className="text-sm text-gray-900">{account.name}</div>
        </td>
        <td className="px-6 py-3 whitespace-nowrap">
          <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${accountTypeColors[account.accountType] || 'bg-gray-100 text-gray-800'}`}>
            {accountTypeLabels[account.accountType] || account.accountType}
          </span>
        </td>
        <td className="px-6 py-3 whitespace-nowrap text-sm text-gray-900">
          {account.accountClass}
        </td>
        <td className="px-6 py-3 whitespace-nowrap text-center">
          {account.supportsQuantities ? (
            <div className="text-xs">
              <div className="flex items-center justify-center">
                <svg className="w-4 h-4 text-green-600 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                </svg>
                <span className="text-green-600 font-medium">{account.defaultUnit || 'бр'}</span>
              </div>
            </div>
          ) : (
            <span className="text-gray-400 text-xs">—</span>
          )}
        </td>
        {showBalances && (
          <>
            <td className="px-6 py-3 whitespace-nowrap text-sm text-gray-900 text-right">
              {account.debitBalance ? parseFloat(account.debitBalance).toFixed(2) + ' лв.' : '-'}
            </td>
            <td className="px-6 py-3 whitespace-nowrap text-sm text-gray-900 text-right">
              {account.creditBalance ? parseFloat(account.creditBalance).toFixed(2) + ' лв.' : '-'}
            </td>
            <td className="px-6 py-3 whitespace-nowrap text-sm text-right">
              <span className={`font-medium ${
                account.netBalance 
                  ? parseFloat(account.netBalance) >= 0 
                    ? 'text-green-600' 
                    : 'text-red-600'
                  : 'text-gray-500'
              }`}>
                {account.netBalance ? parseFloat(account.netBalance).toFixed(2) + ' лв.' : '-'}
              </span>
            </td>
          </>
        )}
        <td className="px-6 py-3 whitespace-nowrap text-center">
          <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
            (account.isActive !== undefined ? account.isActive : true) ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
          }`}>
            {(account.isActive !== undefined ? account.isActive : true) ? 'Активна' : 'Неактивна'}
          </span>
        </td>
        <td className="px-6 py-3 whitespace-nowrap text-sm font-medium">
          <button
            onClick={() => onEdit(account)}
            className="text-blue-600 hover:text-blue-900 mr-3"
          >
            Редактирай
          </button>
        </td>
      </tr>
      {isExpanded && children && children.map(child => (
        <AccountRow
          key={child.account?.id || child.id}
          account={child.account || child}
          level={level + 1}
          showBalances={showBalances}
          onEdit={onEdit}
          onToggle={onToggle}
          isExpanded={expandedNodes.has(child.id)}
          children={child.children}
          expandedNodes={expandedNodes}
        />
      ))}
    </>
  );
}

function AccountModal({ account, isOpen, onClose, onSave, accounts }) {
  const [formData, setFormData] = useState({
    code: '',
    name: '',
    accountType: 'ASSET',
    accountClass: 1,
    parentId: null,
    isVatApplicable: false,
    vatDirection: 'NONE',
    isActive: true,
    isAnalytical: false,
    supportsQuantities: false,
    defaultUnit: '',
  });

  // Get potential parent accounts (synthetic accounts only)
  const getParentAccountOptions = () => {
    return accounts.filter(acc => 
      !acc.isAnalytical && // Only synthetic accounts can be parents
      acc.id !== account?.id // Can't be parent of itself
    );
  };

  useEffect(() => {
    if (account) {
      setFormData({ ...account });
    }
  }, [account]);

  const handleSubmit = (e) => {
    e.preventDefault();
    
    // Validate analytical accounts must have parent
    if (formData.isAnalytical && !formData.parentId) {
      alert('Аналитичните сметки трябва да имат родителска сметка!');
      return;
    }
    
    onSave(formData);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          {account ? 'Редактиране на сметка' : 'Нова сметка'}
        </h3>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Код</label>
            <input
              type="text"
              value={formData.code}
              onChange={(e) => setFormData({ ...formData, code: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Наименование</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              required
            />
          </div>
          
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Тип сметка</label>
              <select
                value={formData.accountType}
                onChange={(e) => setFormData({ ...formData, accountType: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
              >
                {Object.entries(accountTypeLabels).map(([key, label]) => (
                  <option key={key} value={key}>{label}</option>
                ))}
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Клас</label>
              <select
                value={formData.accountClass}
                onChange={(e) => {
                  const accountClass = parseInt(e.target.value);
                  setFormData({ 
                    ...formData, 
                    accountClass,
                    // Auto-enable quantity support for material/production accounts
                    supportsQuantities: accountClass === 2 || accountClass === 3,
                    defaultUnit: (accountClass === 2 || accountClass === 3) && !formData.defaultUnit ? 'бр' : formData.defaultUnit
                  });
                }}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
              >
                {Object.entries(accountClassLabels).map(([key, label]) => (
                  <option key={key} value={key}>{key} - {label.split(' - ')[1]}</option>
                ))}
              </select>
            </div>
          </div>
          
          {/* Parent Account Selection - only show for analytical accounts */}
          {formData.isAnalytical && (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Родителска сметка <span className="text-red-500">*</span>
              </label>
              <select
                value={formData.parentId || ''}
                onChange={(e) => setFormData({ ...formData, parentId: e.target.value ? parseInt(e.target.value) : null })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
                required={formData.isAnalytical}
              >
                <option value="">Избери родителска сметка...</option>
                {getParentAccountOptions().map(parentAccount => (
                  <option key={parentAccount.id} value={parentAccount.id}>
                    {parentAccount.code} - {parentAccount.name}
                  </option>
                ))}
              </select>
              <p className="mt-1 text-sm text-gray-500">
                Аналитичните сметки трябва да имат родителска синтетична сметка
              </p>
            </div>
          )}
          
          <div className="flex items-center space-x-4">
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={formData.isVatApplicable}
                onChange={(e) => setFormData({ ...formData, isVatApplicable: e.target.checked })}
                className="mr-2"
              />
              <span className="text-sm text-gray-700">ДДС приложима</span>
            </label>
            
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={formData.isAnalytical}
                onChange={(e) => setFormData({ ...formData, isAnalytical: e.target.checked })}
                className="mr-2"
              />
              <span className="text-sm text-gray-700">Аналитична</span>
            </label>
          </div>
          
          <div className="space-y-4 border-t border-gray-200 pt-4">
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={formData.supportsQuantities}
                onChange={(e) => setFormData({ 
                  ...formData, 
                  supportsQuantities: e.target.checked,
                  // Auto-set default unit for material/production accounts
                  defaultUnit: e.target.checked && !formData.defaultUnit && (formData.accountClass === 2 || formData.accountClass === 3) 
                    ? 'бр' 
                    : formData.defaultUnit
                })}
                className="mr-2"
              />
              <span className="text-sm text-gray-700">Поддържа количества</span>
            </label>
            <p className="text-xs text-gray-500">
              Препоръчва се за сметки от група 2 (материали) и 3 (продукция)
            </p>
            
            {formData.supportsQuantities && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Мерна единица по подразбиране
                </label>
                <select
                  value={formData.defaultUnit || ''}
                  onChange={(e) => setFormData({ ...formData, defaultUnit: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md"
                >
                  <option value="">Избери мерна единица...</option>
                  <option value="бр">бр</option>
                  <option value="кг">кг</option>
                  <option value="г">г</option>
                  <option value="т">т</option>
                  <option value="л">л</option>
                  <option value="мл">мл</option>
                  <option value="м">м</option>
                  <option value="см">см</option>
                  <option value="м²">м²</option>
                  <option value="м³">м³</option>
                  <option value="час">час</option>
                  <option value="ден">ден</option>
                  <option value="км">км</option>
                </select>
                <p className="mt-1 text-sm text-gray-500">
                  Тази мерна единица ще се използва по подразбиране при журнални записи
                </p>
              </div>
            )}
          </div>
          
          <div className="flex space-x-3 pt-4">
            <button
              type="submit"
              className="flex-1 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700"
            >
              {account ? 'Запази' : 'Създай'}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="flex-1 bg-gray-300 text-gray-700 px-4 py-2 rounded-md hover:bg-gray-400"
            >
              Отказ
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default function ChartOfAccounts() {
  const [accounts, setAccounts] = useState([]);
  const [accountsWithBalance, setAccountsWithBalance] = useState([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [error, setError] = useState(null);
  const [showBalances, setShowBalances] = useState(false);
  const [filter, setFilter] = useState('');
  const [selectedClass, setSelectedClass] = useState('');
  const [selectedType, setSelectedType] = useState('');
  const [showInactive, setShowInactive] = useState(false);
  const [showAnalytical, setShowAnalytical] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingAccount, setEditingAccount] = useState(null);
  const [expandedNodes, setExpandedNodes] = useState(new Set());
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [offset, setOffset] = useState(0);
  const tableContainerRef = useRef(null);

  const loadAccounts = useCallback(async (reset = false) => {
    if (reset) {
      setLoading(true);
      setLoadingMore(false);
      setError(null);
      setHasMore(true);
      setOffset(0);
    } else {
      if (loading || loadingMore || !hasMore) {
        return;
      }
      setLoadingMore(true);
      setError(null);
    }

    try {
      const currentOffset = reset ? 0 : offset;
      const data = await graphqlRequest(ACCOUNT_HIERARCHY_QUERY, {
        companyId,
        includeInactive: showInactive,
        limit: PAGE_SIZE,
        offset: currentOffset
      });

      const newAccounts = data.accountHierarchy || [];

      if (reset) {
        setAccounts(newAccounts);
      } else {
        setAccounts(prev => [...prev, ...newAccounts]);
      }

      setHasMore(newAccounts.length === PAGE_SIZE);
      setOffset(reset ? newAccounts.length : offset + newAccounts.length);
    } catch (err) {
      setError('Грешка при зареждане на сметкоплана: ' + err.message);
      if (reset) {
        setAccounts([]);
        setHasMore(false);
      }
    } finally {
      if (reset) {
        setLoading(false);
      } else {
        setLoadingMore(false);
      }
    }
  }, [companyId, showInactive, loading, loadingMore, hasMore, offset]);

  const loadAccountsWithBalances = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const data = await graphqlRequest(ACCOUNT_BALANCES_QUERY, { 
        companyId,
        asOfDate: new Date().toISOString().split('T')[0]
      });
      setAccountsWithBalance(data.accountBalances || []);
    } catch (err) {
      setError('Грешка при зареждане на салда: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (showBalances) {
      loadAccountsWithBalances();
    } else {
      loadAccounts(true);
    }
  }, [showBalances, companyId, showInactive]);

  const handleScroll = useCallback((event) => {
    if (loading || loadingMore || !hasMore || showBalances) {
      return;
    }

    const { scrollTop, scrollHeight, clientHeight } = event.target;
    if (scrollHeight - scrollTop - clientHeight < 100) {
      loadAccounts(false);
    }
  }, [loadAccounts, hasMore, loading, loadingMore, showBalances]);

  const buildAccountTree = (accounts) => {
    const accountMap = new Map();
    const rootAccounts = [];

    // Create map of accounts
    accounts.forEach(account => {
      accountMap.set(account.id, { ...account, children: [] });
    });

    // Build tree structure
    accounts.forEach(account => {
      if (account.parentId) {
        const parent = accountMap.get(account.parentId);
        if (parent) {
          parent.children.push(accountMap.get(account.id));
        }
      } else {
        rootAccounts.push(accountMap.get(account.id));
      }
    });

    return rootAccounts;
  };

  const filterAccounts = (accounts) => {
    // When filtering by code/name, show all matching accounts regardless of hierarchy
    if (filter) {
      return accounts.filter(account => {
        const matchesFilter = 
          account.code.toLowerCase().includes(filter.toLowerCase()) ||
          account.name.toLowerCase().includes(filter.toLowerCase());
        
        const matchesClass = !selectedClass || account.accountClass.toString() === selectedClass;
        const matchesType = !selectedType || account.accountType === selectedType;
        const matchesActive = showInactive || account.isActive;
        const matchesAnalytical = showAnalytical || !account.isAnalytical;

        return matchesFilter && matchesClass && matchesType && matchesActive && matchesAnalytical;
      });
    }
    
    // When no text filter, apply other filters and respect hierarchy
    return accounts.filter(account => {
      const matchesClass = !selectedClass || account.accountClass.toString() === selectedClass;
      const matchesType = !selectedType || account.accountType === selectedType;
      const matchesActive = showInactive || account.isActive;
      const matchesAnalytical = showAnalytical || !account.isAnalytical;

      return matchesClass && matchesType && matchesActive && matchesAnalytical;
    });
  };

  const handleEdit = (account) => {
    setEditingAccount(account);
    setModalOpen(true);
  };

  const handleSave = async (accountData) => {
    try {
      if (editingAccount) {
        // Update existing account - would need update mutation
        console.log('Update account:', accountData);
      } else {
        // Create new account - remove fields not in CreateAccountInput
        const { isActive: _isActive, isAnalytical: _isAnalytical, ...createInput } = accountData;
        await graphqlRequest(CREATE_ACCOUNT_MUTATION, {
          input: {
            ...createInput,
            companyId: companyId
          }
        });
      }
      
      setModalOpen(false);
      setEditingAccount(null);
      
      // Reload accounts
      if (showBalances) {
        loadAccountsWithBalances();
      } else {
        loadAccounts();
      }
    } catch (err) {
      setError('Грешка при запазване: ' + err.message);
    }
  };

  const toggleNode = (accountId) => {
    const newExpanded = new Set(expandedNodes);
    if (newExpanded.has(accountId)) {
      newExpanded.delete(accountId);
    } else {
      newExpanded.add(accountId);
    }
    setExpandedNodes(newExpanded);
  };

  const displayAccounts = showBalances ? accountsWithBalance : accounts;
  const filteredAccounts = filterAccounts(displayAccounts);
  
  // When filtering, show flat list instead of tree
  const accountsToDisplay = filter 
    ? filteredAccounts.sort((a, b) => a.code.localeCompare(b.code))
    : buildAccountTree(filteredAccounts);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
        <span className="ml-2 text-gray-600">Зарежда сметкоплан...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Сметкоплан</h1>
          <p className="mt-1 text-sm text-gray-500">
            Управление на счетоводните сметки
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <button
            onClick={() => setModalOpen(true)}
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
          >
            <svg className="-ml-1 mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4"></path>
            </svg>
            Нова сметка
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center">
            <div className="text-red-400 mr-3">⚠️</div>
            <div className="text-sm text-red-800">{error}</div>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4">
          <div>
            <input
              type="text"
              placeholder="Търси по код/име..."
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>
          
          <div>
            <select
              value={selectedClass}
              onChange={(e) => setSelectedClass(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-white"
            >
              <option value="">Всички класове</option>
              {Object.entries(accountClassLabels).map(([key, label]) => (
                <option key={key} value={key}>{label}</option>
              ))}
            </select>
          </div>
          
          <div>
            <select
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-white"
            >
              <option value="">Всички типове</option>
              {Object.entries(accountTypeLabels).map(([key, label]) => (
                <option key={key} value={key}>{label}</option>
              ))}
            </select>
          </div>
          
          <div className="flex items-center">
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={showBalances}
                onChange={(e) => setShowBalances(e.target.checked)}
                className="mr-2"
              />
              <span className="text-sm text-gray-700">Салда</span>
            </label>
          </div>
          
          <div className="flex items-center">
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={showInactive}
                onChange={(e) => setShowInactive(e.target.checked)}
                className="mr-2"
              />
              <span className="text-sm text-gray-700">Неактивни</span>
            </label>
          </div>
          
          <div className="flex items-center">
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={showAnalytical}
                onChange={(e) => setShowAnalytical(e.target.checked)}
                className="mr-2"
              />
              <span className="text-sm text-gray-700">Аналитични</span>
            </label>
          </div>
        </div>
      </div>

      {/* Accounts Table */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div
          ref={tableContainerRef}
          onScroll={handleScroll}
          className="overflow-x-auto max-h-[calc(100vh-400px)]"
        >
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Код
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Наименование
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Тип
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Клас
                </th>
                <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Количества
                </th>
                {showBalances && (
                  <>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Дебит
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Кредит
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Салдо
                    </th>
                  </>
                )}
                <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Статус
                </th>
                <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Действия
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filter ? (
                // Show flat list when filtering
                accountsToDisplay.map(account => (
                  <tr key={account.id} className="hover:bg-gray-50">
                    <td className="px-6 py-3 whitespace-nowrap">
                      <div className="flex items-center">
                        <div className="text-sm font-medium text-gray-900">{account.code}</div>
                        {account.isAnalytical && (
                          <span className="ml-2 px-1.5 py-0.5 text-xs bg-gray-100 text-gray-700 rounded">А</span>
                        )}
                        {account.isVatApplicable && (
                          <span className="ml-1 px-1.5 py-0.5 text-xs bg-blue-100 text-blue-700 rounded">ДДС</span>
                        )}
                      </div>
                    </td>
                    <td className="px-6 py-3">
                      <div className="text-sm text-gray-900">{account.name}</div>
                    </td>
                    <td className="px-6 py-3 whitespace-nowrap">
                      <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${accountTypeColors[account.accountType] || 'bg-gray-100 text-gray-800'}`}>
                        {accountTypeLabels[account.accountType] || account.accountType}
                      </span>
                    </td>
                    <td className="px-6 py-3 whitespace-nowrap text-sm text-gray-900">
                      {account.accountClass}
                    </td>
                    <td className="px-6 py-3 whitespace-nowrap text-center">
                      {account.supportsQuantities ? (
                        <div className="text-xs">
                          <div className="flex items-center justify-center">
                            <svg className="w-4 h-4 text-green-600 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                            </svg>
                            <span className="text-green-600 font-medium">{account.defaultUnit || 'бр'}</span>
                          </div>
                        </div>
                      ) : (
                        <span className="text-gray-400 text-xs">—</span>
                      )}
                    </td>
                    {showBalances && (
                      <>
                        <td className="px-6 py-3 whitespace-nowrap text-sm text-gray-900 text-right">
                          {account.debitBalance ? parseFloat(account.debitBalance).toFixed(2) + ' лв.' : '-'}
                        </td>
                        <td className="px-6 py-3 whitespace-nowrap text-sm text-gray-900 text-right">
                          {account.creditBalance ? parseFloat(account.creditBalance).toFixed(2) + ' лв.' : '-'}
                        </td>
                        <td className="px-6 py-3 whitespace-nowrap text-sm text-right">
                          <span className={`font-medium ${
                            account.netBalance 
                              ? parseFloat(account.netBalance) >= 0 
                                ? 'text-green-600' 
                                : 'text-red-600'
                              : 'text-gray-500'
                          }`}>
                            {account.netBalance ? parseFloat(account.netBalance).toFixed(2) + ' лв.' : '-'}
                          </span>
                        </td>
                      </>
                    )}
                    <td className="px-6 py-3 whitespace-nowrap text-center">
                      <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
                        account.isActive ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      }`}>
                        {account.isActive ? 'Активна' : 'Неактивна'}
                      </span>
                    </td>
                    <td className="px-6 py-3 whitespace-nowrap text-sm font-medium">
                      <button
                        onClick={() => handleEdit(account)}
                        className="text-blue-600 hover:text-blue-900 mr-3"
                      >
                        Редактирай
                      </button>
                    </td>
                  </tr>
                ))
              ) : (
                // Show hierarchical tree when not filtering
                accountsToDisplay.map(account => (
                  <AccountRow
                    key={account.id}
                    account={account.account || account}
                    level={0}
                    showBalances={showBalances}
                    onEdit={handleEdit}
                    onToggle={toggleNode}
                    isExpanded={expandedNodes.has(account.id)}
                    children={account.children}
                    expandedNodes={expandedNodes}
                  />
                ))
              )}
              {accountsToDisplay.length === 0 && (
                <tr>
                  <td colSpan={showBalances ? 9 : 6} className="px-6 py-8 text-center text-gray-500">
                    Няма намерени сметки
                  </td>
                </tr>
              )}
            </tbody>
          </table>
          {loadingMore && (
            <div className="p-4 text-center border-t border-gray-200">
              <div className="animate-spin h-6 w-6 border-2 border-blue-500 border-t-transparent rounded-full mx-auto"></div>
              <p className="text-sm text-gray-500 mt-2">Зареждане на още сметки...</p>
            </div>
          )}
        </div>
      </div>

      {/* Account Modal */}
      <AccountModal
        account={editingAccount}
        isOpen={modalOpen}
        onClose={() => {
          setModalOpen(false);
          setEditingAccount(null);
        }}
        onSave={handleSave}
        accounts={accounts}
      />
    </div>
  );
}
