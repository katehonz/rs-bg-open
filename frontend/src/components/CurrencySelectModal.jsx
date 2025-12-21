import { useState, useEffect } from 'react';

export default function CurrencySelectModal({ show, currencies, currentCurrencyCode, onSelect, onClose }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [filteredCurrencies, setFilteredCurrencies] = useState([]);

  // Always include BGN
  const allCurrencies = [
    { code: 'BGN', name: 'Български лев', nameBg: 'Български лев', isActive: true },
    ...currencies
  ];

  useEffect(() => {
    if (show) {
      setSearchTerm('');
      filterCurrencies('');
    }
  }, [show, currencies]);

  const filterCurrencies = (term) => {
    const lowerTerm = term.toLowerCase();
    const filtered = allCurrencies.filter(currency =>
      currency.code.toLowerCase().includes(lowerTerm) ||
      (currency.name && currency.name.toLowerCase().includes(lowerTerm)) ||
      (currency.nameBg && currency.nameBg.toLowerCase().includes(lowerTerm))
    );
    setFilteredCurrencies(filtered);
  };

  const handleSearch = (e) => {
    const term = e.target.value;
    setSearchTerm(term);
    filterCurrencies(term);
  };

  const handleSelect = (currency) => {
    onSelect(currency.code);
  };

  if (!show) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-900">Избери валута</h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Search */}
          <div className="mt-4">
            <input
              type="text"
              value={searchTerm}
              onChange={handleSearch}
              placeholder="Търси по код или име..."
              className="w-full px-4 py-2 border border-gray-300 rounded-md"
              autoFocus
            />
          </div>
        </div>

        {/* Currency List */}
        <div className="flex-1 overflow-y-auto px-6 py-4">
          {filteredCurrencies.length > 0 ? (
            <div className="space-y-2">
              {filteredCurrencies.map((currency) => (
                <button
                  key={currency.code}
                  onClick={() => handleSelect(currency)}
                  className={`w-full text-left px-4 py-3 rounded-lg border-2 transition-colors ${
                    currentCurrencyCode === currency.code
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-blue-300 hover:bg-gray-50'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-mono font-semibold text-gray-900">
                        {currency.code}
                      </div>
                      <div className="text-sm text-gray-600">
                        {currency.nameBg || currency.name}
                      </div>
                    </div>
                    {currentCurrencyCode === currency.code && (
                      <div className="text-blue-500">
                        <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                        </svg>
                      </div>
                    )}
                  </div>
                </button>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-500">
              Няма намерени валути
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
          >
            Затвори
          </button>
        </div>
      </div>
    </div>
  );
}
