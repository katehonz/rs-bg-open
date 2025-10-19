import { useState, useEffect, useRef } from 'react';

export default function AccountSelectModal({ 
  show, 
  accounts = [], 
  currentAccountId = null,
  onSelect, 
  onClose 
}) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedAccount, setSelectedAccount] = useState(null);
  const searchInputRef = useRef(null);

  // Focus search input when modal opens
  useEffect(() => {
    if (show && searchInputRef.current) {
      searchInputRef.current.focus();
      setSearchQuery('');
      
      // Find and select current account if provided
      if (currentAccountId) {
        const account = accounts.find(a => a.id === currentAccountId);
        if (account) {
          setSelectedAccount(account);
        }
      }
    }
  }, [show, currentAccountId, accounts]);

  // Filter and sort accounts based on search query
  const filteredAccounts = searchQuery.trim() 
    ? accounts.filter(account => {
        const query = searchQuery.toLowerCase().trim();
        const codeMatch = account.code.toLowerCase().includes(query);
        const nameMatch = account.name.toLowerCase().includes(query);
        return codeMatch || nameMatch;
      }).sort((a, b) => {
        const query = searchQuery.toLowerCase().trim();
        
        // First show exact matches by code
        const aCodeExact = a.code.toLowerCase() === query;
        const bCodeExact = b.code.toLowerCase() === query;
        if (aCodeExact && !bCodeExact) return -1;
        if (!aCodeExact && bCodeExact) return 1;
        
        // Then show accounts starting with search text
        const aCodeStarts = a.code.toLowerCase().startsWith(query);
        const bCodeStarts = b.code.toLowerCase().startsWith(query);
        if (aCodeStarts && !bCodeStarts) return -1;
        if (!aCodeStarts && bCodeStarts) return 1;
        
        // Finally sort by code
        return a.code.localeCompare(b.code);
      })
    : accounts.sort((a, b) => a.code.localeCompare(b.code));

  const getAccountTypeLabel = (accountType) => {
    const labels = {
      'ASSET': 'Актив',
      'LIABILITY': 'Пасив',
      'EQUITY': 'Капитал',
      'REVENUE': 'Приход',
      'EXPENSE': 'Разход'
    };
    return labels[accountType] || accountType;
  };

  const getAccountTypeColor = (accountType) => {
    const colors = {
      'ASSET': 'bg-blue-100 text-blue-700',
      'LIABILITY': 'bg-red-100 text-red-700',
      'EQUITY': 'bg-purple-100 text-purple-700',
      'REVENUE': 'bg-green-100 text-green-700',
      'EXPENSE': 'bg-yellow-100 text-yellow-700'
    };
    return colors[accountType] || 'bg-gray-100 text-gray-700';
  };

  const handleSelect = (account) => {
    setSelectedAccount(account);
  };

  const handleConfirm = () => {
    if (selectedAccount) {
      onSelect(selectedAccount);
      handleClose();
    }
  };

  const handleClose = () => {
    setSearchQuery('');
    setSelectedAccount(null);
    onClose();
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Escape') {
      handleClose();
    } else if (e.key === 'Enter' && selectedAccount) {
      handleConfirm();
    }
  };

  if (!show) return null;

  return (
    <div 
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onClick={handleClose}
      onKeyDown={handleKeyDown}
    >
      <div 
        className="bg-white rounded-lg w-full max-w-3xl max-h-[80vh] flex flex-col shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h3 className="text-xl font-semibold text-gray-900">Избор на сметка (синтетична или аналитична)</h3>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Search */}
        <div className="p-4 bg-gray-50 border-b border-gray-200">
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg className="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
            <input
              ref={searchInputRef}
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Търсене по код или име на сметка..."
              className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
        </div>

        {/* Accounts List */}
        <div className="flex-1 overflow-y-auto p-4">
          {filteredAccounts.length > 0 ? (
            <div className="space-y-2">
              {filteredAccounts.map((account) => (
                <div
                  key={account.id}
                  onClick={() => handleSelect(account)}
                  className={`p-3 rounded-lg border cursor-pointer transition-all ${
                    selectedAccount?.id === account.id
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-3">
                        <span className="font-mono font-semibold text-gray-900">
                          {account.code}
                        </span>
                        <span className="text-gray-700">
                          {account.name}
                        </span>
                      </div>
                      <div className="flex items-center space-x-2 mt-1">
                        <span className={`inline-flex px-2 py-0.5 text-xs font-medium rounded-full ${getAccountTypeColor(account.accountType)}`}>
                          {getAccountTypeLabel(account.accountType)}
                        </span>
                        {account.isAnalytical ? (
                          <span className="inline-flex px-2 py-0.5 text-xs font-medium rounded-full bg-gray-100 text-gray-700">
                            Аналитична
                          </span>
                        ) : (
                          <span className="inline-flex px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 text-green-700">
                            Синтетична
                          </span>
                        )}
                        {account.isVatApplicable && (
                          <span className="inline-flex px-2 py-0.5 text-xs font-medium rounded-full bg-blue-100 text-blue-700">
                            ДДС
                          </span>
                        )}
                        {account.supportsQuantities && (
                          <span className="inline-flex items-center px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 text-green-700">
                            <svg className="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                            </svg>
                            {account.defaultUnit || 'бр'}
                          </span>
                        )}
                      </div>
                    </div>
                    {selectedAccount?.id === account.id && (
                      <div className="text-blue-500">
                        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                        </svg>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8">
              <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <p className="mt-2 text-sm text-gray-500">Няма намерени сметки</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end space-x-3 p-6 border-t border-gray-200">
          <button
            onClick={handleClose}
            className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Отказ
          </button>
          <button
            onClick={handleConfirm}
            disabled={!selectedAccount}
            className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:bg-gray-400 disabled:cursor-not-allowed"
          >
            Избери
          </button>
        </div>
      </div>
    </div>
  );
}