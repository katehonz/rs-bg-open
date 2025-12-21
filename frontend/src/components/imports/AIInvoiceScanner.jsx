import { useState, useCallback } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

const PROCESS_INVOICE_MUTATION = `
  mutation ProcessInvoice($input: ProcessInvoiceInput!) {
    processInvoice(input: $input) {
      companyId
      requiresManualReview
      document {
        documentType
        transactionType
        documentNumber
        documentDate
        dueDate
        currency
        netAmount
        vatAmount
        totalAmount
        counterpart {
          name
          eik
          vatNumber
          address
        }
        items {
          description
          quantity
          unit
          unitPrice
          totalPrice
          vatRate
        }
      }
      contragent {
        id
        vatNumber
        eik
        companyName
        companyNameBg
        address
        city
        postalCode
        country
        vatValid
        eikValid
      }
      validationSource
      existedInDatabase
      autoCreatedCounterpartId
    }
  }
`;

const CREATE_VAT_JOURNAL_MUTATION = `
  mutation CreateVatJournalFromAI($input: CreateVatJournalFromAIInput!) {
    createVatJournalFromAi(input: $input) {
      success
      journalEntryId
      entryNumber
      message
    }
  }
`;

export default function AIInvoiceScanner() {
  const [dragOver, setDragOver] = useState(false);
  const [uploadedFiles, setUploadedFiles] = useState([]);
  const [processing, setProcessing] = useState(false);
  const [results, setResults] = useState([]);

  // Session-persisted invoice type and VAT operation
  const [invoiceType, setInvoiceType] = useState(() => {
    return sessionStorage.getItem('aiScanner_invoiceType') || 'PURCHASE';
  });
  const [vatOperation, setVatOperation] = useState(() => {
    return sessionStorage.getItem('aiScanner_vatOperation') || 'пок10';
  });

  const acceptedFormats = ['.png', '.jpg', '.jpeg', '.pdf'];

  // VAT operations for purchases
  const purchaseOperations = [
    { value: 'пок09', label: 'пок09 - Без право на ДК или без данък' },
    { value: 'пок10', label: 'пок10 - Пълно право на ДК (стандартно)' },
    { value: 'пок12', label: 'пок12 - Частично право на ДК' },
    { value: 'пок15', label: 'пок15 - Тристранна операция' },
  ];

  // VAT operations for sales
  const salesOperations = [
    { value: 'про11', label: 'про11 - Облагаеми доставки 20%' },
    { value: 'про17', label: 'про17 - Облагаеми доставки 9%' },
    { value: 'про19', label: 'про19 - Облагаеми доставки 0%' },
    { value: 'про13', label: 'про13 - ВОП (вътреобщностно придобиване)' },
    { value: 'про15', label: 'про15 - Износ' },
    { value: 'про16', label: 'про16 - Освободени доставки' },
  ];

  // Update session storage when invoice type changes
  const handleInvoiceTypeChange = (type) => {
    setInvoiceType(type);
    sessionStorage.setItem('aiScanner_invoiceType', type);

    // Set default VAT operation based on invoice type
    const defaultOp = type === 'PURCHASE' ? 'пок10' : 'про11';
    setVatOperation(defaultOp);
    sessionStorage.setItem('aiScanner_vatOperation', defaultOp);
  };

  // Update session storage when VAT operation changes
  const handleVatOperationChange = (operation) => {
    setVatOperation(operation);
    sessionStorage.setItem('aiScanner_vatOperation', operation);
  };

  const handleDragOver = useCallback((e) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e) => {
    e.preventDefault();
    setDragOver(false);
  }, []);

  const convertFileToBase64 = (file) => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const base64 = reader.result.split(',')[1];
        resolve(base64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  };

  const handleDrop = useCallback((e) => {
    e.preventDefault();
    setDragOver(false);

    const files = Array.from(e.dataTransfer.files);
    const validFiles = files.filter(file =>
      acceptedFormats.some(format => file.name.toLowerCase().endsWith(format))
    );

    if (validFiles.length > 0) {
      setUploadedFiles(prev => [...prev, ...validFiles.map(file => ({
        name: file.name,
        size: file.size,
        type: file.type,
        status: 'pending',
        file: file
      }))]);
    }
  }, []);

  const handleFileSelect = useCallback((e) => {
    const files = Array.from(e.target.files);
    setUploadedFiles(prev => [...prev, ...files.map(file => ({
      name: file.name,
      size: file.size,
      type: file.type,
      status: 'pending',
      file: file
    }))]);
  }, []);

  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const removeFile = (index) => {
    setUploadedFiles(prev => prev.filter((_, i) => i !== index));
    setResults(prev => prev.filter((_, i) => i !== index));
  };

  const startAIScanning = async () => {
    if (uploadedFiles.length === 0) return;

    setProcessing(true);
    const currentCompanyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
    const newResults = [];

    for (let i = 0; i < uploadedFiles.length; i++) {
      const fileData = uploadedFiles[i];

      try {
        // Update status to processing
        setUploadedFiles(prev => prev.map((f, idx) =>
          idx === i ? { ...f, status: 'processing' } : f
        ));

        // Convert file to base64
        const base64Content = await convertFileToBase64(fileData.file);

        // Call GraphQL mutation
        const response = await graphqlRequest(PROCESS_INVOICE_MUTATION, {
          input: {
            companyId: currentCompanyId,
            fileName: fileData.name,
            contentType: fileData.type,
            fileBase64: base64Content
          }
        });

        const result = response.processInvoice;

        // Debug: Log the result to see what we received
        console.log('AI Scan Result:', {
          autoCreatedCounterpartId: result.autoCreatedCounterpartId,
          contragentId: result.contragent?.id,
          requiresManualReview: result.requiresManualReview,
          validationSource: result.validationSource,
        });

        // Update status to completed
        setUploadedFiles(prev => prev.map((f, idx) =>
          idx === i ? { ...f, status: 'completed', result } : f
        ));

        newResults.push(result);

      } catch (error) {
        console.error('AI Scanning error:', error);

        // Update status to error
        setUploadedFiles(prev => prev.map((f, idx) =>
          idx === i ? { ...f, status: 'error', error: error.message } : f
        ));

        newResults.push(null);
      }
    }

    setResults(newResults);
    setProcessing(false);
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'pending': return 'bg-yellow-100 text-yellow-800';
      case 'processing': return 'bg-blue-100 text-blue-800';
      case 'completed': return 'bg-green-100 text-green-800';
      case 'error': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 'pending': return 'Чака';
      case 'processing': return 'Сканира...';
      case 'completed': return 'Готово';
      case 'error': return 'Грешка';
      default: return 'Неизвестен';
    }
  };

  const createVatJournalEntry = async (fileIndex) => {
    const file = uploadedFiles[fileIndex];
    if (!file.result || !file.result.document) {
      alert('Няма данни за създаване на ДДС операция');
      return;
    }

    const result = file.result;
    const doc = result.document;

    // Validate required fields
    if (!doc.documentNumber || !doc.documentDate || !doc.netAmount || !doc.vatAmount || !doc.totalAmount) {
      alert('Липсват задължителни данни за документа');
      return;
    }

    // Use auto-created counterpart ID if available, otherwise use contragent.id
    const counterpartId = result.autoCreatedCounterpartId || result.contragent?.id;

    console.log('Creating VAT journal with:', {
      autoCreatedCounterpartId: result.autoCreatedCounterpartId,
      contragentId: result.contragent?.id,
      finalCounterpartId: counterpartId,
      companyId: result.companyId,
    });

    if (!counterpartId) {
      alert('Липсва валиден контрагент. Моля проверете дали има валиден ДДС номер.');
      return;
    }

    const currentCompanyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

    try {
      setUploadedFiles(prev => prev.map((f, idx) =>
        idx === fileIndex ? { ...f, creatingJournal: true } : f
      ));

      const input = {
        companyId: parseInt(result.companyId) || currentCompanyId,
        counterpartId: counterpartId,
        transactionType: invoiceType, // Use selected invoice type from state
        documentNumber: doc.documentNumber,
        documentDate: doc.documentDate,
        vatDate: doc.documentDate,
        accountingDate: doc.documentDate,
        description: `${doc.counterpart?.name || 'Неизвестен контрагент'} - ${doc.documentNumber}`,
        currency: doc.currency || 'BGN',
        netAmount: parseFloat(doc.netAmount),
        vatAmount: parseFloat(doc.vatAmount),
        totalAmount: parseFloat(doc.totalAmount),
        vatOperation: vatOperation, // Include selected VAT operation
      };

      const response = await graphqlRequest(CREATE_VAT_JOURNAL_MUTATION, { input });

      if (response.createVatJournalFromAi.success) {
        const invoiceTypeLabel = invoiceType === 'PURCHASE' ? 'Покупка' : 'Продажба';
        alert(`✓ ${response.createVatJournalFromAi.message}\nЗапис №: ${response.createVatJournalFromAi.entryNumber}\nТип: ${invoiceTypeLabel} (${vatOperation})`);

        setUploadedFiles(prev => prev.map((f, idx) =>
          idx === fileIndex ? { ...f, creatingJournal: false, journalCreated: true, invoiceType, vatOperation } : f
        ));
      } else {
        throw new Error('Грешка при създаване на ДДС операция');
      }
    } catch (error) {
      console.error('Error creating VAT journal:', error);
      alert('Грешка при създаване на ДДС операция: ' + error.message);

      setUploadedFiles(prev => prev.map((f, idx) =>
        idx === fileIndex ? { ...f, creatingJournal: false } : f
      ));
    }
  };

  return (
    <div className="space-y-6">
      {/* Header Info */}
      <div className="flex items-start space-x-4">
        <div className="text-4xl">🤖</div>
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-gray-900">
            AI Сканиране на фактури
          </h3>
          <p className="text-gray-600 mb-3">
            Качете снимка или PDF на фактура и AI ще извлече всички данни автоматично
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            <div className="flex items-center space-x-2">
              <span className="w-2 h-2 bg-green-400 rounded-full"></span>
              <span className="text-sm text-gray-700">Mistral AI Vision</span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="w-2 h-2 bg-green-400 rounded-full"></span>
              <span className="text-sm text-gray-700">Автоматично извличане на данни</span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="w-2 h-2 bg-green-400 rounded-full"></span>
              <span className="text-sm text-gray-700">VIES валидация на ДДС</span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="w-2 h-2 bg-green-400 rounded-full"></span>
              <span className="text-sm text-gray-700">PDF и изображения</span>
            </div>
          </div>
        </div>
      </div>

      {/* Invoice Type and VAT Operation Selection */}
      <div className="bg-blue-50 border-2 border-blue-200 rounded-lg p-4">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Invoice Type Selector */}
          <div>
            <label className="block text-sm font-semibold text-gray-900 mb-2">
              📋 Тип фактури за сканиране
            </label>
            <div className="flex space-x-3">
              <button
                onClick={() => handleInvoiceTypeChange('PURCHASE')}
                className={`flex-1 px-4 py-3 text-sm font-medium rounded-lg transition-all ${
                  invoiceType === 'PURCHASE'
                    ? 'bg-blue-600 text-white shadow-md'
                    : 'bg-white text-gray-700 hover:bg-gray-50 border border-gray-300'
                }`}
              >
                <div className="flex items-center justify-center space-x-2">
                  <span className="text-lg">📥</span>
                  <span>Покупки</span>
                </div>
              </button>
              <button
                onClick={() => handleInvoiceTypeChange('SALE')}
                className={`flex-1 px-4 py-3 text-sm font-medium rounded-lg transition-all ${
                  invoiceType === 'SALE'
                    ? 'bg-green-600 text-white shadow-md'
                    : 'bg-white text-gray-700 hover:bg-gray-50 border border-gray-300'
                }`}
              >
                <div className="flex items-center justify-center space-x-2">
                  <span className="text-lg">📤</span>
                  <span>Продажби</span>
                </div>
              </button>
            </div>
          </div>

          {/* VAT Operation Selector */}
          <div>
            <label className="block text-sm font-semibold text-gray-900 mb-2">
              💼 ДДС операция (дневник)
            </label>
            <select
              value={vatOperation}
              onChange={(e) => handleVatOperationChange(e.target.value)}
              className="w-full px-4 py-3 text-sm border border-gray-300 rounded-lg bg-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              {invoiceType === 'PURCHASE'
                ? purchaseOperations.map(op => (
                    <option key={op.value} value={op.value}>{op.label}</option>
                  ))
                : salesOperations.map(op => (
                    <option key={op.value} value={op.value}>{op.label}</option>
                  ))
              }
            </select>
          </div>
        </div>

        <div className="mt-3 flex items-center space-x-2 text-xs text-blue-800">
          <span>ℹ️</span>
          <span>Изборът се запазва за текущата сесия - всички сканирани фактури ще използват избрания тип и операция</span>
        </div>
      </div>

      {/* Upload Area */}
      <div
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          dragOver
            ? 'border-blue-500 bg-blue-50'
            : 'border-gray-300 hover:border-gray-400'
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="space-y-4">
          <div className="text-5xl">📄</div>
          <div>
            <p className="text-lg font-medium text-gray-900">
              Пуснете фактури тук или
            </p>
            <label className="cursor-pointer">
              <span className="text-blue-600 hover:text-blue-700 font-medium">
                изберете файлове
              </span>
              <input
                type="file"
                multiple
                accept={acceptedFormats.join(',')}
                onChange={handleFileSelect}
                className="hidden"
              />
            </label>
          </div>
          <p className="text-sm text-gray-500">
            Поддържани формати: PNG, JPG, PDF
          </p>
          <div className="text-xs text-gray-500">
            Максимум 10 файла, до 10MB всеки
          </div>
        </div>
      </div>

      {/* Uploaded Files */}
      {uploadedFiles.length > 0 && (
        <div className="space-y-4">
          <h4 className="text-sm font-medium text-gray-900">
            Качени файлове ({uploadedFiles.length})
          </h4>
          <div className="space-y-3">
            {uploadedFiles.map((file, index) => (
              <div key={index} className="bg-white border rounded-lg overflow-hidden">
                <div className="flex items-center justify-between p-4">
                  <div className="flex items-center space-x-3 flex-1">
                    <div className="text-2xl">
                      {file.name.toLowerCase().endsWith('.pdf') ? '📄' : '🖼️'}
                    </div>
                    <div className="flex-1">
                      <p className="text-sm font-medium text-gray-900">
                        {file.name}
                      </p>
                      <p className="text-xs text-gray-500">
                        {formatFileSize(file.size)}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-3">
                    <span className={`px-3 py-1 text-xs rounded-full font-medium ${getStatusColor(file.status)}`}>
                      {getStatusText(file.status)}
                      {file.result?.requiresManualReview && ' ⚠️'}
                    </span>
                    {file.status !== 'processing' && (
                      <button
                        onClick={() => removeFile(index)}
                        className="p-1 text-gray-400 hover:text-red-600 transition-colors"
                      >
                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    )}
                  </div>
                </div>

                {/* Result Details */}
                {file.status === 'completed' && file.result && (
                  <div className="border-t bg-gray-50 p-4 space-y-3">
                    {/* Invoice Type and VAT Operation Badge */}
                    <div className="flex items-center space-x-2 pb-2 border-b">
                      <span className={`px-3 py-1 text-xs font-semibold rounded-full ${
                        invoiceType === 'PURCHASE'
                          ? 'bg-blue-100 text-blue-800'
                          : 'bg-green-100 text-green-800'
                      }`}>
                        {invoiceType === 'PURCHASE' ? '📥 Покупка' : '📤 Продажба'}
                      </span>
                      <span className="px-3 py-1 text-xs font-medium bg-purple-100 text-purple-800 rounded-full">
                        {vatOperation}
                      </span>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <label className="text-xs font-medium text-gray-600">Тип документ</label>
                        <p className="text-sm text-gray-900">{file.result.document.documentType || '-'}</p>
                      </div>
                      <div>
                        <label className="text-xs font-medium text-gray-600">Номер</label>
                        <p className="text-sm text-gray-900">{file.result.document.documentNumber || '-'}</p>
                      </div>
                      <div>
                        <label className="text-xs font-medium text-gray-600">Дата</label>
                        <p className="text-sm text-gray-900">{file.result.document.documentDate || '-'}</p>
                      </div>
                      <div>
                        <label className="text-xs font-medium text-gray-600">Обща сума</label>
                        <p className="text-sm font-semibold text-gray-900">
                          {file.result.document.totalAmount || '0'} {file.result.document.currency || 'BGN'}
                        </p>
                      </div>
                    </div>

                    {file.result.document.counterpart && (
                      <div className="border-t pt-3 space-y-2">
                        <h5 className="text-xs font-semibold text-gray-700">Контрагент</h5>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                          <div>
                            <label className="text-xs font-medium text-gray-600">Име</label>
                            <p className="text-sm text-gray-900">{file.result.document.counterpart.name || '-'}</p>
                          </div>
                          <div>
                            <label className="text-xs font-medium text-gray-600">ЕИК</label>
                            <p className="text-sm text-gray-900 font-mono">{file.result.document.counterpart.eik || '-'}</p>
                          </div>
                          <div>
                            <label className="text-xs font-medium text-gray-600">ДДС номер</label>
                            <div className="flex items-center space-x-2">
                              <p className="text-sm text-gray-900 font-mono">{file.result.document.counterpart.vatNumber || '-'}</p>
                              {file.result.contragent?.vatValid && (
                                <span className="text-green-600" title="Валиден във VIES">✓</span>
                              )}
                            </div>
                          </div>
                          <div>
                            <label className="text-xs font-medium text-gray-600">Адрес</label>
                            <p className="text-sm text-gray-900">{file.result.document.counterpart.address || '-'}</p>
                          </div>
                        </div>

                        {file.result.existedInDatabase && (
                          <div className="flex items-center space-x-2 text-xs text-green-700 bg-green-50 px-2 py-1 rounded">
                            <span>✓</span>
                            <span>Контрагентът съществува в базата данни</span>
                          </div>
                        )}

                        {file.result.validationSource && (
                          <div className="text-xs text-gray-600">
                            Източник: {file.result.validationSource}
                          </div>
                        )}
                      </div>
                    )}

                    {file.result.document.items && file.result.document.items.length > 0 && (
                      <div className="border-t pt-3">
                        <h5 className="text-xs font-semibold text-gray-700 mb-2">
                          Артикули ({file.result.document.items.length})
                        </h5>
                        <div className="space-y-2 max-h-40 overflow-y-auto">
                          {file.result.document.items.map((item, idx) => (
                            <div key={idx} className="text-xs bg-white p-2 rounded border">
                              <div className="font-medium text-gray-900">{item.description}</div>
                              <div className="text-gray-600 mt-1">
                                {item.quantity} {item.unit} × {item.unitPrice} = {item.totalPrice} ({item.vatRate}% ДДС)
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {file.result.requiresManualReview && (
                      <div className="bg-yellow-50 border border-yellow-200 rounded p-3 flex items-start space-x-2">
                        <span className="text-yellow-600">⚠️</span>
                        <div className="text-xs text-yellow-800">
                          <div className="font-medium">Изисква ръчна проверка</div>
                          <div>Моля, прегледайте извлечените данни преди да ги запазите</div>
                        </div>
                      </div>
                    )}

                    {/* Create VAT Journal Button */}
                    {file.result.contragent && (
                      <div className="border-t pt-3 flex items-center justify-between">
                        {file.journalCreated ? (
                          <div className="flex items-center space-x-2 text-green-700">
                            <span>✓</span>
                            <span className="text-sm font-medium">ДДС операция създадена успешно</span>
                          </div>
                        ) : (
                          <button
                            onClick={() => createVatJournalEntry(index)}
                            disabled={file.creatingJournal}
                            className="px-4 py-2 bg-purple-600 text-white text-sm font-medium rounded-md hover:bg-purple-700 transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed flex items-center space-x-2"
                          >
                            {file.creatingJournal ? (
                              <>
                                <span className="animate-spin">⏳</span>
                                <span>Създаване...</span>
                              </>
                            ) : (
                              <>
                                <span>💼</span>
                                <span>Създай ДДС операция</span>
                              </>
                            )}
                          </button>
                        )}
                      </div>
                    )}
                  </div>
                )}

                {/* Error Details */}
                {file.status === 'error' && (
                  <div className="border-t bg-red-50 p-4">
                    <div className="flex items-start space-x-2">
                      <span className="text-red-600">❌</span>
                      <div className="text-sm text-red-800">
                        <div className="font-medium">Грешка при обработка</div>
                        <div className="text-xs mt-1">{file.error}</div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>

          <div className="flex justify-end space-x-3">
            <button
              onClick={() => {
                setUploadedFiles([]);
                setResults([]);
              }}
              disabled={processing}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50"
            >
              Изчисти всички
            </button>
            <button
              onClick={startAIScanning}
              disabled={processing || uploadedFiles.length === 0}
              className="px-6 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-2"
            >
              {processing ? (
                <>
                  <span className="animate-spin">⏳</span>
                  <span>Сканиране...</span>
                </>
              ) : (
                <>
                  <span>🤖</span>
                  <span>Започни AI Сканиране</span>
                </>
              )}
            </button>
          </div>
        </div>
      )}

      {/* Info Box */}
      <div className="bg-blue-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-blue-500 text-xl">💡</div>
          <div className="text-sm text-blue-800">
            <div className="font-medium mb-1">Как работи AI сканирането:</div>
            <ul className="space-y-1 text-sm">
              <li>• <strong>Изберете тип фактури</strong> - Покупки или Продажби (запазва се за сесията)</li>
              <li>• <strong>Изберете ДДС операция</strong> - в кой дневник да се запише фактурата</li>
              <li>• Качете снимка или PDF на фактура</li>
              <li>• Mistral AI автоматично разпознава текста и извлича данните</li>
              <li>• Системата валидира ДДС номера чрез EU VIES</li>
              <li>• Контрагентът се проверява в базата данни и се създава автоматично ако липсва</li>
              <li>• Извличат се всички артикули, суми, дати и данни</li>
              <li>• Създайте ДДС операция с един клик - автоматично отива в избрания дневник</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
