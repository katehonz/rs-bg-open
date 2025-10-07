import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';
import AccountSelectModal from '../components/AccountSelectModal';
import CounterpartSelectModal from '../components/CounterpartSelectModal';
import AddCounterpartModal from '../components/AddCounterpartModal';

// GraphQL queries
const ACCOUNTS_QUERY = `
  query GetAccounts($companyId: Int!) {
    accountHierarchy(companyId: $companyId) {
      id
      code
      name
      accountType
      accountClass
      isVatApplicable
      vatDirection
      isActive
      isAnalytical
    }
  }
`;

const COUNTERPARTS_QUERY = `
  query GetCounterparts($companyId: Int!) {
    counterparts(companyId: $companyId) {
      id
      name
      eik
      vatNumber
      isVatRegistered
      address
      city
      country
      isActive
    }
  }
`;

const CREATE_JOURNAL_ENTRY_MUTATION = `
  mutation CreateJournalEntry($input: CreateJournalEntryInput!) {
    createJournalEntry(input: $input) {
      id
      documentNumber
      description
      documentDate
      accountingDate
      vatDocumentType
      vatPurchaseOperation
      vatSalesOperation
      vatAdditionalOperation
      vatAdditionalData
    }
  }
`;

const UPDATE_JOURNAL_ENTRY_MUTATION = `
  mutation UpdateJournalEntry($id: Int!, $input: UpdateJournalEntryInput!) {
    updateJournalEntry(id: $id, input: $input) {
      id
      documentNumber
      description
      documentDate
      accountingDate
      vatDocumentType
      vatPurchaseOperation
      vatSalesOperation
      vatAdditionalOperation
      vatAdditionalData
    }
  }
`;


export default function VATEntry() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [accounts, setAccounts] = useState([]);
  const [counterparts, setCounterparts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [editingEntryId, setEditingEntryId] = useState(null);
  const [isEditMode, setIsEditMode] = useState(false);
  
  // Modal states
  const [showAccountModal, setShowAccountModal] = useState(false);
  const [showCounterpartModal, setShowCounterpartModal] = useState(false);
  const [showAddCounterpartModal, setShowAddCounterpartModal] = useState(false);
  const [currentLineIndex, setCurrentLineIndex] = useState(null);

  // VAT document types according to PPZDDS requirements
  const VAT_DOCUMENT_TYPES = {
    '01': 'Фактура',
    '02': 'Дебитно известие',
    '03': 'Кредитно известие',
    '04': 'Регистър на стоки под режим складиране на стоки до поискване, изпратени или транспортирани от територията на страната до територията на друга държава членка',
    '05': 'Регистър на стоки под режим складиране на стоки до поискване, получени на територията на страната',
    '07': 'Митническа декларация',
    '09': 'Протокол или друг документ',
    '11': 'Фактура - касова отчетност',
    '12': 'Дебитно известие - касова отчетност',
    '13': 'Кредитно известие - касова отчетност',
    '81': 'Отчет за извършените продажби',
    '82': 'Отчет за извършените продажби при специален ред на облагане',
    '91': 'Протокол за изискуемия данък по чл. 151в, ал. 3 от ЗДДС',
    '92': 'Протокол за данъчния кредит по чл. 151г, ал. 8 от ЗДДС или отчет по чл. 104ж, ал. 11',
    '93': 'Протокол за изискуемия данък по чл. 151в, ал. 7 от ЗДДС с получател по доставката лице, което не прилага специалния режим',
    '94': 'Протокол за изискуемия данък по чл. 151в, ал. 7 от ЗДДС с получател по доставката лице, което прилага специалния режим',
    '95': 'Протокол за безвъзмездно предоставяне на хранителни стоки, за което е приложим чл. 6, ал. 4, т. 4 от ЗДДС'
  };

  // VAT operations for purchases
  const VAT_PURCHASE_OPERATIONS = {
    '0': 'Не влиза в дневник',
    '1': 'Получени доставки с право на пълен данъчен кредит',
    '2': 'Получени доставки с право на частичен данъчен кредит',
    '3': 'Получени доставки без ДДС',
    '5': 'Придобиване от посредник в тристранна операция',
    '6': 'Получени доставки без право на ДКт'
  };

  // VAT operations for sales
  const VAT_SALES_OPERATIONS = {
    '0': 'Не влиза в дневник',
    '1': 'Облагаеми доставки с 20%',
    '2': 'Облагаеми доставки с 9%',
    '3': 'Облагаеми доставки с 0% по глава трета',
    '4': 'Вътреобщностни доставки',
    '5': 'Доставка на услуги по чл.21, ал.2 от ЗДДС с място на изпълнение на територията на друга държава членка',
    '6': 'Доставки по чл.69, ал.2 от ЗДДС',
    '7': 'Дистанционни продажби с място на изпълнение на територията на друга държава членка',
    '8': 'Освободени доставки',
    '9': 'Доставка от посредник в тристранна операция',
    '10': 'Доставки по чл. 140, 146, 173 от ЗДДС',
    '9001': 'ВОП',
    '9002': 'Получени доставки по чл.82, ал.2-6 от ЗДДС'
  };

  // Additional VAT operations
  const ADDITIONAL_VAT_OPERATIONS = {
    '0': 'Не участва във VIES декларация',
    '1': 'Вътреобщностна доставка',
    '2': 'Посредник в тристранна операция',
    '3': 'Доставки по чл.21, ал.2 от ЗДДС',
    '4': 'Изпращане на стоки под режим складиране',
    '5': 'Връщане на стоки под режим складиране',
    '6': 'Замяна на получател по чл.15а, ал.2, т.3 от ЗДДС',
    '7': 'Хляб',
    '8': 'Брашно'
  };

  // Additional data
  const ADDITIONAL_DATA = {
    '1': 'Част I на приложение 2',
    '2': 'Част II на приложение 2'
  };

  // VAT Operation data
  const [vatOperation, setVATOperation] = useState({
    // Document info
    documentNumber: '',
    documentDate: new Date().toISOString().split('T')[0],
    accountingDate: new Date().toISOString().split('T')[0],
    vatDate: new Date().toISOString().split('T')[0], // ДДС дата - определя отчетния период
    documentType: '01', // Bulgarian VAT document type codes
    description: '',
    
    // Counterpart
    counterpartId: null,
    counterpartName: '',
    counterpartEIK: '',
    counterpartVATNumber: '',
    counterpartAddress: '',
    
    // VAT info
    vatDirection: 'OUTPUT', // OUTPUT (sales), INPUT (purchases)
    baseAmount: 0,
    vatRate: 20, // 20, 9, 0
    vatAmount: 0,
    totalAmount: 0,
    
    // Bulgarian VAT operation codes
    purchaseOperation: '0',
    salesOperation: '0',
    additionalOperation: '0',
    additionalData: '',
    
    // Currency
    currencyCode: 'BGN',
    exchangeRate: 1
  });

  // Accounting lines - same structure as JournalEntry.jsx
  const [lines, setLines] = useState([
    {
      accountId: null,
      accountCode: '',
      accountName: '',
      debit: 0,
      credit: 0,
      description: '',
      currencyCode: 'BGN',
      exchangeRate: 1,
      quantity: 0,
      unitOfMeasure: '',
      unitPrice: 0,
      counterpartId: null
    },
    {
      accountId: null,
      accountCode: '',
      accountName: '',
      debit: 0,
      credit: 0,
      description: '',
      currencyCode: 'BGN',
      exchangeRate: 1,
      quantity: 0,
      unitOfMeasure: '',
      unitPrice: 0,
      counterpartId: null
    }
  ]);

  // Selected counterpart for the operation
  const [selectedCounterpart, setSelectedCounterpart] = useState(null);

  // Tab state for operation type
  const [activeTab, setActiveTab] = useState('vat'); // 'vat' or 'payment'

  // Payment lines (separate from VAT operation lines)
  const [paymentLines, setPaymentLines] = useState([
    {
      accountId: null,
      accountCode: '',
      accountName: '',
      debit: 0,
      credit: 0,
      description: '',
      currencyCode: 'BGN',
      exchangeRate: 1,
      quantity: 0,
      unitOfMeasure: '',
      unitPrice: 0,
      counterpartId: null
    },
    {
      accountId: null,
      accountCode: '',
      accountName: '',
      debit: 0,
      credit: 0,
      description: '',
      currencyCode: 'BGN',
      exchangeRate: 1,
      quantity: 0,
      unitOfMeasure: '',
      unitPrice: 0,
      counterpartId: null
    }
  ]);

  // Load data on mount
  useEffect(() => {
    const loadData = async () => {
      await loadAccounts();
      await loadCounterparts();
      
      // Check for edit mode from URL parameters
      const urlParams = new URLSearchParams(window.location.search);
      const editId = urlParams.get('edit');
      if (editId) {
        setEditingEntryId(parseInt(editId));
        setIsEditMode(true);
        await loadEntryForEdit(parseInt(editId));
      }
    };
    
    loadData();
  }, [companyId]);

  // Auto-calculate VAT when base amount or rate changes
  useEffect(() => {
    const vat = vatOperation.baseAmount * (vatOperation.vatRate / 100);
    const total = vatOperation.baseAmount + vat;
    
    setVATOperation(prev => ({
      ...prev,
      vatAmount: vat,
      totalAmount: total
    }));
  }, [vatOperation.baseAmount, vatOperation.vatRate]);

  const loadAccounts = async () => {
    try {
      const data = await graphqlRequest(ACCOUNTS_QUERY, { companyId });
      const activeAccounts = (data.accountHierarchy || []).filter(a => a.isActive);
      setAccounts(activeAccounts);
    } catch (err) {
      setError('Грешка при зареждане на сметките: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const loadCounterparts = async () => {
    try {
      const data = await graphqlRequest(COUNTERPARTS_QUERY, { companyId });
      const activeCounterparts = (data.counterparts || []).filter(c => c.isActive);
      setCounterparts(activeCounterparts);
    } catch (err) {
      console.error('Грешка при зареждане на контрагенти:', err.message);
    }
  };

  // Load journal entry for editing
  const loadEntryForEdit = async (entryId) => {
    try {
      const LOAD_ENTRY_QUERY = `
        query GetJournalEntryForEdit($id: Int!) {
          journalEntryWithLines(id: $id) {
            id
            entryNumber
            documentDate
            vatDate
            accountingDate
            documentNumber
            description
            totalAmount
            totalVatAmount
            isPosted
            vatDocumentType
            vatPurchaseOperation
            vatSalesOperation
            vatAdditionalOperation
            vatAdditionalData
            lines {
              id
              accountId
              debitAmount
              creditAmount
              counterpartId
              description
            }
          }
        }
      `;
      
      const data = await graphqlRequest(LOAD_ENTRY_QUERY, { id: entryId });
      const entryData = data.journalEntryWithLines;
      
      if (entryData) {
        // Determine VAT direction from operation type
        const hasVatPurchaseOperation = entryData.vatPurchaseOperation && entryData.vatPurchaseOperation !== '0';
        const hasVatSalesOperation = entryData.vatSalesOperation && entryData.vatSalesOperation !== '0';
        const vatDirection = hasVatSalesOperation ? 'OUTPUT' : (hasVatPurchaseOperation ? 'INPUT' : 'OUTPUT');
        
        // Update VAT operation with loaded data
        setVATOperation(prev => ({
          ...prev,
          documentDate: entryData.documentDate,
          vatDate: entryData.vatDate || entryData.documentDate,
          accountingDate: entryData.accountingDate,
          documentNumber: entryData.documentNumber || '',
          description: entryData.description,
          documentType: entryData.vatDocumentType || '01',
          vatDirection: vatDirection,
          purchaseOperation: entryData.vatPurchaseOperation || '0',
          salesOperation: entryData.vatSalesOperation || '0',
          additionalOperation: entryData.vatAdditionalOperation || '0',
          additionalData: entryData.vatAdditionalData || ''
        }));

        // Update VAT lines with loaded data
        if (entryData.lines && entryData.lines.length > 0) {
          // Load accounts if not loaded yet
          let currentAccounts = accounts;
          if (accounts.length === 0) {
            const accountData = await graphqlRequest(ACCOUNTS_QUERY, { companyId });
            currentAccounts = (accountData.accountHierarchy || []).filter(a => a.isActive);
            setAccounts(currentAccounts);
          }
          
          const transformedLines = entryData.lines.map(line => {
            const account = currentAccounts.find(acc => acc.id === line.accountId);
            return {
              accountId: line.accountId,
              accountCode: account?.code || '',
              accountName: account?.name || '',
              debit: parseFloat(line.debitAmount) || 0,
              credit: parseFloat(line.creditAmount) || 0,
              description: line.description || '',
              currencyCode: 'BGN',
              exchangeRate: 1,
              quantity: 0,
              unitOfMeasure: '',
              unitPrice: 0,
              counterpartId: line.counterpartId
            };
          });
          
          // Set lines for VAT operation (main lines)
          setLines(transformedLines);
          
          // Calculate VAT amounts from the loaded lines
          let vatAmount = 0;
          let baseAmount = 0;
          let vatRate = 20; // Default VAT rate
          
          // Find VAT and base amounts from the lines
          transformedLines.forEach(line => {
            const account = currentAccounts.find(acc => acc.id === line.accountId);
            if (account) {
              // Check if this is a VAT account (codes starting with 453)
              if (account.code && account.code.startsWith('453')) {
                vatAmount += (line.debit || 0) + (line.credit || 0);
              }
              // Check if this is an expense/revenue account (class 6 or 7)
              else if (account.accountClass === 6 || account.accountClass === 7) {
                baseAmount += (line.debit || 0) + (line.credit || 0);
              }
            }
          });
          
          // Calculate VAT rate if we have both amounts
          if (baseAmount > 0 && vatAmount > 0) {
            vatRate = Math.round((vatAmount / baseAmount) * 100);
          }
          
          const totalAmount = baseAmount + vatAmount;
          
          // Update VAT operation amounts
          setVATOperation(prev => ({
            ...prev,
            baseAmount: baseAmount,
            vatRate: vatRate,
            vatAmount: vatAmount,
            totalAmount: totalAmount
          }));
          
          // Reset payment lines to empty defaults when editing VAT entry
          const resetLine = { accountId: null, accountCode: '', accountName: '', debit: 0, credit: 0, description: '', currencyCode: 'BGN', exchangeRate: 1, quantity: 0, unitOfMeasure: '', unitPrice: 0, counterpartId: null };
          setPaymentLines([resetLine, { ...resetLine }]);
        }
        
        // Find and set the counterpart if any line has one
        const firstCounterpartId = entryData.lines.find(line => line.counterpartId)?.counterpartId;
        if (firstCounterpartId) {
          // Load counterparts if not loaded yet, then find counterpart
          if (counterparts.length === 0) {
            const counterpartData = await graphqlRequest(COUNTERPARTS_QUERY, { companyId });
            const activeCounterparts = (counterpartData.counterparts || []).filter(c => c.isActive);
            setCounterparts(activeCounterparts);
            
            const counterpart = activeCounterparts.find(c => c.id === firstCounterpartId);
            if (counterpart) {
              setSelectedCounterpart(counterpart);
              // Also update VAT operation with counterpart info
              setVATOperation(prev => ({
                ...prev,
                counterpartId: counterpart.id,
                counterpartName: counterpart.name,
                counterpartEIK: counterpart.eik || ''
              }));
            }
          } else {
            const counterpart = counterparts.find(c => c.id === firstCounterpartId);
            if (counterpart) {
              setSelectedCounterpart(counterpart);
              // Also update VAT operation with counterpart info
              setVATOperation(prev => ({
                ...prev,
                counterpartId: counterpart.id,
                counterpartName: counterpart.name,
                counterpartEIK: counterpart.eik || ''
              }));
            }
          }
        }
      }
    } catch (err) {
      setError('Грешка при зареждане на записа: ' + err.message);
    }
  };

  const handleCounterpartSelect = (counterpart) => {
    setSelectedCounterpart(counterpart);
    setVATOperation(prev => ({
      ...prev,
      counterpartId: counterpart.id,
      counterpartName: counterpart.name,
      counterpartEIK: counterpart.eik || '',
      counterpartVATNumber: counterpart.vatNumber || '',
      counterpartAddress: counterpart.address || ''
    }));
    
    // Auto-fill counterpart in all lines (both VAT and payment)
    const newLines = lines.map(line => ({
      ...line,
      counterpartId: counterpart.id
    }));
    setLines(newLines);

    const newPaymentLines = paymentLines.map(line => ({
      ...line,
      counterpartId: counterpart.id
    }));
    setPaymentLines(newPaymentLines);
    
    setShowCounterpartModal(false);
  };

  const openAccountModal = (lineIndex) => {
    setCurrentLineIndex(lineIndex);
    setShowAccountModal(true);
  };

  const handleAccountSelect = (account) => {
    if (currentLineIndex !== null) {
      if (activeTab === 'vat') {
        const newLines = [...lines];
        newLines[currentLineIndex] = {
          ...newLines[currentLineIndex],
          accountId: account.id,
          accountCode: account.code,
          accountName: account.name
        };
        setLines(newLines);
      } else {
        const newLines = [...paymentLines];
        newLines[currentLineIndex] = {
          ...newLines[currentLineIndex],
          accountId: account.id,
          accountCode: account.code,
          accountName: account.name
        };
        setPaymentLines(newLines);
      }
    }
    setShowAccountModal(false);
    setCurrentLineIndex(null);
  };

  const updateLineAmount = (index, field, value) => {
    const amount = parseFloat(value) || 0;
    
    if (activeTab === 'vat') {
      const newLines = [...lines];
      if (field === 'debit') {
        newLines[index].debit = amount;
        newLines[index].credit = 0;
      } else {
        newLines[index].credit = amount;
        newLines[index].debit = 0;
      }
      setLines(newLines);
    } else {
      const newLines = [...paymentLines];
      if (field === 'debit') {
        newLines[index].debit = amount;
        newLines[index].credit = 0;
      } else {
        newLines[index].credit = amount;
        newLines[index].debit = 0;
      }
      setPaymentLines(newLines);
    }
  };

  // Handle currency change
  const handleCurrencyChange = async (lineIndex, currencyCode) => {
    const rates = { 'EUR': 1.9558, 'USD': 1.8234, 'GBP': 2.3567 };
    const rate = currencyCode === 'BGN' ? 1 : (rates[currencyCode] || 1);

    if (activeTab === 'vat') {
      const newLines = [...lines];
      newLines[lineIndex].exchangeRate = rate;
      newLines[lineIndex].currencyCode = currencyCode;
      setLines(newLines);
    } else {
      const newLines = [...paymentLines];
      newLines[lineIndex].exchangeRate = rate;
      newLines[lineIndex].currencyCode = currencyCode;
      setPaymentLines(newLines);
    }
  };

  const addLine = () => {
    const newLine = {
      accountId: null,
      accountCode: '',
      accountName: '',
      debit: 0,
      credit: 0,
      description: '',
      currencyCode: 'BGN',
      exchangeRate: 1,
      quantity: 0,
      unitOfMeasure: '',
      unitPrice: 0,
      counterpartId: selectedCounterpart?.id || null
    };

    if (activeTab === 'vat') {
      setLines(prev => [...prev, newLine]);
    } else {
      setPaymentLines(prev => [...prev, newLine]);
    }
  };

  const removeLine = (index) => {
    if (activeTab === 'vat') {
      if (lines.length > 2) {
        setLines(prev => prev.filter((_, i) => i !== index));
      }
    } else {
      if (paymentLines.length > 2) {
        setPaymentLines(prev => prev.filter((_, i) => i !== index));
      }
    }
  };

  // Update quantities and prices for both tabs
  const updateLineField = (index, field, value) => {
    if (activeTab === 'vat') {
      const newLines = [...lines];
      newLines[index][field] = field === 'quantity' || field === 'unitPrice' || field === 'exchangeRate' 
        ? (parseFloat(value) || 0) 
        : value;
      setLines(newLines);
    } else {
      const newLines = [...paymentLines];
      newLines[index][field] = field === 'quantity' || field === 'unitPrice' || field === 'exchangeRate'
        ? (parseFloat(value) || 0)
        : value;
      setPaymentLines(newLines);
    }
  };

  // Calculate totals for current tab
  const currentLines = activeTab === 'vat' ? lines : paymentLines;
  const totals = currentLines.reduce((acc, line) => ({
    debit: acc.debit + (parseFloat(line.debit) || 0),
    credit: acc.credit + (parseFloat(line.credit) || 0)
  }), { debit: 0, credit: 0 });

  const isBalanced = Math.abs(totals.debit - totals.credit) < 0.01;

  const saveEntry = async () => {
    if (!vatOperation.documentNumber || !vatOperation.description || !vatOperation.vatDate) {
      setError('Моля попълнете задължителните полета! ДДС датата е задължителна за определяне на отчетния период.');
      return;
    }

    // Validate VAT date is in the correct format and within reasonable bounds
    const vatDate = new Date(vatOperation.vatDate);
    const currentDate = new Date();
    const maxFutureDate = new Date(currentDate.getFullYear() + 1, 11, 31); // End of next year
    const minPastDate = new Date(currentDate.getFullYear() - 5, 0, 1); // Start of 5 years ago

    if (vatDate > maxFutureDate) {
      setError('ДДС датата не може да бъде повече от 1 година в бъдещето!');
      return;
    }

    if (vatDate < minPastDate) {
      setError('ДДС датата не може да бъде повече от 5 години в миналото!');
      return;
    }

    // Get VAT month for the operation
    const vatMonth = vatDate.getMonth() + 1; // 1-12
    const vatYear = vatDate.getFullYear();
    console.log(`ДДС операцията ще попадне в отчетен период: ${vatMonth.toString().padStart(2, '0')}/${vatYear}`);
    
    setError(null); // Clear any previous errors

    // Get valid lines from both tabs
    const validVATLines = lines.filter(line => 
      line.accountId && (line.debit > 0 || line.credit > 0)
    );
    
    const validPaymentLines = paymentLines.filter(line => 
      line.accountId && (line.debit > 0 || line.credit > 0)
    );

    const allEntries = [];

    // Add VAT operation entry if has valid lines
    if (validVATLines.length > 0) {
      // Check VAT operation balance
      const vatTotals = validVATLines.reduce((acc, line) => ({
        debit: acc.debit + (parseFloat(line.debit) || 0),
        credit: acc.credit + (parseFloat(line.credit) || 0)
      }), { debit: 0, credit: 0 });

      if (Math.abs(vatTotals.debit - vatTotals.credit) >= 0.01) {
        setError(`ДДС операцията не е балансирана! Дебит: ${vatTotals.debit.toFixed(2)} лв., Кредит: ${vatTotals.credit.toFixed(2)} лв.`);
        return;
      }

      allEntries.push({
        documentNumber: vatOperation.documentNumber,
        description: vatOperation.description,
        documentDate: vatOperation.documentDate,
        accountingDate: vatOperation.accountingDate,
        vatDate: vatOperation.vatDate, // ДДС дата определя отчетния период
        companyId: companyId,
        vatDocumentType: vatOperation.documentType,
        vatPurchaseOperation: vatOperation.vatDirection === 'INPUT' ? vatOperation.purchaseOperation : null,
        vatSalesOperation: vatOperation.vatDirection === 'OUTPUT' ? vatOperation.salesOperation : null,
        vatAdditionalOperation: vatOperation.additionalOperation !== '0' ? vatOperation.additionalOperation : null,
        vatAdditionalData: vatOperation.additionalData || null,
        lines: validVATLines.map(line => ({
          accountId: parseInt(line.accountId),
          debitAmount: line.debit > 0 ? line.debit : null,
          creditAmount: line.credit > 0 ? line.credit : null,
          description: line.description || vatOperation.description,
          counterpartId: line.counterpartId ? parseInt(line.counterpartId) : null
        }))
      });
    }

    // Add payment entry if has valid lines
    if (validPaymentLines.length > 0) {
      // Check payment balance
      const paymentTotals = validPaymentLines.reduce((acc, line) => ({
        debit: acc.debit + (parseFloat(line.debit) || 0),
        credit: acc.credit + (parseFloat(line.credit) || 0)
      }), { debit: 0, credit: 0 });

      if (Math.abs(paymentTotals.debit - paymentTotals.credit) >= 0.01) {
        setError(`Плащането не е балансирано! Дебит: ${paymentTotals.debit.toFixed(2)} лв., Кредит: ${paymentTotals.credit.toFixed(2)} лв.`);
        return;
      }

      allEntries.push({
        documentNumber: vatOperation.documentNumber + '-PAY',
        description: 'Плащане: ' + vatOperation.description,
        documentDate: vatOperation.documentDate,
        accountingDate: vatOperation.accountingDate,
        vatDate: vatOperation.documentDate,
        companyId: companyId,
        vatDocumentType: null, // Payment entries don't have VAT codes
        vatPurchaseOperation: null,
        vatSalesOperation: null,
        vatAdditionalOperation: null,
        vatAdditionalData: null,
        lines: validPaymentLines.map(line => ({
          accountId: parseInt(line.accountId),
          debitAmount: line.debit > 0 ? line.debit : null,
          creditAmount: line.credit > 0 ? line.credit : null,
          description: line.description || ('Плащане: ' + vatOperation.description),
          counterpartId: line.counterpartId ? parseInt(line.counterpartId) : null
        }))
      });
    }

    if (allEntries.length === 0) {
      setError('Няма валидни записи за запазване!');
      return;
    }

    try {
      if (isEditMode && editingEntryId) {
        // Update existing entry
        const entryInput = allEntries[0]; // VAT entries are single entries
        if (entryInput) {
          await graphqlRequest(UPDATE_JOURNAL_ENTRY_MUTATION, { 
            id: editingEntryId, 
            input: {
              documentNumber: entryInput.documentNumber,
              description: entryInput.description,
              documentDate: entryInput.documentDate,
              accountingDate: entryInput.accountingDate,
              vatDocumentType: entryInput.vatDocumentType,
              vatPurchaseOperation: entryInput.vatPurchaseOperation,
              vatSalesOperation: entryInput.vatSalesOperation,
              vatAdditionalOperation: entryInput.vatAdditionalOperation,
              vatAdditionalData: entryInput.vatAdditionalData,
              lines: entryInput.lines
            }
          });
        }
      } else {
        // Save all entries
        for (const entryInput of allEntries) {
          await graphqlRequest(CREATE_JOURNAL_ENTRY_MUTATION, { input: entryInput });
        }
      }
      
      // Reset form
      const resetLine = { accountId: null, accountCode: '', accountName: '', debit: 0, credit: 0, description: '', currencyCode: 'BGN', exchangeRate: 1, quantity: 0, unitOfMeasure: '', unitPrice: 0, counterpartId: null };
      
      setVATOperation({
        documentNumber: '',
        documentDate: new Date().toISOString().split('T')[0],
        accountingDate: new Date().toISOString().split('T')[0],
        documentType: '01',
        description: '',
        counterpartId: null,
        counterpartName: '',
        counterpartEIK: '',
        counterpartVATNumber: '',
        counterpartAddress: '',
        vatDirection: 'OUTPUT',
        baseAmount: 0,
        vatRate: 20,
        vatAmount: 0,
        totalAmount: 0,
        purchaseOperation: '0',
        salesOperation: '0',
        additionalOperation: '0',
        additionalData: '',
        currencyCode: 'BGN',
        exchangeRate: 1
      });
      
      setLines([resetLine, { ...resetLine }]);
      setPaymentLines([resetLine, { ...resetLine }]);
      setActiveTab('vat');
      setSelectedCounterpart(null);
      setError(null);
      alert(`${allEntries.length} записа са създадени успешно!`);
      
    } catch (err) {
      setError('Грешка при запазване: ' + err.message);
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
          <h1 className="text-2xl font-bold text-gray-900">
            {isEditMode ? 'Редакция на ДДС операция' : 'Нова ДДС операция'}
          </h1>
          <p className="mt-1 text-sm text-gray-500">
            Въведете ДДС операция с автоматично генерирани счетоводни записи
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

      {/* VAT Operation Form */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <span className="w-2 h-2 bg-purple-400 rounded-full mr-2"></span>
          ДДС Информация
        </h3>
        
        {/* Basic Document Info */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Вид документ *
            </label>
            <select
              value={vatOperation.documentType}
              onChange={(e) => setVATOperation(prev => ({ ...prev, documentType: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              {Object.entries(VAT_DOCUMENT_TYPES).map(([code, name]) => (
                <option key={code} value={code}>
                  {code} - {name}
                </option>
              ))}
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Номер на документ *
            </label>
            <input
              type="text"
              value={vatOperation.documentNumber}
              onChange={(e) => setVATOperation(prev => ({ ...prev, documentNumber: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm font-mono"
              placeholder="0000000001"
              maxLength="10"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Дата на документ *
            </label>
            <input
              type="date"
              value={vatOperation.documentDate}
              onChange={(e) => setVATOperation(prev => ({ ...prev, documentDate: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Счетоводна дата *
            </label>
            <input
              type="date"
              value={vatOperation.accountingDate}
              onChange={(e) => setVATOperation(prev => ({ ...prev, accountingDate: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              ДДС дата *
            </label>
            <input
              type="date"
              value={vatOperation.vatDate}
              onChange={(e) => setVATOperation(prev => ({ ...prev, vatDate: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              title="Определя отчетния период за ДДС декларацията"
            />
            <p className="text-xs text-blue-600 mt-1 font-medium">
              Период: {vatOperation.vatDate ? 
                (() => {
                  const date = new Date(vatOperation.vatDate);
                  const month = (date.getMonth() + 1).toString().padStart(2, '0');
                  const year = date.getFullYear();
                  return `${month}/${year}`;
                })() 
                : '--/----'
              }
            </p>
          </div>
        </div>

        {/* Description */}
        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Описание *
          </label>
          <input
            type="text"
            value={vatOperation.description}
            onChange={(e) => setVATOperation(prev => ({ ...prev, description: e.target.value }))}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            placeholder="Описание на операцията"
          />
        </div>

        {/* Counterpart Selection */}
        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Контрагент *
          </label>
          <div className="flex items-center space-x-4">
            <div className="flex-1">
              {selectedCounterpart ? (
                <div className="flex items-center justify-between p-3 border border-gray-300 rounded-md bg-gray-50">
                  <div>
                    <div className="font-medium text-gray-900">{selectedCounterpart.name}</div>
                    <div className="text-sm text-gray-600">
                      {selectedCounterpart.eik && `БУЛСТАТ: ${selectedCounterpart.eik}`}
                      {selectedCounterpart.vatNumber && selectedCounterpart.isVatRegistered && 
                        ` • ДДС: ${selectedCounterpart.vatNumber}`}
                    </div>
                  </div>
                  <button
                    onClick={() => {
                      setSelectedCounterpart(null);
                      setVATOperation(prev => ({
                        ...prev,
                        counterpartId: null,
                        counterpartName: '',
                        counterpartEIK: '',
                        counterpartVATNumber: '',
                        counterpartAddress: ''
                      }));
                    }}
                    className="text-red-600 hover:text-red-800 ml-2"
                  >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              ) : (
                <button
                  onClick={() => setShowCounterpartModal(true)}
                  className="w-full p-3 border border-gray-300 border-dashed rounded-md text-gray-500 hover:border-gray-400 hover:text-gray-600 text-left"
                >
                  Избери контрагент...
                </button>
              )}
            </div>
            <button
              onClick={() => setShowCounterpartModal(true)}
              className="px-3 py-2 bg-blue-50 text-blue-600 rounded-md text-sm hover:bg-blue-100"
            >
              Търси
            </button>
            <button
              onClick={() => setShowAddCounterpartModal(true)}
              className="px-3 py-2 bg-green-600 text-white rounded-md text-sm hover:bg-green-700"
            >
              Нов
            </button>
          </div>
        </div>

        {/* VAT Details */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Операция
            </label>
            <select
              value={vatOperation.vatDirection}
              onChange={(e) => setVATOperation(prev => ({ ...prev, vatDirection: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="OUTPUT">Продажба (изходящо)</option>
              <option value="INPUT">Покупка (входящо)</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Основа (без ДДС)
            </label>
            <input
              type="number"
              step="0.01"
              value={vatOperation.baseAmount}
              onChange={(e) => setVATOperation(prev => ({ ...prev, baseAmount: parseFloat(e.target.value) || 0 }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              placeholder="0.00"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              ДДС ставка (%)
            </label>
            <select
              value={vatOperation.vatRate}
              onChange={(e) => setVATOperation(prev => ({ ...prev, vatRate: parseInt(e.target.value) }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value={20}>20%</option>
              <option value={9}>9%</option>
              <option value={0}>0%</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              ДДС сума
            </label>
            <input
              type="number"
              step="0.01"
              value={vatOperation.vatAmount.toFixed(2)}
              readOnly
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Всичко с ДДС
            </label>
            <input
              type="number"
              step="0.01"
              value={vatOperation.totalAmount.toFixed(2)}
              readOnly
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50 font-medium"
            />
          </div>
        </div>

        {/* Bulgarian VAT Operation Codes */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {vatOperation.vatDirection === 'INPUT' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                ДДС операция покупка
              </label>
              <select
                value={vatOperation.purchaseOperation}
                onChange={(e) => setVATOperation(prev => ({ ...prev, purchaseOperation: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              >
                {Object.entries(VAT_PURCHASE_OPERATIONS).map(([code, name]) => (
                  <option key={code} value={code}>
                    {code} - {name}
                  </option>
                ))}
              </select>
            </div>
          )}

          {vatOperation.vatDirection === 'OUTPUT' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                ДДС операция продажба
              </label>
              <select
                value={vatOperation.salesOperation}
                onChange={(e) => setVATOperation(prev => ({ ...prev, salesOperation: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              >
                {Object.entries(VAT_SALES_OPERATIONS).map(([code, name]) => (
                  <option key={code} value={code}>
                    {code} - {name}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Допълнителна операция
            </label>
            <select
              value={vatOperation.additionalOperation}
              onChange={(e) => setVATOperation(prev => ({ ...prev, additionalOperation: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              {Object.entries(ADDITIONAL_VAT_OPERATIONS).map(([code, name]) => (
                <option key={code} value={code}>
                  {code} - {name}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Допълнителни данни
            </label>
            <select
              value={vatOperation.additionalData}
              onChange={(e) => setVATOperation(prev => ({ ...prev, additionalData: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="">-</option>
              {Object.entries(ADDITIONAL_DATA).map(([code, name]) => (
                <option key={code} value={code}>
                  {code} - {name}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Operation Tabs */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="border-b border-gray-200">
          <nav className="flex space-x-8">
            <button
              onClick={() => setActiveTab('vat')}
              className={`py-2 px-1 border-b-2 font-medium text-sm flex items-center space-x-2 ${
                activeTab === 'vat'
                  ? 'border-purple-500 text-purple-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>💼</span>
              <span>ДДС Операция</span>
            </button>
            <button
              onClick={() => setActiveTab('payment')}
              className={`py-2 px-1 border-b-2 font-medium text-sm flex items-center space-x-2 ${
                activeTab === 'payment'
                  ? 'border-green-500 text-green-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>💳</span>
              <span>Плащане</span>
            </button>
          </nav>
        </div>

        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-900 flex items-center">
              <span className={`w-2 h-2 rounded-full mr-2 ${
                activeTab === 'vat' ? 'bg-purple-400' : 'bg-green-400'
              }`}></span>
              {activeTab === 'vat' ? 'ДДС Счетоводни записи' : 'Плащане Счетоводни записи'}
            </h3>
            <button
              onClick={addLine}
              className="inline-flex items-center px-3 py-1.5 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
              </svg>
              Добави ред
            </button>
          </div>
        </div>
        
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-48">
                  Сметка
                </th>
                <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-32">
                  Контрагент
                </th>
                <th className="px-3 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider w-24">
                  Дебит
                </th>
                <th className="px-3 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider w-24">
                  Кредит
                </th>
                <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                  Валута
                </th>
                <th className="px-3 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                  Курс
                </th>
                <th className="px-3 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                  Количество
                </th>
                <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-16">
                  Мярка
                </th>
                <th className="px-3 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                  Ед. цена
                </th>
                <th className="px-3 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-12">
                  Действия
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {currentLines.map((line, index) => (
                <tr key={index}>
                  {/* Сметка */}
                  <td className="px-3 py-2">
                    <button
                      onClick={() => openAccountModal(index)}
                      className="w-full text-left px-2 py-1 border border-gray-300 rounded text-xs hover:bg-gray-50"
                    >
                      {line.accountCode ? (
                        <span>
                          <span className="font-mono font-medium">{line.accountCode}</span>
                          <br />
                          <span className="text-gray-600">{line.accountName}</span>
                        </span>
                      ) : (
                        <span className="text-gray-400">Избери сметка...</span>
                      )}
                    </button>
                  </td>

                  {/* Контрагент */}
                  <td className="px-3 py-2">
                    {line.counterpartId ? (
                      <div className="text-xs">
                        {(() => {
                          const counterpart = counterparts.find(c => c.id === line.counterpartId);
                          return counterpart ? (
                            <div>
                              <div className="font-medium text-gray-700 truncate" title={counterpart.name}>
                                {counterpart.name.length > 15 ? counterpart.name.substring(0, 15) + '...' : counterpart.name}
                              </div>
                              {counterpart.eik && (
                                <div className="text-gray-500 font-mono">
                                  {counterpart.eik}
                                </div>
                              )}
                            </div>
                          ) : (
                            <span className="text-gray-400">Не е намерен</span>
                          );
                        })()}
                      </div>
                    ) : (
                      <span className="text-gray-400 text-xs">-</span>
                    )}
                  </td>

                  {/* Дебит */}
                  <td className="px-3 py-2">
                    <input
                      type="number"
                      value={line.debit || ''}
                      onChange={(e) => updateLineAmount(index, 'debit', e.target.value)}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs text-right"
                      placeholder="0.00"
                      step="0.01"
                      min="0"
                    />
                  </td>

                  {/* Кредит */}
                  <td className="px-3 py-2">
                    <input
                      type="number"
                      value={line.credit || ''}
                      onChange={(e) => updateLineAmount(index, 'credit', e.target.value)}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs text-right"
                      placeholder="0.00"
                      step="0.01"
                      min="0"
                    />
                  </td>

                  {/* Валута */}
                  <td className="px-3 py-2">
                    <select
                      value={line.currencyCode}
                      onChange={(e) => handleCurrencyChange(index, e.target.value)}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs"
                    >
                      <option value="BGN">BGN</option>
                      <option value="EUR">EUR</option>
                      <option value="USD">USD</option>
                      <option value="GBP">GBP</option>
                    </select>
                  </td>

                  {/* Курс */}
                  <td className="px-3 py-2">
                    {line.currencyCode === 'BGN' ? (
                      <span className="text-xs text-gray-500">1.00</span>
                    ) : (
                      <input
                        type="number"
                        value={line.exchangeRate.toFixed(6)}
                        readOnly
                        className="w-full px-2 py-1 border border-gray-300 rounded text-xs text-right bg-gray-50"
                        title="Курс от БНБ"
                      />
                    )}
                  </td>

                  {/* Количество */}
                  <td className="px-3 py-2">
                    <input
                      type="number"
                      value={line.quantity || ''}
                      onChange={(e) => updateLineField(index, 'quantity', e.target.value)}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs text-right"
                      placeholder="0"
                      step="0.001"
                      min="0"
                    />
                  </td>

                  {/* Мярка */}
                  <td className="px-3 py-2">
                    <select
                      value={line.unitOfMeasure}
                      onChange={(e) => updateLineField(index, 'unitOfMeasure', e.target.value)}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs"
                    >
                      <option value="">-</option>
                      <option value="бр">бр</option>
                      <option value="кг">кг</option>
                      <option value="м">м</option>
                      <option value="л">л</option>
                      <option value="час">час</option>
                    </select>
                  </td>

                  {/* Ед. цена */}
                  <td className="px-3 py-2">
                    <input
                      type="number"
                      value={line.unitPrice || ''}
                      onChange={(e) => updateLineField(index, 'unitPrice', e.target.value)}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs text-right"
                      placeholder="0.00"
                      step="0.01"
                      min="0"
                    />
                  </td>

                  {/* Действия */}
                  <td className="px-3 py-2 text-center">
                    {currentLines.length > 2 && (
                      <button
                        onClick={() => removeLine(index)}
                        className="text-red-600 hover:text-red-900"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    )}
                  </td>
                </tr>
              ))}
              
              {/* Totals Row */}
              <tr className="bg-gray-50 font-semibold">
                <td className="px-3 py-3 text-right">Общо:</td>
                <td></td>
                <td className="px-3 py-3 text-right">
                  {totals.debit.toFixed(2)} лв.
                </td>
                <td className="px-3 py-3 text-right">
                  {totals.credit.toFixed(2)} лв.
                </td>
                <td colSpan="6"></td>
              </tr>
              
              {/* Balance Status */}
              <tr>
                <td colSpan="10" className="px-6 py-3">
                  <div className={`text-center font-medium ${
                    isBalanced ? 'text-green-600' : 'text-red-600'
                  }`}>
                    {isBalanced 
                      ? '✓ Записът е балансиран' 
                      : `⚠️ Разлика: ${Math.abs(totals.debit - totals.credit).toFixed(2)} лв.`
                    }
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center justify-end space-x-3">
        <button
          onClick={() => window.history.back()}
          className="px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
        >
          Отказ
        </button>
        <button
          onClick={saveEntry}
          disabled={!isBalanced || !vatOperation.documentNumber || !vatOperation.description}
          className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          Запази ДДС операция
        </button>
      </div>

      {/* Modals */}
      <AccountSelectModal
        show={showAccountModal}
        accounts={accounts}
        currentAccountId={currentLines[currentLineIndex]?.accountId}
        onSelect={handleAccountSelect}
        onClose={() => setShowAccountModal(false)}
      />

      <CounterpartSelectModal
        show={showCounterpartModal}
        counterparts={counterparts}
        currentCounterpartId={selectedCounterpart?.id}
        onSelect={handleCounterpartSelect}
        onClose={() => setShowCounterpartModal(false)}
        onAddNew={() => {
          setShowCounterpartModal(false);
          setShowAddCounterpartModal(true);
        }}
      />

      <AddCounterpartModal
        show={showAddCounterpartModal}
        onSave={(counterpartData) => {
          // Handle creating new counterpart
          console.log('Create counterpart:', counterpartData);
          setShowAddCounterpartModal(false);
        }}
        onClose={() => setShowAddCounterpartModal(false)}
      />
    </div>
  );
}
