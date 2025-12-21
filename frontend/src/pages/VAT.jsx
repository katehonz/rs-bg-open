import { useState, useEffect, useRef } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

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

  // VAT Journals state
  const [purchaseJournal, setPurchaseJournal] = useState(null);
  const [salesJournal, setSalesJournal] = useState(null);
  const [loadingJournal, setLoadingJournal] = useState(false);

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

      // Check if data and vatReturns exist
      if (!data || !data.vatReturns) {
        console.warn('No VAT returns data received');
        return;
      }

      // Load current period VAT return if exists
      const currentReturn = data.vatReturns.find(vr =>
        vr.periodYear === vatDeclaration.periodYear &&
        vr.periodMonth === vatDeclaration.periodMonth
      );

      if (currentReturn) {
        setCurrentVATReturn(currentReturn);
        setVATDeclaration(prev => ({ ...prev, ...currentReturn }));
      }
    } catch (err) {
      console.error('Error loading VAT returns:', err);
      setError('Грешка при зареждане на ДДС декларации: ' + (err.message || 'Неизвестна грешка'));
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

  // Load VAT Purchase Journal
  const loadPurchaseJournal = async () => {
    setLoadingJournal(true);
    setError(null);

    try {
      const year = parseInt(selectedPeriod.substring(0, 4), 10);
      const month = parseInt(selectedPeriod.substring(4, 6), 10);

      const data = await graphqlRequest(`
        query GetVATPurchaseJournal($input: VatJournalInput!) {
          vatPurchaseJournal(input: $input) {
            companyName
            companyVat
            periodYear
            periodMonth
            entries {
              lineNumber
              documentType
              documentNumber
              documentDate
              counterpartVat
              counterpartName
              operationDescription
              col09Base
              col10Base
              col10Vat
              col12Base
              col12Vat
              col14Correction
              col15Triangular
            }
            totals {
              lineNumber
              counterpartName
              col09Base
              col10Base
              col10Vat
              col12Base
              col12Vat
              col14Correction
              col15Triangular
            }
          }
        }
      `, {
        input: {
          companyId: companyId,
          year: year,
          month: month
        }
      });

      setPurchaseJournal(data.vatPurchaseJournal);
    } catch (err) {
      setError('Грешка при зареждане на дневник покупки: ' + err.message);
    } finally {
      setLoadingJournal(false);
    }
  };

  // Load VAT Sales Journal
  const loadSalesJournal = async () => {
    setLoadingJournal(true);
    setError(null);

    try {
      const year = parseInt(selectedPeriod.substring(0, 4), 10);
      const month = parseInt(selectedPeriod.substring(4, 6), 10);

      const data = await graphqlRequest(`
        query GetVATSalesJournal($input: VatJournalInput!) {
          vatSalesJournal(input: $input) {
            companyName
            companyVat
            periodYear
            periodMonth
            entries {
              lineNumber
              documentType
              documentNumber
              documentDate
              counterpartVat
              counterpartName
              operationDescription
              col11Base20
              col11Vat20
              col12OtherVat
              col13EuAcquisition
              col14Base9
              col14Vat9
              col15Export
              col16Exempt
            }
            totals {
              lineNumber
              counterpartName
              col11Base20
              col11Vat20
              col12OtherVat
              col13EuAcquisition
              col14Base9
              col14Vat9
              col15Export
              col16Exempt
            }
          }
        }
      `, {
        input: {
          companyId: companyId,
          year: year,
          month: month
        }
      });

      setSalesJournal(data.vatSalesJournal);
    } catch (err) {
      setError('Грешка при зареждане на дневник продажби: ' + err.message);
    } finally {
      setLoadingJournal(false);
    }
  };

  // Export Purchase Journal to Excel
  const exportPurchaseJournalToExcel = async () => {
    if (!purchaseJournal) {
      setError('Моля, първо заредете дневника на покупките');
      return;
    }

    try {
      const XLSX = await import('xlsx');

      // Prepare data for Excel
      const excelData = [];

      // Header rows
      excelData.push([
        'Компания:', purchaseJournal.companyName,
        'ДДС №:', purchaseJournal.companyVat,
        'Период:', `${purchaseJournal.periodMonth.toString().padStart(2, '0')}/${purchaseJournal.periodYear}`
      ]);
      excelData.push([]); // Empty row

      // Column headers
      excelData.push([
        '№', 'Дата', 'Док №', 'Контрагент', 'ДДС №',
        'пок09 Основа', 'пок09 ДДС',
        'пок10 Основа', 'пок10 ДДС',
        'пок12 Основа', 'пок12 ДДС',
        'пок14 Корекция', 'пок15 Тристранна'
      ]);

      // Data rows
      purchaseJournal.entries.forEach(entry => {
        excelData.push([
          entry.lineNumber,
          entry.documentDate,
          entry.documentNumber,
          entry.counterpartName,
          entry.counterpartVat || '',
          parseFloat(entry.col09Base),
          0, // пок09 няма ДДС
          parseFloat(entry.col10Base),
          parseFloat(entry.col10Vat),
          parseFloat(entry.col12Base),
          parseFloat(entry.col12Vat),
          parseFloat(entry.col14Correction),
          parseFloat(entry.col15Triangular)
        ]);
      });

      // Totals row
      excelData.push([
        '', '', '', 'ОБЩО', '',
        parseFloat(purchaseJournal.totals.col09Base),
        0,
        parseFloat(purchaseJournal.totals.col10Base),
        parseFloat(purchaseJournal.totals.col10Vat),
        parseFloat(purchaseJournal.totals.col12Base),
        parseFloat(purchaseJournal.totals.col12Vat),
        parseFloat(purchaseJournal.totals.col14Correction),
        parseFloat(purchaseJournal.totals.col15Triangular)
      ]);

      // Create worksheet and workbook
      const ws = XLSX.utils.aoa_to_sheet(excelData);

      // Set column widths
      ws['!cols'] = [
        { wch: 5 },  // №
        { wch: 12 }, // Дата
        { wch: 15 }, // Док №
        { wch: 30 }, // Контрагент
        { wch: 15 }, // ДДС №
        { wch: 12 }, // пок09 Основа
        { wch: 10 }, // пок09 ДДС
        { wch: 12 }, // пок10 Основа
        { wch: 12 }, // пок10 ДДС
        { wch: 12 }, // пок12 Основа
        { wch: 12 }, // пок12 ДДС
        { wch: 12 }, // пок14
        { wch: 12 }  // пок15
      ];

      const wb = XLSX.utils.book_new();
      XLSX.utils.book_append_sheet(wb, ws, 'Дневник покупки');

      // Download file
      XLSX.writeFile(wb, `Дневник_покупки_${selectedPeriod}.xlsx`);
    } catch (err) {
      setError('Грешка при експорт: ' + err.message);
    }
  };

  // Export Sales Journal to Excel
  const exportSalesJournalToExcel = async () => {
    if (!salesJournal) {
      setError('Моля, първо заредете дневника на продажбите');
      return;
    }

    try {
      const XLSX = await import('xlsx');

      // Prepare data for Excel
      const excelData = [];

      // Header rows
      excelData.push([
        'Компания:', salesJournal.companyName,
        'ДДС №:', salesJournal.companyVat,
        'Период:', `${salesJournal.periodMonth.toString().padStart(2, '0')}/${salesJournal.periodYear}`
      ]);
      excelData.push([]); // Empty row

      // Column headers
      excelData.push([
        '№', 'Дата', 'Док №', 'Контрагент', 'ДДС №',
        'про11 Основа 20%', 'про11 ДДС 20%',
        'про12 Др ДДС',
        'про13 ВОП',
        'про14 Основа 9%', 'про14 ДДС 9%',
        'про15 Експорт', 'про16 Освободени'
      ]);

      // Data rows
      salesJournal.entries.forEach(entry => {
        excelData.push([
          entry.lineNumber,
          entry.documentDate,
          entry.documentNumber,
          entry.counterpartName,
          entry.counterpartVat || '',
          parseFloat(entry.col11Base20),
          parseFloat(entry.col11Vat20),
          parseFloat(entry.col12OtherVat),
          parseFloat(entry.col13EuAcquisition),
          parseFloat(entry.col14Base9),
          parseFloat(entry.col14Vat9),
          parseFloat(entry.col15Export),
          parseFloat(entry.col16Exempt)
        ]);
      });

      // Totals row
      excelData.push([
        '', '', '', 'ОБЩО', '',
        parseFloat(salesJournal.totals.col11Base20),
        parseFloat(salesJournal.totals.col11Vat20),
        parseFloat(salesJournal.totals.col12OtherVat),
        parseFloat(salesJournal.totals.col13EuAcquisition),
        parseFloat(salesJournal.totals.col14Base9),
        parseFloat(salesJournal.totals.col14Vat9),
        parseFloat(salesJournal.totals.col15Export),
        parseFloat(salesJournal.totals.col16Exempt)
      ]);

      // Create worksheet and workbook
      const ws = XLSX.utils.aoa_to_sheet(excelData);

      // Set column widths
      ws['!cols'] = [
        { wch: 5 },  // №
        { wch: 12 }, // Дата
        { wch: 15 }, // Док №
        { wch: 30 }, // Контрагент
        { wch: 15 }, // ДДС №
        { wch: 14 }, // про11 Основа
        { wch: 14 }, // про11 ДДС
        { wch: 12 }, // про12
        { wch: 12 }, // про13
        { wch: 14 }, // про14 Основа
        { wch: 14 }, // про14 ДДС
        { wch: 12 }, // про15
        { wch: 14 }  // про16
      ];

      const wb = XLSX.utils.book_new();
      XLSX.utils.book_append_sheet(wb, ws, 'Дневник продажби');

      // Download file
      XLSX.writeFile(wb, `Дневник_продажби_${selectedPeriod}.xlsx`);
    } catch (err) {
      setError('Грешка при експорт: ' + err.message);
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

                  // Check if data exists
                  if (!data || !data.vatDataSummary) {
                    throw new Error('Няма налични данни за избрания период');
                  }

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
          <div className="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
            <div>
              <h3 className="text-lg font-semibold text-gray-900">Дневник на покупките</h3>
              <p className="text-sm text-gray-500">Според изискванията на НАП - Колони пок09-пок15</p>
            </div>
            <div className="flex gap-2">
              <button
                onClick={loadPurchaseJournal}
                className="px-4 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
                disabled={loadingJournal}
              >
                {loadingJournal ? 'Зарежда...' : 'Зареди дневник'}
              </button>
              <button
                onClick={exportPurchaseJournalToExcel}
                className="px-4 py-2 bg-emerald-600 text-white rounded-md text-sm font-medium hover:bg-emerald-700 disabled:opacity-50"
                disabled={!purchaseJournal}
              >
                📊 Excel
              </button>
              <button
                onClick={exportVATPurchases}
                className="px-4 py-2 bg-green-600 text-white rounded-md text-sm font-medium hover:bg-green-700 disabled:opacity-50"
                disabled={exportingNap}
              >
                {exportingNap ? 'Експортира...' : '📥 POKUPKI.TXT'}
              </button>
            </div>
          </div>

          <div className="p-6">
            {!purchaseJournal && !loadingJournal && (
              <div className="text-center text-gray-500 py-12">
                <div className="text-4xl mb-4">📥</div>
                <p className="text-lg">Натиснете "Зареди дневник" за да видите записите</p>
                <p className="text-sm mt-2">Дневникът показва покупки с ДДС за избрания период</p>
              </div>
            )}

            {loadingJournal && (
              <div className="text-center py-12">
                <div className="animate-spin h-12 w-12 border-4 border-blue-500 border-t-transparent rounded-full mx-auto mb-4"></div>
                <p className="text-gray-600">Зарежда се дневник на покупките...</p>
              </div>
            )}

            {purchaseJournal && !loadingJournal && (
              <div>
                <div className="mb-4 p-4 bg-gray-50 rounded-lg">
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div><span className="text-gray-600">Компания:</span> <span className="font-medium">{purchaseJournal.companyName}</span></div>
                    <div><span className="text-gray-600">ДДС номер:</span> <span className="font-medium">{purchaseJournal.companyVat}</span></div>
                    <div><span className="text-gray-600">Период:</span> <span className="font-medium">{purchaseJournal.periodMonth.toString().padStart(2, '0')}/{purchaseJournal.periodYear}</span></div>
                    <div><span className="text-gray-600">Записи:</span> <span className="font-medium">{purchaseJournal.entries.length}</span></div>
                  </div>
                </div>

                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200 border">
                    <thead className="bg-gray-50">
                      <tr>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">№</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">Дата</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">Док №</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">Контрагент</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">ДДС №</th>
                        <th colSpan="2" className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase border-r bg-gray-100">пок09 Без ДК</th>
                        <th colSpan="2" className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase border-r bg-blue-50">пок10 Пълен ДК</th>
                        <th colSpan="2" className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase border-r bg-green-50">пок12 Частичен ДК</th>
                        <th rowSpan="2" className="px-2 py-3 text-right text-xs font-medium text-gray-500 uppercase border-r bg-yellow-50">пок14 Корекция</th>
                        <th rowSpan="2" className="px-2 py-3 text-right text-xs font-medium text-gray-500 uppercase bg-purple-50">пок15 Тристр</th>
                      </tr>
                      <tr>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-gray-100">Основа</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-gray-100">ДДС</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-blue-50">Основа</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-blue-50">ДДС</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-green-50">Основа</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-green-50">ДДС</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {purchaseJournal.entries.map((entry) => (
                        <tr key={entry.lineNumber} className="hover:bg-gray-50">
                          <td className="px-2 py-2 text-sm text-gray-900 border-r">{entry.lineNumber}</td>
                          <td className="px-2 py-2 text-sm text-gray-900 border-r whitespace-nowrap">{entry.documentDate}</td>
                          <td className="px-2 py-2 text-sm text-gray-900 border-r">{entry.documentNumber}</td>
                          <td className="px-2 py-2 text-sm text-gray-900 border-r max-w-xs truncate" title={entry.counterpartName}>{entry.counterpartName}</td>
                          <td className="px-2 py-2 text-sm text-gray-600 border-r">{entry.counterpartVat || '-'}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-gray-50">{parseFloat(entry.col09Base).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-600 border-r bg-gray-50">-</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-blue-50 font-medium">{parseFloat(entry.col10Base).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-green-600 border-r bg-blue-50 font-medium">{parseFloat(entry.col10Vat).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-green-50">{parseFloat(entry.col12Base).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-green-600 border-r bg-green-50">{parseFloat(entry.col12Vat).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-yellow-50">{parseFloat(entry.col14Correction).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 bg-purple-50">{parseFloat(entry.col15Triangular).toFixed(2)}</td>
                        </tr>
                      ))}

                      {/* Totals Row */}
                      {purchaseJournal.totals && (
                        <tr className="bg-gray-100 font-bold border-t-2 border-gray-300">
                          <td colSpan="5" className="px-2 py-3 text-sm text-gray-900 border-r">{purchaseJournal.totals.counterpartName}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900 border-r">{parseFloat(purchaseJournal.totals.col09Base).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-600 border-r">-</td>
                          <td className="px-2 py-3 text-sm text-right text-blue-900 border-r">{parseFloat(purchaseJournal.totals.col10Base).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-green-700 border-r">{parseFloat(purchaseJournal.totals.col10Vat).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900 border-r">{parseFloat(purchaseJournal.totals.col12Base).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-green-700 border-r">{parseFloat(purchaseJournal.totals.col12Vat).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900 border-r">{parseFloat(purchaseJournal.totals.col14Correction).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900">{parseFloat(purchaseJournal.totals.col15Triangular).toFixed(2)}</td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'sales' && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
            <div>
              <h3 className="text-lg font-semibold text-gray-900">Дневник на продажбите</h3>
              <p className="text-sm text-gray-500">Според изискванията на НАП - Колони про11-про25</p>
            </div>
            <div className="flex gap-2">
              <button
                onClick={loadSalesJournal}
                className="px-4 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
                disabled={loadingJournal}
              >
                {loadingJournal ? 'Зарежда...' : 'Зареди дневник'}
              </button>
              <button
                onClick={exportSalesJournalToExcel}
                className="px-4 py-2 bg-emerald-600 text-white rounded-md text-sm font-medium hover:bg-emerald-700 disabled:opacity-50"
                disabled={!salesJournal}
              >
                📊 Excel
              </button>
              <button
                onClick={exportVATSales}
                className="px-4 py-2 bg-green-600 text-white rounded-md text-sm font-medium hover:bg-green-700 disabled:opacity-50"
                disabled={exportingNap}
              >
                {exportingNap ? 'Експортира...' : '📤 PRODAGBI.TXT'}
              </button>
            </div>
          </div>

          <div className="p-6">
            {!salesJournal && !loadingJournal && (
              <div className="text-center text-gray-500 py-12">
                <div className="text-4xl mb-4">📤</div>
                <p className="text-lg">Натиснете "Зареди дневник" за да видите записите</p>
                <p className="text-sm mt-2">Дневникът показва продажби с ДДС за избрания период</p>
              </div>
            )}

            {loadingJournal && (
              <div className="text-center py-12">
                <div className="animate-spin h-12 w-12 border-4 border-blue-500 border-t-transparent rounded-full mx-auto mb-4"></div>
                <p className="text-gray-600">Зарежда се дневник на продажбите...</p>
              </div>
            )}

            {salesJournal && !loadingJournal && (
              <div>
                <div className="mb-4 p-4 bg-gray-50 rounded-lg">
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div><span className="text-gray-600">Компания:</span> <span className="font-medium">{salesJournal.companyName}</span></div>
                    <div><span className="text-gray-600">ДДС номер:</span> <span className="font-medium">{salesJournal.companyVat}</span></div>
                    <div><span className="text-gray-600">Период:</span> <span className="font-medium">{salesJournal.periodMonth.toString().padStart(2, '0')}/{salesJournal.periodYear}</span></div>
                    <div><span className="text-gray-600">Записи:</span> <span className="font-medium">{salesJournal.entries.length}</span></div>
                  </div>
                </div>

                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200 border">
                    <thead className="bg-gray-50">
                      <tr>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">№</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">Дата</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">Док №</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">Контрагент</th>
                        <th rowSpan="2" className="px-2 py-3 text-left text-xs font-medium text-gray-500 uppercase border-r">ДДС №</th>
                        <th colSpan="2" className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase border-r bg-blue-50">про11 (20%)</th>
                        <th rowSpan="2" className="px-2 py-3 text-right text-xs font-medium text-gray-500 uppercase border-r bg-gray-100">про12 Др ДДС</th>
                        <th rowSpan="2" className="px-2 py-3 text-right text-xs font-medium text-gray-500 uppercase border-r bg-purple-50">про13 ВОП</th>
                        <th colSpan="2" className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase border-r bg-green-50">про14 (9%)</th>
                        <th rowSpan="2" className="px-2 py-3 text-right text-xs font-medium text-gray-500 uppercase border-r bg-yellow-50">про15 Експорт</th>
                        <th rowSpan="2" className="px-2 py-3 text-right text-xs font-medium text-gray-500 uppercase bg-orange-50">про16 Освоб</th>
                      </tr>
                      <tr>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-blue-50">Основа</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-blue-50">ДДС</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-green-50">Основа</th>
                        <th className="px-2 py-2 text-right text-xs font-medium text-gray-500 uppercase border-r bg-green-50">ДДС</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {salesJournal.entries.map((entry) => (
                        <tr key={entry.lineNumber} className="hover:bg-gray-50">
                          <td className="px-2 py-2 text-sm text-gray-900 border-r">{entry.lineNumber}</td>
                          <td className="px-2 py-2 text-sm text-gray-900 border-r whitespace-nowrap">{entry.documentDate}</td>
                          <td className="px-2 py-2 text-sm text-gray-900 border-r">{entry.documentNumber}</td>
                          <td className="px-2 py-2 text-sm text-gray-900 border-r max-w-xs truncate" title={entry.counterpartName}>{entry.counterpartName}</td>
                          <td className="px-2 py-2 text-sm text-gray-600 border-r">{entry.counterpartVat || '-'}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-blue-50 font-medium">{parseFloat(entry.col11Base20).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-blue-600 border-r bg-blue-50 font-medium">{parseFloat(entry.col11Vat20).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-gray-50">{parseFloat(entry.col12OtherVat).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-purple-50">{parseFloat(entry.col13EuAcquisition).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-green-50">{parseFloat(entry.col14Base9).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-green-600 border-r bg-green-50">{parseFloat(entry.col14Vat9).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 border-r bg-yellow-50">{parseFloat(entry.col15Export).toFixed(2)}</td>
                          <td className="px-2 py-2 text-sm text-right text-gray-900 bg-orange-50">{parseFloat(entry.col16Exempt).toFixed(2)}</td>
                        </tr>
                      ))}

                      {/* Totals Row */}
                      {salesJournal.totals && (
                        <tr className="bg-gray-100 font-bold border-t-2 border-gray-300">
                          <td colSpan="5" className="px-2 py-3 text-sm text-gray-900 border-r">{salesJournal.totals.counterpartName}</td>
                          <td className="px-2 py-3 text-sm text-right text-blue-900 border-r">{parseFloat(salesJournal.totals.col11Base20).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-blue-700 border-r">{parseFloat(salesJournal.totals.col11Vat20).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900 border-r">{parseFloat(salesJournal.totals.col12OtherVat).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900 border-r">{parseFloat(salesJournal.totals.col13EuAcquisition).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900 border-r">{parseFloat(salesJournal.totals.col14Base9).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-green-700 border-r">{parseFloat(salesJournal.totals.col14Vat9).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900 border-r">{parseFloat(salesJournal.totals.col15Export).toFixed(2)}</td>
                          <td className="px-2 py-3 text-sm text-right text-gray-900">{parseFloat(salesJournal.totals.col16Exempt).toFixed(2)}</td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
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
