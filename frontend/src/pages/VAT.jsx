import { useState, useEffect, useRef } from 'react';

// GraphQL queries for VAT - updated to match backend schema
const VAT_RETURNS_QUERY = `
  query GetVATReturns($filter: VatReturnFilter, $limit: Int, $offset: Int) {
    vatReturns(filter: $filter, limit: $limit, offset: $offset) {
      id
      periodYear
      periodMonth
      periodFrom
      periodTo
      outputVatAmount
      inputVatAmount
      vatToPay
      vatToRefund
      baseAmount20
      vatAmount20
      baseAmount9
      vatAmount9
      baseAmount0
      exemptAmount
      status
      submittedAt
      submittedBy
      dueDate
      notes
      companyId
      createdAt
      updatedAt
    }
  }
`;

// Mutations for VAT operations
const CREATE_VAT_RETURN_MUTATION = `
  mutation CreateVatReturn($input: CreateVatReturnInput!) {
    createVatReturn(input: $input) {
      id
      periodYear
      periodMonth
      status
    }
  }
`;

const UPDATE_VAT_RETURN_MUTATION = `
  mutation UpdateVatReturn($id: Int!, $input: UpdateVatReturnInput!) {
    updateVatReturn(id: $id, input: $input) {
      id
      outputVatAmount
      inputVatAmount
      baseAmount20
      vatAmount20
      baseAmount9
      vatAmount9
      baseAmount0
      exemptAmount
    }
  }
`;

const SUBMIT_VAT_RETURN_MUTATION = `
  mutation SubmitVatReturn($id: Int!) {
    submitVatReturn(id: $id) {
      id
      status
      submittedAt
    }
  }
`;

const GENERATE_NAP_FILES_MUTATION = `
  mutation GenerateNapFiles($companyId: Int!, $year: Int!, $month: Int!) {
    generateVatFilesForNap(companyId: $companyId, year: $year, month: $month) {
      success
      deklarContent
      pokupkiContent
      prodagbiContent
    }
  }
`;

// GraphQL client function
async function graphqlRequest(query, variables = {}) {
  try {
    const response = await fetch('http://localhost:8080/graphql', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
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

export default function VAT() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [activeTab, setActiveTab] = useState('declaration'); // declaration, purchases, sales, analysis
  const [selectedPeriod, setSelectedPeriod] = useState(getCurrentPeriod());
  const [currentVATReturn, setCurrentVATReturn] = useState(null);
  const [vatDataSummary, setVatDataSummary] = useState(null);
  const [exportingNap, setExportingNap] = useState(false);
  const napFilesCacheRef = useRef({ period: null, companyId: null, files: null });

  // VAT Declaration state matching backend schema
  const [vatDeclaration, setVATDeclaration] = useState({
    id: null,
    periodYear: new Date().getFullYear(),
    periodMonth: new Date().getMonth() + 1,
    periodFrom: null,
    periodTo: null,
    outputVatAmount: 0,
    inputVatAmount: 0,
    vatToPay: 0,
    vatToRefund: 0,
    baseAmount20: 0,
    vatAmount20: 0,
    baseAmount9: 0,
    vatAmount9: 0,
    baseAmount0: 0,
    exemptAmount: 0,
    status: 'DRAFT',
    notes: '',
    companyId: parseInt(localStorage.getItem('currentCompanyId')) || 1
  });

  function getCurrentPeriod() {
    const now = new Date();
    const year = now.getFullYear();
    const month = (now.getMonth() + 1).toString().padStart(2, '0');
    return `${year}${month}`;
  }

  // Load VAT returns
  useEffect(() => {
    if (activeTab === 'declaration') {
      loadVATReturns();
    }
  }, [companyId, selectedPeriod, activeTab]);

  const loadVATReturns = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const periodYear = parseInt(selectedPeriod.substring(0, 4), 10);

      const data = await graphqlRequest(VAT_RETURNS_QUERY, {
        filter: {
          companyId: companyId,
          periodYear
        },
        limit: 50
      });
      // Load current period VAT return if exists
      const currentReturn = data.vatReturns?.find(vr => 
        vr.periodYear === vatDeclaration.periodYear && 
        vr.periodMonth === vatDeclaration.periodMonth
      );
      
      if (currentReturn) {
        setCurrentVATReturn(currentReturn);
        setVATDeclaration(prev => ({ ...prev, ...currentReturn }));
      }
    } catch (err) {
      setError('Грешка при зареждане на ДДС декларации: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  // Create or update VAT return
  const saveVATReturn = async () => {
    setLoading(true);
    setError(null);
    
    try {
      if (currentVATReturn) {
        // Update existing VAT return
        const data = await graphqlRequest(UPDATE_VAT_RETURN_MUTATION, {
          id: currentVATReturn.id,
          input: {
            outputVatAmount: vatDeclaration.outputVatAmount,
            inputVatAmount: vatDeclaration.inputVatAmount,
            baseAmount20: vatDeclaration.baseAmount20,
            vatAmount20: vatDeclaration.vatAmount20,
            baseAmount9: vatDeclaration.baseAmount9,
            vatAmount9: vatDeclaration.vatAmount9,
            baseAmount0: vatDeclaration.baseAmount0,
            exemptAmount: vatDeclaration.exemptAmount,
            notes: vatDeclaration.notes
          }
        });
        setCurrentVATReturn(data.updateVatReturn);
      } else {
        // Create new VAT return
        const data = await graphqlRequest(CREATE_VAT_RETURN_MUTATION, {
          input: {
            periodYear: vatDeclaration.periodYear,
            periodMonth: vatDeclaration.periodMonth,
            companyId: vatDeclaration.companyId,
            notes: vatDeclaration.notes
          }
        });
        setCurrentVATReturn(data.createVatReturn);
        setVATDeclaration(prev => ({ ...prev, id: data.createVatReturn.id }));
      }
      
      // Refresh the VAT returns list
      loadVATReturns();
      
    } catch (err) {
      setError('Грешка при запазване на ДДС декларация: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  // Submit VAT return
  const submitVATReturn = async () => {
    if (!currentVATReturn) {
      setError('Моля, първо запазете декларацията');
      return;
    }
    
    setLoading(true);
    setError(null);
    
    try {
      const data = await graphqlRequest(SUBMIT_VAT_RETURN_MUTATION, {
        id: currentVATReturn.id
      });
      setCurrentVATReturn(data.submitVatReturn);
      setVATDeclaration(prev => ({ 
        ...prev, 
        status: data.submitVatReturn.status,
        submittedAt: data.submitVatReturn.submittedAt
      }));
    } catch (err) {
      setError('Грешка при подаване на ДДС декларация: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const fetchNapFiles = async () => {
    const periodKey = selectedPeriod;
    if (!periodKey || periodKey.length !== 6) {
      throw new Error('Невалиден период за експорт');
    }

    if (
      napFilesCacheRef.current.period === periodKey &&
      napFilesCacheRef.current.companyId === companyId &&
      napFilesCacheRef.current.files
    ) {
      return napFilesCacheRef.current.files;
    }

    const year = parseInt(periodKey.substring(0, 4), 10);
    const month = parseInt(periodKey.substring(4, 6), 10);

    if (Number.isNaN(year) || Number.isNaN(month)) {
      throw new Error('Невалиден формат на периода');
    }

    const response = await graphqlRequest(GENERATE_NAP_FILES_MUTATION, {
      companyId,
      year,
      month,
    });

    const result = response?.generateVatFilesForNap;

    if (!result || !result.success) {
      throw new Error('Сървърът не върна валидни файлове');
    }

    napFilesCacheRef.current = { period: periodKey, companyId, files: result };

    return result;
  };

  const decodeBase64ToBytes = (base64String) => {
    if (base64String === undefined || base64String === null) {
      return null;
    }

    if (base64String === '') {
      return new Uint8Array();
    }

    const binaryString = atob(base64String);
    const bytes = new Uint8Array(binaryString.length);

    for (let i = 0; i < binaryString.length; i += 1) {
      bytes[i] = binaryString.charCodeAt(i);
    }

    return bytes;
  };

  const downloadBytesAsFile = (data, filename, mimeType = 'text/plain;charset=windows-1251') => {
    if (!data) {
      throw new Error(`Липсва съдържание за ${filename}`);
    }

    const blob = data instanceof Blob ? data : new Blob([data], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  // Export VAT declaration to TXT format (DEKLAR.TXT) using backend formatter
  const exportVATDeclaration = async () => {
    setExportingNap(true);
    setError(null);

    try {
      const files = await fetchNapFiles();
      const bytes = decodeBase64ToBytes(files.deklarContent);
      downloadBytesAsFile(bytes, 'DEKLAR.TXT');
    } catch (err) {
      setError('Грешка при експорт на декларация: ' + err.message);
    } finally {
      setExportingNap(false);
    }
  };

  // Export VAT purchases to TXT format (POKUPKI.TXT)
  const exportVATPurchases = async () => {
    setExportingNap(true);
    setError(null);

    try {
      const files = await fetchNapFiles();
      const bytes = decodeBase64ToBytes(files.pokupkiContent);
      downloadBytesAsFile(bytes, 'POKUPKI.TXT');
    } catch (err) {
      setError('Грешка при експорт на покупки: ' + err.message);
    } finally {
      setExportingNap(false);
    }
  };

  // Export VAT sales to TXT format (PRODAGBI.TXT)
  const exportVATSales = async () => {
    setExportingNap(true);
    setError(null);

    try {
      const files = await fetchNapFiles();
      const bytes = decodeBase64ToBytes(files.prodagbiContent);
      downloadBytesAsFile(bytes, 'PRODAGBI.TXT');
    } catch (err) {
      setError('Грешка при експорт на продажби: ' + err.message);
    } finally {
      setExportingNap(false);
    }
  };

  const exportNapZip = async () => {
    setExportingNap(true);
    setError(null);

    try {
      const files = await fetchNapFiles();
      const { default: JSZip } = await import('jszip');
      const zip = new JSZip();

      const fileMap = [
        { key: 'deklarContent', name: 'DEKLAR.TXT' },
        { key: 'pokupkiContent', name: 'POKUPKI.TXT' },
        { key: 'prodagbiContent', name: 'PRODAGBI.TXT' },
      ];

      let addedFiles = 0;

      fileMap.forEach(({ key, name }) => {
        const bytes = decodeBase64ToBytes(files[key]);
        if (bytes !== null) {
          zip.file(name, bytes);
          addedFiles += 1;
        }
      });

      if (addedFiles === 0) {
        throw new Error('Няма налични файлове за избрания период');
      }

      const zipBlob = await zip.generateAsync({ type: 'blob' });
      downloadBytesAsFile(zipBlob, `VAT_${selectedPeriod}.zip`, 'application/zip');
    } catch (err) {
      setError('Грешка при генериране на архив: ' + err.message);
    } finally {
      setExportingNap(false);
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
          <h1 className="text-2xl font-bold text-gray-900">ДДС Декларации</h1>
          <p className="mt-1 text-sm text-gray-500">
            Управление на ДДС декларации и отчети
          </p>
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

      {/* Navigation Tabs */}
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8">
          {[
            { key: 'declaration', label: 'ДДС Декларация', icon: '📊' },
            { key: 'purchases', label: 'Покупки с ДДС', icon: '📥' },
            { key: 'sales', label: 'Продажби с ДДС', icon: '📤' },
            { key: 'analysis', label: 'ДДС Анализ', icon: '🔍' }
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`py-2 px-1 border-b-2 font-medium text-sm flex items-center space-x-2 ${
                activeTab === tab.key
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Period Selection */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-4">
        <div className="flex items-center space-x-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Период
            </label>
            <input
              type="month"
              value={`${selectedPeriod.substring(0, 4)}-${selectedPeriod.substring(4, 6)}`}
              onChange={(e) => {
                const [year, month] = e.target.value.split('-');
                setSelectedPeriod(`${year}${month}`);
                setVATDeclaration(prev => ({
                  ...prev,
                  periodYear: year,
                  periodMonth: month
                }));
              }}
              className="px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>
          
          <div className="pt-6 flex gap-2 items-center">
            <button
              onClick={async () => {
                setLoading(true);
                setError(null);
                try {
                  const [year, month] = selectedPeriod.match(/(.{4})(.{2})/).slice(1);
                  const fromDate = `${year}-${month.padStart(2, '0')}-01`;
                  // Calculate last day of month properly
                  const lastDayOfMonth = new Date(parseInt(year), parseInt(month), 0).getDate();
                  const toDate = `${year}-${month.padStart(2, '0')}-${lastDayOfMonth.toString().padStart(2, '0')}`;
                  
                  const data = await graphqlRequest(`
                    query GetVATSummary($companyId: Int!, $fromDate: String!, $toDate: String!) {
                      vatDataSummary: journalEntries(
                        filter: {
                          companyId: $companyId,
                          fromDate: $fromDate,
                          toDate: $toDate,
                          isPosted: true
                        }
                      ) {
                        id
                        entryNumber
                        documentDate
                        description
                        totalAmount
                        totalVatAmount
                        vatDocumentType
                        isPosted
                      }
                    }
                  `, {
                    companyId: companyId,
                    fromDate: fromDate,
                    toDate: toDate
                  });
                  
                  // Process summary statistics
                  const entries = data.vatDataSummary || [];
                  const salesEntries = entries.filter(e => e.vatDocumentType === '01');
                  const purchaseEntries = entries.filter(e => e.vatDocumentType === '03');
                  const vatEntries = entries.filter(e => e.vatDocumentType);
                  
                  // Calculate detailed statistics
                  const summary = {
                    period: `${month}/${year}`,
                    totalEntries: entries.length,
                    salesEntries: salesEntries.length,
                    purchaseEntries: purchaseEntries.length,
                    vatEntries: vatEntries.length,
                    postedEntries: entries.filter(e => e.isPosted).length,
                    // Detailed data
                    entries: entries,
                    salesData: salesEntries,
                    purchaseData: purchaseEntries,
                    // Financial summaries - ensure all values are numbers
                    totalSalesBase: salesEntries.reduce((sum, e) => sum + (parseFloat(e.totalAmount || 0) - parseFloat(e.totalVatAmount || 0)), 0),
                    totalSalesVat: salesEntries.reduce((sum, e) => sum + parseFloat(e.totalVatAmount || 0), 0),
                    totalPurchasesBase: purchaseEntries.reduce((sum, e) => sum + (parseFloat(e.totalAmount || 0) - parseFloat(e.totalVatAmount || 0)), 0),
                    totalPurchasesVat: purchaseEntries.reduce((sum, e) => sum + parseFloat(e.totalVatAmount || 0), 0)
                  };
                  
                  setVatDataSummary(summary);
                  
                  // Update VAT declaration with summary
                  setVATDeclaration(prev => ({
                    ...prev,
                    // Calculate totals
                    baseAmount20: summary.totalSalesBase,
                    vatAmount20: summary.totalSalesVat,
                    inputVatAmount: summary.totalPurchasesVat
                  }));
                    
                } catch (err) {
                  setError('Грешка при зареждане на ДДС данни: ' + err.message);
                } finally {
                  setLoading(false);
                }
              }}
              className="px-4 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700"
            >
              Зареди ДДС данни
            </button>
            <button
              onClick={exportNapZip}
              className="px-4 py-2 bg-green-600 text-white rounded-md text-sm font-medium hover:bg-green-700 flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
              disabled={exportingNap}
            >
              {exportingNap ? (
                <>
                  <svg className="w-4 h-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 2a10 10 0 100 20 10 10 0 000-20z" />
                  </svg>
                  Генерира...
                </>
              ) : (
                <>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
                  </svg>
                  Експорт за НАП
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Tab Content */}
      {activeTab === 'declaration' && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">ДДС Декларация</h3>
            <p className="text-sm text-gray-500">Период: {selectedPeriod.substring(4, 6)}/{selectedPeriod.substring(0, 4)}</p>
          </div>
          
          <div className="p-6 space-y-6">
            {/* Company Info */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Статус
                </label>
                <div className="px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50">
                  {vatDeclaration.status === 'DRAFT' ? 'Чернова' : 
                   vatDeclaration.status === 'SUBMITTED' ? 'Подадена' : 'Одобрена'}
                </div>
              </div>
              {currentVATReturn && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Краен срок
                  </label>
                  <div className="px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50">
                    {currentVATReturn.dueDate}
                  </div>
                </div>
              )}
            </div>

            {/* Sales Section */}
            <div className="border border-gray-200 rounded-lg p-4">
              <h4 className="text-md font-semibold text-gray-800 mb-4">Продажби</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Основа 20% ДДС
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    value={vatDeclaration.baseAmount20}
                    onChange={(e) => {
                      const base = parseFloat(e.target.value) || 0;
                      const vat = base * 0.2;
                      setVATDeclaration(prev => ({ 
                        ...prev, 
                        baseAmount20: base,
                        vatAmount20: vat
                      }));
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                    disabled={vatDeclaration.status !== 'DRAFT'}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    ДДС 20%
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    value={vatDeclaration.vatAmount20}
                    readOnly
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Основа 9% ДДС
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    value={vatDeclaration.baseAmount9}
                    onChange={(e) => {
                      const base = parseFloat(e.target.value) || 0;
                      const vat = base * 0.09;
                      setVATDeclaration(prev => ({ 
                        ...prev, 
                        baseAmount9: base,
                        vatAmount9: vat
                      }));
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                    disabled={vatDeclaration.status !== 'DRAFT'}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    ДДС 9%
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    value={vatDeclaration.vatAmount9}
                    readOnly
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Продажби 0% ДДС
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    value={vatDeclaration.baseAmount0}
                    onChange={(e) => setVATDeclaration(prev => ({ ...prev, baseAmount0: parseFloat(e.target.value) || 0 }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                    disabled={vatDeclaration.status !== 'DRAFT'}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Освободени продажби
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    value={vatDeclaration.exemptAmount}
                    onChange={(e) => setVATDeclaration(prev => ({ ...prev, exemptAmount: parseFloat(e.target.value) || 0 }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                    disabled={vatDeclaration.status !== 'DRAFT'}
                  />
                </div>
              </div>
            </div>

            {/* Input VAT Section */}
            <div className="border border-gray-200 rounded-lg p-4">
              <h4 className="text-md font-semibold text-gray-800 mb-4">Зачетен ДДС</h4>
              <div className="grid grid-cols-1 md:grid-cols-1 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Зачетен входящ ДДС
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    value={vatDeclaration.inputVatAmount}
                    onChange={(e) => setVATDeclaration(prev => ({ ...prev, inputVatAmount: parseFloat(e.target.value) || 0 }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                    disabled={vatDeclaration.status !== 'DRAFT'}
                  />
                </div>
              </div>
            </div>

            {/* Summary */}
            <div className="border border-gray-200 rounded-lg p-4 bg-gray-50">
              <h4 className="text-md font-semibold text-gray-800 mb-4">Резултат</h4>
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div className="text-center">
                  <div className="text-sm text-gray-600">ДДС за плащане</div>
                  <div className="text-xl font-bold text-red-600">
                    {Math.max(0, (vatDeclaration.vatAmount20 + vatDeclaration.vatAmount9) - vatDeclaration.inputVatAmount).toFixed(2)} лв.
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-sm text-gray-600">ДДС за възстановяване</div>
                  <div className="text-xl font-bold text-green-600">
                    {Math.max(0, vatDeclaration.inputVatAmount - (vatDeclaration.vatAmount20 + vatDeclaration.vatAmount9)).toFixed(2)} лв.
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-sm text-gray-600">Общо продажби</div>
                  <div className="text-lg font-semibold text-gray-800">
                    {(vatDeclaration.baseAmount20 + vatDeclaration.baseAmount9 + vatDeclaration.baseAmount0 + vatDeclaration.exemptAmount).toFixed(2)} лв.
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-sm text-gray-600">Дължим ДДС</div>
                  <div className="text-lg font-semibold text-gray-800">
                    {(vatDeclaration.vatAmount20 + vatDeclaration.vatAmount9).toFixed(2)} лв.
                  </div>
                </div>
              </div>
            </div>

            {/* Actions */}
            <div className="flex items-center space-x-4">
              <button
                onClick={exportVATDeclaration}
                className="px-4 py-2 bg-green-600 text-white rounded-md text-sm font-medium hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
                disabled={exportingNap}
              >
                {exportingNap ? 'Генерира...' : '📥 Експорт DEKLAR.TXT'}
              </button>
              {vatDeclaration.status === 'DRAFT' && (
                <button
                  onClick={saveVATReturn}
                  className="px-4 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700"
                  disabled={loading}
                >
                  💾 Запази декларация
                </button>
              )}
              {currentVATReturn && vatDeclaration.status === 'DRAFT' && (
                <button
                  onClick={submitVATReturn}
                  className="px-4 py-2 bg-orange-600 text-white rounded-md text-sm font-medium hover:bg-orange-700"
                  disabled={loading}
                >
                  📤 Подай декларация
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {activeTab === 'purchases' && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">Покупки с ДДС</h3>
            <p className="text-sm text-gray-500">Регистър на покупки за ДДС целите</p>
          </div>
          
          <div className="p-6">
            <div className="text-center text-gray-500 py-8">
              <div className="text-4xl mb-4">📥</div>
              <p>Регистърът на покупки ще се генерира от счетоводните записи</p>
              <p className="text-sm mt-2">Формат: POKUPKI.TXT според НАП изискванията</p>
              <div className="mt-6 flex justify-center">
                <button
                  onClick={exportVATPurchases}
                  className="px-4 py-2 bg-green-600 text-white rounded-md text-sm font-medium hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
                  disabled={exportingNap}
                >
                  {exportingNap ? 'Генерира...' : '📥 Експорт POKUPKI.TXT'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === 'sales' && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">Продажби с ДДС</h3>
            <p className="text-sm text-gray-500">Регистър на продажби за ДДС целите</p>
          </div>
          
          <div className="p-6">
            <div className="text-center text-gray-500 py-8">
              <div className="text-4xl mb-4">📤</div>
              <p>Регистърът на продажби ще се генерира от счетоводните записи</p>
              <p className="text-sm mt-2">Формат: PRODAGBI.TXT според НАП изискванията</p>
              <div className="mt-6 flex justify-center">
                <button
                  onClick={exportVATSales}
                  className="px-4 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                  disabled={exportingNap}
                >
                  {exportingNap ? 'Генерира...' : '📤 Експорт PRODAGBI.TXT'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === 'analysis' && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">ДДС Анализ</h3>
            <p className="text-sm text-gray-500">Анализ на ДДС операции и тенденции</p>
          </div>
          
          <div className="p-6">
            <div className="text-center text-gray-500 py-8">
              <div className="text-4xl mb-4">🔍</div>
              <p>ДДС анализът ще показва детайли за ДДС операциите</p>
              <p className="text-sm mt-2">Графики, отчети и сравнения по периоди</p>
            </div>
          </div>
        </div>
      )}

      {/* VAT Data Summary Report */}
      {vatDataSummary && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">Справка за дневници с ДДС</h3>
            <p className="text-sm text-gray-500">Период: {vatDataSummary.period}</p>
          </div>
          
          <div className="p-6">
            {/* Summary Statistics */}
            <div className="grid grid-cols-2 md:grid-cols-6 gap-4 mb-6">
              <div className="bg-blue-50 p-4 rounded-lg">
                <div className="text-2xl font-bold text-blue-600">{vatDataSummary.totalEntries}</div>
                <div className="text-sm text-blue-800">Общо записи</div>
              </div>
              <div className="bg-green-50 p-4 rounded-lg">
                <div className="text-2xl font-bold text-green-600">{vatDataSummary.salesEntries}</div>
                <div className="text-sm text-green-800">Продажби (01)</div>
              </div>
              <div className="bg-purple-50 p-4 rounded-lg">
                <div className="text-2xl font-bold text-purple-600">{vatDataSummary.purchaseEntries}</div>
                <div className="text-sm text-purple-800">Покупки (03)</div>
              </div>
              <div className="bg-orange-50 p-4 rounded-lg">
                <div className="text-2xl font-bold text-orange-600">{vatDataSummary.vatEntries}</div>
                <div className="text-sm text-orange-800">ДДС записи</div>
              </div>
              <div className="bg-teal-50 p-4 rounded-lg">
                <div className="text-2xl font-bold text-teal-600">{vatDataSummary.postedEntries}</div>
                <div className="text-sm text-teal-800">Осчетоводени</div>
              </div>
              <div className="bg-red-50 p-4 rounded-lg">
                <div className="text-2xl font-bold text-red-600">{vatDataSummary.totalEntries - vatDataSummary.postedEntries}</div>
                <div className="text-sm text-red-800">Неосчетоводени</div>
              </div>
            </div>

            {/* Financial Summary */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
              <div className="border rounded-lg p-4">
                <h4 className="text-md font-semibold text-gray-800 mb-3 flex items-center">
                  <span className="text-green-500 mr-2">📤</span>
                  Обобщение на продажби
                </h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-gray-600">Основа за ДДС:</span>
                    <span className="font-medium">{(vatDataSummary.totalSalesBase || 0).toFixed(2)} лв.</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">ДДС:</span>
                    <span className="font-medium">{(vatDataSummary.totalSalesVat || 0).toFixed(2)} лв.</span>
                  </div>
                  <div className="flex justify-between border-t pt-2">
                    <span className="text-gray-800 font-semibold">Общо продажби:</span>
                    <span className="font-bold">{((vatDataSummary.totalSalesBase || 0) + (vatDataSummary.totalSalesVat || 0)).toFixed(2)} лв.</span>
                  </div>
                </div>
              </div>
              
              <div className="border rounded-lg p-4">
                <h4 className="text-md font-semibold text-gray-800 mb-3 flex items-center">
                  <span className="text-purple-500 mr-2">📥</span>
                  Обобщение на покупки
                </h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-gray-600">Основа за ДДС:</span>
                    <span className="font-medium">{(vatDataSummary.totalPurchasesBase || 0).toFixed(2)} лв.</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">ДДС:</span>
                    <span className="font-medium">{(vatDataSummary.totalPurchasesVat || 0).toFixed(2)} лв.</span>
                  </div>
                  <div className="flex justify-between border-t pt-2">
                    <span className="text-gray-800 font-semibold">Общо покупки:</span>
                    <span className="font-bold">{((vatDataSummary.totalPurchasesBase || 0) + (vatDataSummary.totalPurchasesVat || 0)).toFixed(2)} лв.</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Detailed Tables */}
            <div className="space-y-6">
              {/* Sales Table */}
              {vatDataSummary.salesData.length > 0 && (
                <div>
                  <h4 className="text-md font-semibold text-gray-800 mb-3">Детайли продажби ({vatDataSummary.salesData.length})</h4>
                  <div className="overflow-x-auto">
                    <table className="min-w-full divide-y divide-gray-200 border rounded-lg">
                      <thead className="bg-gray-50">
                        <tr>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Документ</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дата</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Описание</th>
                          <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Основа</th>
                          <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">ДДС</th>
                          <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Общо</th>
                        </tr>
                      </thead>
                      <tbody className="bg-white divide-y divide-gray-200">
                        {vatDataSummary.salesData.slice(0, 10).map((entry) => (
                          <tr key={entry.id}>
                            <td className="px-4 py-3 text-sm text-gray-900">{entry.entryNumber}</td>
                            <td className="px-4 py-3 text-sm text-gray-900">{entry.documentDate}</td>
                            <td className="px-4 py-3 text-sm text-gray-600 max-w-xs truncate">{entry.description}</td>
                            <td className="px-4 py-3 text-sm text-gray-900 text-right">{(parseFloat(entry.totalAmount || 0) - parseFloat(entry.totalVatAmount || 0)).toFixed(2)}</td>
                            <td className="px-4 py-3 text-sm text-gray-900 text-right">{parseFloat(entry.totalVatAmount || 0).toFixed(2)}</td>
                            <td className="px-4 py-3 text-sm font-medium text-gray-900 text-right">{parseFloat(entry.totalAmount || 0).toFixed(2)}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    {vatDataSummary.salesData.length > 10 && (
                      <div className="text-center text-sm text-gray-500 py-2">
                        ... и още {vatDataSummary.salesData.length - 10} записа
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* Purchases Table */}
              {vatDataSummary.purchaseData.length > 0 && (
                <div>
                  <h4 className="text-md font-semibold text-gray-800 mb-3">Детайли покупки ({vatDataSummary.purchaseData.length})</h4>
                  <div className="overflow-x-auto">
                    <table className="min-w-full divide-y divide-gray-200 border rounded-lg">
                      <thead className="bg-gray-50">
                        <tr>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Документ</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Дата</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Описание</th>
                          <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Основа</th>
                          <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">ДДС</th>
                          <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Общо</th>
                        </tr>
                      </thead>
                      <tbody className="bg-white divide-y divide-gray-200">
                        {vatDataSummary.purchaseData.slice(0, 10).map((entry) => (
                          <tr key={entry.id}>
                            <td className="px-4 py-3 text-sm text-gray-900">{entry.entryNumber}</td>
                            <td className="px-4 py-3 text-sm text-gray-900">{entry.documentDate}</td>
                            <td className="px-4 py-3 text-sm text-gray-600 max-w-xs truncate">{entry.description}</td>
                            <td className="px-4 py-3 text-sm text-gray-900 text-right">{(parseFloat(entry.totalAmount || 0) - parseFloat(entry.totalVatAmount || 0)).toFixed(2)}</td>
                            <td className="px-4 py-3 text-sm text-gray-900 text-right">{parseFloat(entry.totalVatAmount || 0).toFixed(2)}</td>
                            <td className="px-4 py-3 text-sm font-medium text-gray-900 text-right">{parseFloat(entry.totalAmount || 0).toFixed(2)}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    {vatDataSummary.purchaseData.length > 10 && (
                      <div className="text-center text-sm text-gray-500 py-2">
                        ... и още {vatDataSummary.purchaseData.length - 10} записа
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
