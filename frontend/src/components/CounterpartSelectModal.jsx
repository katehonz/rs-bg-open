import { useState, useEffect, useRef } from 'react';

export default function CounterpartSelectModal({ 
  show, 
  counterparts = [], 
  currentCounterpartId = null,
  onSelect, 
  onClose,
  onAddNew 
}) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCounterpart, setSelectedCounterpart] = useState(null);
  const searchInputRef = useRef(null);

  // Focus search input when modal opens
  useEffect(() => {
    if (show && searchInputRef.current) {
      searchInputRef.current.focus();
      setSearchQuery('');
      
      // Find and select current counterpart if provided
      if (currentCounterpartId) {
        const counterpart = counterparts.find(c => c.id === currentCounterpartId);
        if (counterpart) {
          setSelectedCounterpart(counterpart);
        }
      }
    }
  }, [show, currentCounterpartId, counterparts]);

  // Filter and sort counterparts based on search query
  const filteredCounterparts = searchQuery.trim() 
    ? counterparts.filter(counterpart => {
        const query = searchQuery.toLowerCase().trim();
        const nameMatch = counterpart.name.toLowerCase().includes(query);
        const eikMatch = counterpart.eik && counterpart.eik.includes(query);
        const vatMatch = counterpart.vatNumber && counterpart.vatNumber.includes(query);
        return nameMatch || eikMatch || vatMatch;
      }).sort((a, b) => {
        const query = searchQuery.toLowerCase().trim();
        
        // First show exact matches by name
        const aNameExact = a.name.toLowerCase() === query;
        const bNameExact = b.name.toLowerCase() === query;
        if (aNameExact && !bNameExact) return -1;
        if (!aNameExact && bNameExact) return 1;
        
        // Then show names starting with search text
        const aNameStarts = a.name.toLowerCase().startsWith(query);
        const bNameStarts = b.name.toLowerCase().startsWith(query);
        if (aNameStarts && !bNameStarts) return -1;
        if (!aNameStarts && bNameStarts) return 1;
        
        // Finally sort by name
        return a.name.localeCompare(b.name);
      })
    : counterparts.sort((a, b) => a.name.localeCompare(b.name));

  const handleSelect = (counterpart) => {
    setSelectedCounterpart(counterpart);
  };

  const handleConfirm = () => {
    if (selectedCounterpart) {
      onSelect(selectedCounterpart);
      handleClose();
    }
  };

  const handleClose = () => {
    setSearchQuery('');
    setSelectedCounterpart(null);
    onClose();
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Escape') {
      handleClose();
    } else if (e.key === 'Enter' && selectedCounterpart) {
      handleConfirm();
    }
  };

  const handleAddNew = () => {
    onAddNew();
    handleClose();
  };

  if (!show) return null;

  return (
    <div 
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onClick={handleClose}
      onKeyDown={handleKeyDown}
    >
      <div 
        className="bg-white rounded-lg w-full max-w-4xl max-h-[80vh] flex flex-col shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h3 className="text-xl font-semibold text-gray-900">Избор на контрагент</h3>
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
              placeholder="Търсене по име, БУЛСТАТ или ДДС номер..."
              className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
        </div>

        {/* Counterparts List */}
        <div className="flex-1 overflow-y-auto p-4">
          {filteredCounterparts.length > 0 ? (
            <div className="space-y-2">
              {filteredCounterparts.map((counterpart) => (
                <div
                  key={counterpart.id}
                  onClick={() => handleSelect(counterpart)}
                  className={`p-4 rounded-lg border cursor-pointer transition-all ${
                    selectedCounterpart?.id === counterpart.id
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-3">
                        <span className="font-semibold text-gray-900">
                          {counterpart.name}
                        </span>
                        {counterpart.isVatRegistered && (
                          <span className="inline-flex px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 text-green-700">
                            ДДС
                          </span>
                        )}
                      </div>
                      
                      <div className="mt-1 space-y-1">
                        {counterpart.eik && (
                          <div className="text-sm text-gray-600">
                            <span className="font-medium">БУЛСТАТ:</span> {counterpart.eik}
                          </div>
                        )}
                        {counterpart.vatNumber && counterpart.isVatRegistered && (
                          <div className="text-sm text-gray-600">
                            <span className="font-medium">ДДС номер:</span> {counterpart.vatNumber}
                          </div>
                        )}
                        {(counterpart.street || counterpart.address) && (
                          <div className="text-sm text-gray-500">
                            {counterpart.street || counterpart.address}
                            {counterpart.postalCode && `, ${counterpart.postalCode}`}
                            {counterpart.city && `, ${counterpart.city}`}
                            {counterpart.country && counterpart.country !== 'България' && `, ${counterpart.country}`}
                          </div>
                        )}
                      </div>
                    </div>
                    {selectedCounterpart?.id === counterpart.id && (
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
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
              <p className="mt-2 text-sm text-gray-500">
                {searchQuery ? 'Няма намерени контрагенти' : 'Няма добавени контрагенти'}
              </p>
              {searchQuery && (
                <button
                  onClick={handleAddNew}
                  className="mt-3 inline-flex items-center px-3 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700"
                >
                  <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
                  </svg>
                  Добави нов контрагент
                </button>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-gray-200">
          <button
            onClick={handleAddNew}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700"
          >
            <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
            </svg>
            Добави нов
          </button>
          
          <div className="flex items-center space-x-3">
            <button
              onClick={handleClose}
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Отказ
            </button>
            <button
              onClick={handleConfirm}
              disabled={!selectedCounterpart}
              className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:bg-gray-400 disabled:cursor-not-allowed"
            >
              Избери
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
