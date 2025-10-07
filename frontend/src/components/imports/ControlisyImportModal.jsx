import { useState, useEffect } from 'react';

export default function ControlisyImportModal({ show, onClose, files = [], onImportComplete }) {
  const [step, setStep] = useState(1);
  const [parsedData, setParsedData] = useState(null);
  const [accountMapping, setAccountMapping] = useState({});
  const [importResults, setImportResults] = useState(null);
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState([]);

  // Parse Controlisy XML files using REST API
  const parseControlisyFiles = async (files) => {
    setLoading(true);
    try {
      // Read file content and parse each XML
      const parsedFiles = [];
      
      for (const file of files) {
        const xmlContent = await readFileAsText(file.file);
        
        // Call REST API to parse XML
        const response = await fetch('http://localhost:8080/api/controlisy/parse', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            file_name: file.name,
            xml_content: btoa(unescape(encodeURIComponent(xmlContent))) // UTF-8 safe base64 encoding
          })
        });
        
        if (response.ok) {
          const result = await response.json();
          if (result.success && result.data) {
            parsedFiles.push({
              fileName: file.name,
              data: result.data,
              type: result.document_type // 'sale' or 'purchase' from API
            });
          }
        }
      }
      
      // Combine all parsed data
      const combinedData = {
        batchId: 'CTRL-' + Date.now(),
        importDate: new Date().toISOString(),
        source: 'controlisy',
        files: parsedFiles,
        contractors: [],
        documents: [],
        statistics: {
          totalDocuments: 0,
          totalEntries: 0,
          balancedDocuments: 0,
          unbalancedDocuments: 0,
          uniqueAccounts: []
        }
      };
      
      // Aggregate contractors and documents from all files
      const contractorMap = new Map();
      const accountSet = new Set();
      
      for (const file of parsedFiles) {
        // Add contractors
        if (file.data.contractors) {
          for (const contractor of file.data.contractors) {
            contractorMap.set(contractor.ca_contractor_id, {
              id: contractor.ca_contractor_id,
              name: contractor.contractor_name,
              eik: contractor.contractor_eik,
              vatNumber: contractor.contractor_vat_number
            });
          }
        }
        
        // Add documents with proper structure
        if (file.data.documents) {
          for (const doc of file.data.documents) {
            const totalAmount = parseFloat(doc.total_amount_bgn || 0);
            const netAmount = parseFloat(doc.net_amount_bgn || 0);
            const vatAmount = parseFloat(doc.vat_amount_bgn || 0);
            
            // Calculate entries from accountings
            const entries = [];
            let isBalanced = true;
            let totalDebit = 0;
            let totalCredit = 0;
            
            if (doc.accountings) {
              for (const accounting of doc.accountings) {
                if (accounting.accounting_details) {
                  for (const detail of accounting.accounting_details) {
                    const amount = parseFloat(accounting.amount_bgn || 0);
                    const entry = {
                      account: detail.account_number,
                      accountName: detail.account_name,
                      debit: detail.direction === 'Debit' ? amount : 0,
                      credit: detail.direction === 'Credit' ? amount : 0,
                      direction: detail.direction,
                      quantity: detail.quantity,
                      unit: detail.unit,
                      contractorName: detail.contractor_name
                    };
                    entries.push(entry);
                    totalDebit += entry.debit;
                    totalCredit += entry.credit;
                    accountSet.add(detail.account_number);
                  }
                }
              }
            }
            
            isBalanced = Math.abs(totalDebit - totalCredit) < 0.01;
            
            combinedData.documents.push({
              docNumber: doc.document_number,
              docDate: doc.document_date,
              docType: file.type,
              description: doc.reason,
              totalAmount,
              netAmount,
              vatAmount,
              contractorId: doc.ca_contractor_id,
              contractorName: contractorMap.get(doc.ca_contractor_id)?.name || '',
              isBalanced,
              entries,
              vatData: doc.vat_data
            });
          }
        }
      }
      
      // Convert maps/sets to arrays
      combinedData.contractors = Array.from(contractorMap.values());
      combinedData.statistics.uniqueAccounts = Array.from(accountSet);
      combinedData.statistics.totalDocuments = combinedData.documents.length;
      combinedData.statistics.totalEntries = combinedData.documents.reduce((sum, doc) => sum + doc.entries.length, 0);
      combinedData.statistics.balancedDocuments = combinedData.documents.filter(d => d.isBalanced).length;
      combinedData.statistics.unbalancedDocuments = combinedData.documents.filter(d => !d.isBalanced).length;
      
      setParsedData(combinedData);
      
      // Auto-generate account mapping suggestions
      const mapping = {};
      combinedData.statistics.uniqueAccounts.forEach(account => {
        mapping[account] = account; // Default 1:1 mapping
      });
      setAccountMapping(mapping);
      
      setStep(2);
    } catch (error) {
      setErrors([{ message: 'Грешка при парсване на файловете: ' + error.message }]);
    } finally {
      setLoading(false);
    }
  };

  // Helper function to read file as text
  const readFileAsText = (file) => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => resolve(e.target.result);
      reader.onerror = reject;
      reader.readAsText(file);
    });
  };

  const stageForReview = async () => {
    setLoading(true);
    try {
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      const stagedImports = [];
      
      // Import each file to staging (database) - they will have status 'staged'
      for (const file of parsedData.files) {
        // First get raw XML content for the file
        const xmlFile = files.find(f => f.name === file.fileName);
        const xmlContent = xmlFile ? await readFileAsText(xmlFile.file) : '';
        
        // Use REST API for file import
        const response = await fetch('http://localhost:8080/api/controlisy/import', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            company_id: companyId,
            file_name: file.fileName,
            xml_content: btoa(unescape(encodeURIComponent(xmlContent))) // UTF-8 safe base64 encoding
          })
        });
        
        if (response.ok) {
          const result = await response.json();
          if (result.success) {
            stagedImports.push({
              importId: result.import_id,
              fileName: file.fileName,
              message: result.message,
              documentsCount: result.documents_count,
              contractorsCount: result.contractors_count
            });
          }
        } else {
          const error = await response.json();
          setErrors(prev => [...prev, `Failed to import ${file.fileName}: ${error.error || 'Unknown error'}`]);
        }
      }
      
      const results = {
        staged: stagedImports.length,
        errors: 0,
        warnings: 0,
        imports: stagedImports,
        details: stagedImports.map(s => ({
          importId: s.importId,
          fileName: s.fileName,
          status: 'staged',
          message: s.message,
          documentsCount: s.documentsCount,
          contractorsCount: s.contractorsCount
        }))
      };
      
      setImportResults(results);
      setStep(4);
    } catch (error) {
      setErrors([{ message: 'Грешка при импорт в staging: ' + error.message }]);
    } finally {
      setLoading(false);
    }
  };

  const updateAccountMapping = (oldAccount, newAccount) => {
    setAccountMapping(prev => ({
      ...prev,
      [oldAccount]: newAccount
    }));
  };

  const resetModal = () => {
    setStep(1);
    setParsedData(null);
    setAccountMapping({});
    setImportResults(null);
    setErrors([]);
    setLoading(false);
  };

  const handleClose = () => {
    resetModal();
    onClose();
  };

  // Start parsing when files are provided
  useEffect(() => {
    if (show && files.length > 0 && step === 1) {
      parseControlisyFiles(files);
    }
  }, [show, files, step]);

  if (!show) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 bg-purple-50">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="text-2xl">📄</div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900">
                  Controlisy импорт
                </h3>
                <p className="text-sm text-gray-600">
                  Стъпка {step} от 4
                </p>
              </div>
            </div>
            <button
              onClick={handleClose}
              className="p-2 hover:bg-gray-100 rounded-full"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        {/* Progress bar */}
        <div className="px-6 py-2 bg-gray-50">
          <div className="flex items-center space-x-2">
            {[1, 2, 3, 4].map((num) => (
              <div
                key={num}
                className={`flex-1 h-2 rounded-full ${
                  num <= step ? 'bg-purple-500' : 'bg-gray-200'
                }`}
              />
            ))}
          </div>
          <div className="flex justify-between text-xs text-gray-500 mt-1">
            <span>Парсване</span>
            <span>Преглед</span>
            <span>Мапиране</span>
            <span>Импорт</span>
          </div>
        </div>

        <div className="p-6 overflow-y-auto max-h-[calc(90vh-160px)]">
          {/* Step 1: Parsing */}
          {step === 1 && (
            <div className="text-center py-8">
              <div className="animate-spin w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full mx-auto"></div>
              <h4 className="mt-4 text-lg font-medium">Обработка на файловете...</h4>
              <p className="text-gray-600 mt-2">
                Парсиране на XML структурата от Controlisy
              </p>
              <div className="mt-4 text-sm text-gray-500">
                {files.map(file => (
                  <div key={file.name} className="flex items-center justify-center space-x-2">
                    <span>📄</span>
                    <span>{file.name}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Step 2: Review */}
          {step === 2 && parsedData && (
            <div className="space-y-6">
              <div>
                <h4 className="text-lg font-semibold mb-4">Преглед на данните за импорт</h4>
                
                {/* Statistics */}
                <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
                  <div className="bg-blue-50 p-4 rounded-lg">
                    <div className="text-2xl font-bold text-blue-600">
                      {parsedData.statistics.totalDocuments}
                    </div>
                    <div className="text-sm text-blue-800">Документи</div>
                  </div>
                  <div className="bg-green-50 p-4 rounded-lg">
                    <div className="text-2xl font-bold text-green-600">
                      {parsedData.statistics.totalEntries}
                    </div>
                    <div className="text-sm text-green-800">Записи</div>
                  </div>
                  <div className="bg-purple-50 p-4 rounded-lg">
                    <div className="text-2xl font-bold text-purple-600">
                      {parsedData.contractors.length}
                    </div>
                    <div className="text-sm text-purple-800">Контрагенти</div>
                  </div>
                  <div className="bg-yellow-50 p-4 rounded-lg">
                    <div className="text-2xl font-bold text-yellow-600">
                      {parsedData.statistics.uniqueAccounts.length}
                    </div>
                    <div className="text-sm text-yellow-800">Сметки</div>
                  </div>
                </div>

                {/* Documents preview */}
                <div className="bg-gray-50 rounded-lg p-4">
                  <h5 className="font-medium mb-3">Документи за импорт:</h5>
                  <div className="space-y-3 max-h-96 overflow-y-auto">
                    {parsedData.documents.slice(0, 10).map((doc, index) => (
                      <div key={index} className="bg-white p-3 rounded border">
                        <div className="flex justify-between items-start">
                          <div>
                            <div className="font-medium">{doc.docNumber}</div>
                            <div className="text-sm text-gray-600">{doc.description}</div>
                            <div className="text-xs text-gray-500">
                              {doc.docDate} • {doc.entries.length} записа • {doc.contractorName}
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="font-medium">{doc.totalAmount?.toFixed(2)} лв</div>
                            <div className={`text-xs px-2 py-1 rounded-full ${
                              doc.isBalanced 
                                ? 'bg-green-100 text-green-800' 
                                : 'bg-red-100 text-red-800'
                            }`}>
                              {doc.isBalanced ? 'Балансиран' : 'Не е балансиран'}
                            </div>
                          </div>
                        </div>
                        {doc.vatData && (
                          <div className="mt-2 text-xs text-gray-600">
                            ДДС: {doc.vatAmount?.toFixed(2)} лв ({((doc.vatAmount/doc.netAmount)*100).toFixed(0)}%)
                          </div>
                        )}
                      </div>
                    ))}
                    {parsedData.documents.length > 10 && (
                      <div className="text-center text-sm text-gray-500">
                        ... и още {parsedData.documents.length - 10} документа
                      </div>
                    )}
                  </div>
                </div>

                <div className="flex justify-end space-x-3 mt-6">
                  <button
                    onClick={handleClose}
                    className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
                  >
                    Отказ
                  </button>
                  <button
                    onClick={() => setStep(3)}
                    className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700"
                  >
                    Продължи към мапиране
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Step 3: Account Mapping */}
          {step === 3 && parsedData && (
            <div className="space-y-6">
              <div>
                <h4 className="text-lg font-semibold mb-4">Мапиране на сметки</h4>
                <p className="text-gray-600 mb-4">
                  Свържете сметките от Controlisy с вашите сметки от сметкоплана
                </p>

                <div className="space-y-3 max-h-96 overflow-y-auto">
                  {parsedData.statistics.uniqueAccounts.map(account => (
                    <div key={account} className="flex items-center space-x-4 p-3 border rounded-lg">
                      <div className="min-w-0 flex-1">
                        <div className="font-medium">Controlisy сметка: {account}</div>
                        <div className="text-sm text-gray-600">
                          {/* Find account name from first occurrence */}
                          {parsedData.documents
                            .flatMap(doc => doc.entries)
                            .find(entry => entry.account === account)?.accountName || 'Без име'}
                        </div>
                      </div>
                      <div className="text-2xl text-gray-400">→</div>
                      <div className="min-w-0 flex-1">
                        <input
                          type="text"
                          value={accountMapping[account] || ''}
                          onChange={(e) => updateAccountMapping(account, e.target.value)}
                          placeholder="Въведете номер на сметка"
                          className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                        />
                        <div className="text-xs text-gray-500 mt-1">
                          Ваша сметка от сметкоплана
                        </div>
                      </div>
                    </div>
                  ))}
                </div>

                <div className="bg-blue-50 p-4 rounded-lg mt-4">
                  <div className="flex items-start space-x-2">
                    <div className="text-blue-500">💡</div>
                    <div className="text-sm text-blue-800">
                      <div className="font-medium mb-1">Съвет:</div>
                      <div>Можете да запазите това мапиране за бъдещи импорти от Controlisy. 
                      Ако оставите празно поле, ще се използва оригиналният номер на сметката.</div>
                    </div>
                  </div>
                </div>

                <div className="flex justify-end space-x-3 mt-6">
                  <button
                    onClick={() => setStep(2)}
                    className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
                  >
                    Назад
                  </button>
                  <button
                    onClick={stageForReview}
                    disabled={loading}
                    className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:opacity-50"
                  >
                    {loading ? 'Подготвя за преглед...' : 'Подготви за преглед'}
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Step 4: Staging Results */}
          {step === 4 && importResults && (
            <div className="space-y-6">
              <div className="text-center">
                <div className="text-6xl mb-4">📋</div>
                <h4 className="text-lg font-semibold">Файловете са подготвени за преглед</h4>
                <p className="text-gray-600 mt-2">
                  {importResults.staged} файла са готови за преглед и редакция
                </p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="bg-blue-50 p-4 rounded-lg text-center">
                  <div className="text-2xl font-bold text-blue-600">
                    {importResults.staged}
                  </div>
                  <div className="text-sm text-blue-800">Подготвени</div>
                </div>
                <div className="bg-red-50 p-4 rounded-lg text-center">
                  <div className="text-2xl font-bold text-red-600">
                    {importResults.errors}
                  </div>
                  <div className="text-sm text-red-800">Грешки</div>
                </div>
                <div className="bg-yellow-50 p-4 rounded-lg text-center">
                  <div className="text-2xl font-bold text-yellow-600">
                    {importResults.warnings}
                  </div>
                  <div className="text-sm text-yellow-800">Предупреждения</div>
                </div>
              </div>

              <div className="space-y-2">
                {importResults.details.map((detail, index) => (
                  <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded">
                    <div className="flex-1">
                      <div className="font-medium">{detail.fileName}</div>
                      <div className="text-sm text-gray-600">
                        {detail.documentsCount} документа, {detail.contractorsCount} контрагента
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className={`px-2 py-1 text-xs rounded-full ${
                        detail.status === 'staged' 
                          ? 'bg-blue-100 text-blue-800'
                          : 'bg-red-100 text-red-800'
                      }`}>
                        {detail.status === 'staged' ? 'Готов за преглед' : 'Грешка'}
                      </span>
                      {detail.status === 'staged' && (
                        <span className="text-xs text-gray-500">ID: {detail.importId}</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              <div className="bg-blue-50 p-4 rounded-lg">
                <div className="flex items-start space-x-2">
                  <div className="text-blue-500">🔍</div>
                  <div className="text-sm text-blue-800">
                    <div className="font-medium mb-1">Следващи стъпки:</div>
                    <div>Отидете в секцията "Импорти" → "Controlisy" за да прегледате и редактирате данните преди финалния импорт в счетоводните таблици.</div>
                  </div>
                </div>
              </div>

              <div className="flex justify-end space-x-3">
                <button
                  onClick={() => {
                    if (onImportComplete) {
                      onImportComplete(importResults);
                    }
                    handleClose();
                  }}
                  className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700"
                >
                  Затвори
                </button>
              </div>
            </div>
          )}

          {/* Errors */}
          {errors.length > 0 && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <h5 className="font-medium text-red-800 mb-2">Възникнаха грешки:</h5>
              <ul className="text-sm text-red-700 space-y-1">
                {errors.map((error, index) => (
                  <li key={index}>• {error.message}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}