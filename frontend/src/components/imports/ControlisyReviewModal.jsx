import { useState, useEffect } from 'react';

export default function ControlisyReviewModal({
  show,
  onClose,
  selectedImport,
  reviewData,
  onDataUpdate,
  onMarkReviewed
}) {
  const [editingData, setEditingData] = useState(reviewData);
  const [activeTab, setActiveTab] = useState('overview');
  const [saving, setSaving] = useState(false);
  const [viesValidation, setViesValidation] = useState({}); // Store VIES validation results
  const [validatingVies, setValidatingVies] = useState({});  // Track validation in progress

  // Update editingData when reviewData changes
  useEffect(() => {
    setEditingData(reviewData);
  }, [reviewData]);

  if (!show || !reviewData || !selectedImport) return null;
  
  // Debug logging for status
  console.log('ControlisyReviewModal - selectedImport.status:', selectedImport.status);

  const handleDataChange = (path, value) => {
    const newData = { ...editingData };
    
    // Simple path handling for nested updates
    const keys = path.split('.');
    let current = newData;
    
    for (let i = 0; i < keys.length - 1; i++) {
      current = current[keys[i]];
    }
    
    current[keys[keys.length - 1]] = value;
    setEditingData(newData);
  };

  const saveChanges = async () => {
    setSaving(true);
    try {
      const response = await fetch('/graphql', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: `
            mutation UpdateStagedImportData($importId: Int!, $updatedData: String!) {
              updateStagedImportData(importId: $importId, updatedData: $updatedData) {
                success
                message
              }
            }
          `,
          variables: {
            importId: selectedImport.id,
            updatedData: JSON.stringify(editingData)
          }
        })
      });
      
      const result = await response.json();
      if (result.data?.updateStagedImportData?.success) {
        if (onDataUpdate) onDataUpdate(editingData);
      }
    } catch (error) {
      console.error('Error saving changes:', error);
    } finally {
      setSaving(false);
    }
  };

  const markAsReviewed = async () => {
    try {
      const userId = parseInt(localStorage.getItem('currentUserId')) || 1;
      const response = await fetch('/graphql', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: `
            mutation MarkImportAsReviewed($importId: Int!, $userId: Int!) {
              markImportAsReviewed(importId: $importId, userId: $userId) {
                success
                message
              }
            }
          `,
          variables: {
            importId: selectedImport.id,
            userId: userId
          }
        })
      });
      
      const result = await response.json();
      if (result.data?.markImportAsReviewed?.success) {
        if (onMarkReviewed) onMarkReviewed();
        onClose();
      }
    } catch (error) {
      console.error('Error marking as reviewed:', error);
    }
  };

  // VIES validation function
  const validateVatNumber = async (vatNumber, contractorIndex) => {
    if (!vatNumber || vatNumber.length < 2) {
      alert('Невалиден ДДС номер');
      return;
    }

    // Extract country code and number (e.g., BG123456789 -> BG + 123456789)
    const countryCode = vatNumber.substring(0, 2).toUpperCase();
    const vatNum = vatNumber.substring(2);

    setValidatingVies(prev => ({ ...prev, [contractorIndex]: true }));

    try {
      // Call VIES API через backend или директно
      const response = await fetch(`http://ec.europa.eu/taxation_customs/vies/rest-api/ms/${countryCode}/vat/${vatNum}`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' }
      });

      if (response.ok) {
        const result = await response.json();
        setViesValidation(prev => ({
          ...prev,
          [contractorIndex]: {
            valid: result.valid || false,
            name: result.name || '',
            address: result.address || '',
            countryCode: countryCode,
            vatNumber: vatNum,
            requestDate: new Date().toISOString()
          }
        }));
      } else {
        setViesValidation(prev => ({
          ...prev,
          [contractorIndex]: {
            valid: false,
            error: 'Грешка при проверка във VIES'
          }
        }));
      }
    } catch (error) {
      console.error('VIES validation error:', error);
      setViesValidation(prev => ({
        ...prev,
        [contractorIndex]: {
          valid: false,
          error: 'Невъзможна връзка с VIES система'
        }
      }));
    } finally {
      setValidatingVies(prev => ({ ...prev, [contractorIndex]: false }));
    }
  };

  const processImport = async () => {
    try {
      const response = await fetch(`/api/controlisy/process/${selectedImport.id}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });

      if (response.ok) {
        const result = await response.json();
        if (result.success) {
          alert(`Импортът е завършен успешно! Документите са създадени в счетоводните записи.`);
          if (onMarkReviewed) onMarkReviewed(); // Refresh the import list
          onClose();

          // Optionally redirect to journal entries
          if (window.confirm('Искате ли да видите създадените счетоводни записи?')) {
            window.location.href = '/accounting/journal-entries';
          }
        }
      } else {
        const error = await response.json();
        alert(`Грешка при импорт: ${error.error || 'Неизвестна грешка'}`);
      }
    } catch (error) {
      console.error('Error processing import:', error);
      alert('Грешка при импорт: ' + error.message);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-6xl w-full mx-4 max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 bg-blue-50">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="text-2xl">🔍</div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900">
                  Преглед и редакция на импорт
                </h3>
                <p className="text-sm text-gray-600">
                  {selectedImport.fileName} • {editingData?.contractors?.length || 0} контрагенти • {editingData?.documents?.length || 0} документа
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 hover:bg-gray-100 rounded-full"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        {/* Tabs */}
        <div className="px-6 py-2 bg-gray-50 border-b border-gray-200">
          <div className="flex space-x-4">
            {[
              { id: 'overview', label: 'Общ преглед', icon: '📊' },
              { id: 'contractors', label: 'Контрагенти', icon: '🏢' },
              { id: 'documents', label: 'Документи', icon: '📄' },
              { id: 'accounts', label: 'Сметки', icon: '💰' }
            ].map(tab => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center space-x-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === tab.id
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
                }`}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="p-6 overflow-y-auto max-h-[calc(90vh-200px)]">
          {/* Overview Tab */}
          {activeTab === 'overview' && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div className="bg-blue-50 p-4 rounded-lg">
                  <div className="text-2xl font-bold text-blue-600">
                    {editingData?.documents?.length || 0}
                  </div>
                  <div className="text-sm text-blue-800">Документи</div>
                </div>
                <div className="bg-purple-50 p-4 rounded-lg">
                  <div className="text-2xl font-bold text-purple-600">
                    {editingData?.contractors?.length || 0}
                  </div>
                  <div className="text-sm text-purple-800">Контрагенти</div>
                </div>
                <div className="bg-green-50 p-4 rounded-lg">
                  <div className="text-2xl font-bold text-green-600">
                    {editingData?.documents?.reduce((sum, doc) => sum + (parseFloat(doc.total_amount_bgn) || 0), 0)?.toFixed(2) || '0.00'}
                  </div>
                  <div className="text-sm text-green-800">Общо сума (лв)</div>
                </div>
                <div className="bg-yellow-50 p-4 rounded-lg">
                  <div className="text-2xl font-bold text-yellow-600">
                    {editingData?.documents?.reduce((sum, doc) => sum + (parseFloat(doc.vat_amount_bgn) || 0), 0)?.toFixed(2) || '0.00'}
                  </div>
                  <div className="text-sm text-yellow-800">ДДС (лв)</div>
                </div>
              </div>
              
              <div className="bg-gray-50 p-4 rounded-lg">
                <h5 className="font-medium mb-3">Статус на импорта:</h5>
                <div className="flex items-center space-x-2">
                  <span className={`px-3 py-1 text-sm rounded-full font-medium ${
                    selectedImport.status === 'staged' 
                      ? 'bg-blue-100 text-blue-800'
                      : 'bg-yellow-100 text-yellow-800'
                  }`}>
                    {selectedImport.status === 'staged' ? 'За преглед' : 'Прегледан'}
                  </span>
                  <span className="text-sm text-gray-600">
                    Импортиран на {new Date(selectedImport.import_date).toLocaleString('bg-BG')}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Contractors Tab */}
          {activeTab === 'contractors' && (
            <div className="space-y-4">
              <h5 className="font-medium">Контрагенти ({editingData?.contractors?.length || 0})</h5>
              <div className="space-y-3 max-h-96 overflow-y-auto">
                {editingData?.contractors?.map((contractor, index) => {
                  const viesResult = viesValidation[index];
                  const isValidating = validatingVies[index];

                  return (
                    <div key={index} className="bg-gray-50 p-4 rounded-lg border">
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">Име</label>
                          <input
                            type="text"
                            value={contractor.contractor_name || ''}
                            onChange={(e) => handleDataChange(`contractors.${index}.contractor_name`, e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">ЕИК</label>
                          <input
                            type="text"
                            value={contractor.contractor_eik || ''}
                            onChange={(e) => handleDataChange(`contractors.${index}.contractor_eik`, e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">ДДС номер</label>
                          <div className="flex space-x-2">
                            <input
                              type="text"
                              value={contractor.contractor_vat_number || ''}
                              onChange={(e) => handleDataChange(`contractors.${index}.contractor_vat_number`, e.target.value)}
                              className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm font-mono"
                            />
                            <button
                              onClick={() => validateVatNumber(contractor.contractor_vat_number, index)}
                              disabled={isValidating || !contractor.contractor_vat_number}
                              className="px-3 py-2 bg-blue-600 text-white rounded-md text-sm hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
                              title="Провери в VIES"
                            >
                              {isValidating ? '⏳' : '🇪🇺'}
                            </button>
                          </div>
                          {viesResult && (
                            <div className={`mt-2 p-2 rounded text-xs ${
                              viesResult.valid
                                ? 'bg-green-100 border border-green-300 text-green-800'
                                : 'bg-red-100 border border-red-300 text-red-800'
                            }`}>
                              {viesResult.valid ? (
                                <>
                                  <div className="font-medium">✓ Валиден ДДС номер във VIES</div>
                                  {viesResult.name && <div className="mt-1">Име: {viesResult.name}</div>}
                                  {viesResult.address && <div className="mt-1">Адрес: {viesResult.address}</div>}
                                </>
                              ) : (
                                <div className="font-medium">✗ {viesResult.error || 'Невалиден ДДС номер във VIES'}</div>
                              )}
                            </div>
                          )}
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">ID в системата</label>
                          <input
                            type="text"
                            value={contractor.ca_contractor_id || ''}
                            readOnly
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-100"
                          />
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Documents Tab */}
          {activeTab === 'documents' && (
            <div className="space-y-4">
              <h5 className="font-medium">Документи ({editingData?.documents?.length || 0})</h5>
              <div className="space-y-3 max-h-96 overflow-y-auto">
                {editingData?.documents?.map((document, index) => {
                  // Find contractor for this document
                  const contractor = editingData?.contractors?.find(c => c.ca_contractor_id === document.ca_contractor_id);

                  return (
                    <div key={index} className="bg-gray-50 p-4 rounded-lg border">
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">Номер документ</label>
                          <input
                            type="text"
                            value={document.document_number || ''}
                            onChange={(e) => handleDataChange(`documents.${index}.document_number`, e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">Дата</label>
                          <input
                            type="date"
                            value={document.document_date || ''}
                            onChange={(e) => handleDataChange(`documents.${index}.document_date`, e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">Обща сума</label>
                          <input
                            type="number"
                            step="0.01"
                            value={parseFloat(document.total_amount_bgn) || ''}
                            onChange={(e) => handleDataChange(`documents.${index}.total_amount_bgn`, e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                        </div>

                        {/* Contractor Info - ДДС номер и ЕИК */}
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">Контрагент</label>
                          <input
                            type="text"
                            value={contractor?.contractor_name || 'Неизвестен'}
                            readOnly
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-100"
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">ЕИК</label>
                          <input
                            type="text"
                            value={contractor?.contractor_eik || '-'}
                            readOnly
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-100"
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">ДДС номер</label>
                          <input
                            type="text"
                            value={contractor?.contractor_vat_number || '-'}
                            readOnly
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-100 font-mono"
                          />
                        </div>

                        <div className="md:col-span-3">
                          <label className="block text-sm font-medium text-gray-700 mb-1">Основание</label>
                          <textarea
                            value={document.reason || ''}
                            onChange={(e) => handleDataChange(`documents.${index}.reason`, e.target.value)}
                            rows={2}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Accounts Tab */}
          {activeTab === 'accounts' && (
            <div className="space-y-4">
              <h5 className="font-medium">Използвани сметки</h5>
              <div className="bg-gray-50 p-4 rounded-lg">
                <div className="text-sm text-gray-600 mb-4">
                  Преглед на всички сметки, които ще бъдат засегнати при импорта
                </div>
                <div className="space-y-2">
                  {editingData?.documents?.flatMap(doc => 
                    doc.accountings?.flatMap(acc => 
                      acc.accounting_details?.map(detail => ({
                        account: detail.account_number,
                        name: detail.account_name,
                        direction: detail.direction
                      })) || []
                    ) || []
                  ).filter((value, index, self) => 
                    index === self.findIndex(item => item.account === value.account)
                  ).map((account, index) => (
                    <div key={index} className="flex items-center justify-between p-3 bg-white rounded border">
                      <div>
                        <div className="font-medium">{account.account}</div>
                        <div className="text-sm text-gray-600">{account.name}</div>
                      </div>
                      <div className="text-xs text-gray-500">
                        Използва се в {editingData.documents.filter(doc =>
                          doc.accountings?.some(acc =>
                            acc.accounting_details?.some(detail => detail.account_number === account.account)
                          )
                        ).length} документа
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 bg-gray-50">
          <div className="flex justify-between">
            <button
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
            >
              Затвори
            </button>
            <div className="flex space-x-3">
              <button
                onClick={saveChanges}
                disabled={saving}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
              >
                {saving ? 'Запазва...' : 'Запази промените'}
              </button>
              {selectedImport.status === 'staged' && (
                <>
                  <button
                    onClick={markAsReviewed}
                    className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                  >
                    Потвърди като прегледан
                  </button>
                  <button
                    onClick={processImport}
                    className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700"
                  >
                    Импортирай директно
                  </button>
                </>
              )}
              
              {selectedImport.status === 'reviewed' && (
                <button
                  onClick={processImport}
                  className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700"
                >
                  Импортирай в счетоводството
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}