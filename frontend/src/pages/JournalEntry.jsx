import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';
import AccountSelectModal from '../components/AccountSelectModal';
import CounterpartSelectModal from '../components/CounterpartSelectModal';
import AddCounterpartModal from '../components/AddCounterpartModal';
import AverageCostButton from '../components/AverageCostButton';

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
      supportsQuantities
      defaultUnit
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
      street
      address
      city
      postalCode
      country
      isActive
    }
  }
`;

const CREATE_COUNTERPART_MUTATION = `
  mutation CreateCounterpart($input: CreateCounterpartInput!) {
    createCounterpart(input: $input) {
      id
      name
      eik
      vatNumber
      isVatRegistered
      street
      address
      city
      postalCode
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
      lines {
        id
        accountId
        debitAmount
        creditAmount
        description
      }
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
    }
  }
`;


export default function JournalEntry() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [accounts, setAccounts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [editingEntryId, setEditingEntryId] = useState(null);
  const [isEditMode, setIsEditMode] = useState(false);
  const [showAccountModal, setShowAccountModal] = useState(false);
  const [currentLineIndex, setCurrentLineIndex] = useState(null);
  const [activeGroupIndex, setActiveGroupIndex] = useState(0);
  
  // Document types
  const documentTypes = {
    'INVOICE': { label: 'Фактура', code: 'ФК', isTaxDocument: true },
    'CREDIT_NOTE': { label: 'Кредитно известие', code: 'КИ', isTaxDocument: true },
    'DEBIT_NOTE': { label: 'Дебитно известие', code: 'ДИ', isTaxDocument: true },
    'PROTOCOL': { label: 'Протокол', code: 'ПР', isTaxDocument: true },
    'LEDGER': { label: 'Ведомост', code: 'ВД', isTaxDocument: false },
    'CASH_RECEIPT_ORDER': { label: 'ПКО', code: 'ПКО', isTaxDocument: false },
    'CASH_PAYMENT_ORDER': { label: 'РКО', code: 'РКО', isTaxDocument: false },
    'BANK_STATEMENT': { label: 'Банково извлечение', code: 'БИ', isTaxDocument: false },
    'OTHER': { label: 'Друг документ', code: 'ДР', isTaxDocument: false }
  };

  // Group types for linked documents
  const groupTypes = {
    'MAIN': { label: 'Основна операция', color: 'blue' },
    'PAYMENT': { label: 'Плащане', color: 'green' },
    'EXPENSE': { label: 'Разход', color: 'orange' },
    'VAT': { label: 'ДДС', color: 'purple' },
    'OTHER': { label: 'Друго', color: 'gray' }
  };

  // Form state with document groups (статии)
  const [documentGroups, setDocumentGroups] = useState([
    {
      id: 0,
      name: 'Основен документ',
      type: 'MAIN',
      documentNumber: '',
      description: '',
      documentDate: new Date().toISOString().split('T')[0],
      accountingDate: new Date().toISOString().split('T')[0],
      documentType: '',
      lines: [
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
      ]
    }
  ]);

  // Legacy entry state for backward compatibility - now points to main group
  const [entry, setEntry] = useState(documentGroups[0]);

  const [currencies] = useState([
    { code: 'EUR', name: 'Евро' },
    { code: 'USD', name: 'Долар САЩ' },
    { code: 'GBP', name: 'Британска лира' }
  ]);
  const [documentNumberError, setDocumentNumberError] = useState('');
  const [linkedGroups, setLinkedGroups] = useState([]);
  const [nextGroupId, setNextGroupId] = useState(1);
  const [selectedLines, setSelectedLines] = useState([]);
  const [showGroupingMode, setShowGroupingMode] = useState(false);
  const [counterparts, setCounterparts] = useState([]);
  const [selectedCounterpart, setSelectedCounterpart] = useState(null);
  const [showCounterpartModal, setShowCounterpartModal] = useState(false);
  const [showAddCounterpartModal, setShowAddCounterpartModal] = useState(false);
  const [showGroupMenu, setShowGroupMenu] = useState(false);

  // Load accounts and counterparts
  useEffect(() => {
    const loadData = async () => {
      const loadedAccounts = await loadAccounts();
      await loadCounterparts();
      
      // Check for edit mode from URL parameters after loading accounts
      const urlParams = new URLSearchParams(window.location.search);
      const editId = urlParams.get('edit');
      if (editId) {
        setEditingEntryId(parseInt(editId));
        setIsEditMode(true);
        await loadEntryForEdit(parseInt(editId), loadedAccounts);
      }
    };
    
    loadData();
  }, [companyId]);

  // Update entry when active group changes
  useEffect(() => {
    setEntry(documentGroups[activeGroupIndex] || documentGroups[0]);
  }, [activeGroupIndex, documentGroups]);

  const loadAccounts = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const data = await graphqlRequest(ACCOUNTS_QUERY, { companyId });
      // Allow both synthetic and analytical accounts for selection
      const activeAccounts = (data.accountHierarchy || []).filter(a => a.isActive);
      setAccounts(activeAccounts);
      return activeAccounts;
    } catch (err) {
      setError('Грешка при зареждане на сметките: ' + err.message);
      throw err;
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
  const loadEntryForEdit = async (entryId, accountsList = null) => {
    const accountsToUse = accountsList || accounts;
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
        // Transform to match the form structure
        const transformedEntry = {
          entryNumber: entryData.entryNumber,
          documentDate: entryData.documentDate,
          vatDate: entryData.vatDate || '',
          accountingDate: entryData.accountingDate,
          documentNumber: entryData.documentNumber || '',
          description: entryData.description,
          vatDocumentType: entryData.vatDocumentType || '',
          vatPurchaseOperation: entryData.vatPurchaseOperation || '',
          vatSalesOperation: entryData.vatSalesOperation || '',
          vatAdditionalOperation: entryData.vatAdditionalOperation || '',
          vatAdditionalData: entryData.vatAdditionalData || '',
          lines: entryData.lines.map(line => {
            const account = accountsToUse.find(acc => acc.id === line.accountId);
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
              counterpartId: line.counterpartId
            }
          })
        };
        
        setEntry(transformedEntry);
        
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
            }
          } else {
            const counterpart = counterparts.find(c => c.id === firstCounterpartId);
            if (counterpart) {
              setSelectedCounterpart(counterpart);
            }
          }
        }
        
        // Update document groups with loaded entry
        const newGroups = [...documentGroups];
        newGroups[0] = transformedEntry;
        setDocumentGroups(newGroups);
      }
    } catch (err) {
      setError('Грешка при зареждане на записа: ' + err.message);
    }
  };

  // Helper functions
  const isTaxDocument = documentTypes[entry.documentType]?.isTaxDocument || false;

  // Document groups management
  const addDocumentGroup = (type = 'OTHER') => {
    const mainGroup = documentGroups[0];
    const nextId = Math.max(...documentGroups.map(g => g.id)) + 1;
    const groupLabel = groupTypes[type]?.label || 'Нова група';
    
    const newGroup = {
      id: nextId,
      name: `${groupLabel} ${nextId}`,
      type: type,
      documentNumber: generateLinkedDocumentNumber(mainGroup.documentNumber, nextId),
      description: mainGroup.description,
      documentDate: mainGroup.documentDate,
      accountingDate: mainGroup.accountingDate,
      documentType: mainGroup.documentType,
      lines: [
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
          counterpartId: selectedCounterpart?.id || null
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
          counterpartId: selectedCounterpart?.id || null
        }
      ]
    };

    setDocumentGroups(prev => [...prev, newGroup]);
    setActiveGroupIndex(documentGroups.length); // Switch to new group
  };

  const removeDocumentGroup = (groupId) => {
    if (groupId === 0) return; // Cannot remove main group
    
    const newGroups = documentGroups.filter(g => g.id !== groupId);
    setDocumentGroups(newGroups);
    
    // Switch to main group if current group was removed
    if (documentGroups[activeGroupIndex]?.id === groupId) {
      setActiveGroupIndex(0);
    }
  };

  const updateDocumentGroup = (groupId, updates) => {
    setDocumentGroups(prev => prev.map(group => 
      group.id === groupId ? { ...group, ...updates } : group
    ));
  };

  const generateLinkedDocumentNumber = (mainNumber, groupId) => {
    if (!mainNumber) return '';
    return `${mainNumber}-${groupId}`;
  };

  const switchToGroup = (groupIndex) => {
    // Save current entry to current group before switching
    if (documentGroups[activeGroupIndex]) {
      updateDocumentGroup(documentGroups[activeGroupIndex].id, entry);
    }
    setActiveGroupIndex(groupIndex);
  };

  // Format tax document number (10 digits with leading zeros)
  const formatTaxDocumentNumber = (number) => {
    const cleanNumber = String(number).replace(/\D/g, '');
    return cleanNumber.slice(0, 10).padStart(10, '0');
  };

  // Generate document number for non-tax documents
  const generateDocumentNumber = () => {
    if (isTaxDocument) return;
    
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    const segments = [];
    
    for (let i = 0; i < 3; i++) {
      let segment = '';
      const segmentLength = i === 1 ? 4 : 3;
      
      for (let j = 0; j < segmentLength; j++) {
        segment += chars.charAt(Math.floor(Math.random() * chars.length));
      }
      segments.push(segment);
    }
    
    const prefix = documentTypes[entry.documentType]?.code || 'DOC';
    const newNumber = `${prefix}-${segments.join('-')}`;
    
    setEntry(prev => ({
      ...prev,
      documentNumber: newNumber
    }));

    // Update linked groups if this is the main group
    if (activeGroupIndex === 0) {
      documentGroups.forEach((group, index) => {
        if (index > 0) { // Skip main group
          const linkedNumber = generateLinkedDocumentNumber(newNumber, group.id);
          updateDocumentGroup(group.id, { documentNumber: linkedNumber });
        }
      });
    }
  };

  // Validate and format document number
  const handleDocumentNumberChange = (value) => {
    setDocumentNumberError('');
    
    if (isTaxDocument) {
      // Only allow digits for tax documents
      const cleanNumber = value.replace(/\D/g, '');
      if (cleanNumber.length > 10) {
        setDocumentNumberError('Максимум 10 цифри');
        return;
      }
      setEntry(prev => ({ ...prev, documentNumber: cleanNumber }));
    } else {
      setEntry(prev => ({ ...prev, documentNumber: value }));
    }

    // Update linked groups' document numbers
    if (activeGroupIndex === 0) { // Main group changed
      documentGroups.forEach((group, index) => {
        if (index > 0) { // Skip main group
          const linkedNumber = generateLinkedDocumentNumber(value, group.id);
          updateDocumentGroup(group.id, { documentNumber: linkedNumber });
        }
      });
    }
  };

  // Handle document number blur (format tax documents)
  const handleDocumentNumberBlur = () => {
    if (isTaxDocument && entry.documentNumber) {
      const formatted = formatTaxDocumentNumber(entry.documentNumber);
      setEntry(prev => ({ ...prev, documentNumber: formatted }));
    }
  };

  // Fetch exchange rate from BNB
  const fetchExchangeRate = async (currencyCode) => {
    try {
      // Simplified - in real app would call BNB API or backend
      const rates = {
        'EUR': 1.9558,
        'USD': 1.8234,
        'GBP': 2.3567
      };
      return rates[currencyCode] || 1;
    } catch (error) {
      console.error('Error fetching exchange rate:', error);
      return 1;
    }
  };

  // Handle currency change
  const handleCurrencyChange = async (lineIndex, currencyCode) => {
    const newLines = [...entry.lines];
    
    if (currencyCode === 'BGN') {
      newLines[lineIndex].exchangeRate = 1;
    } else {
      const rate = await fetchExchangeRate(currencyCode);
      newLines[lineIndex].exchangeRate = rate;
    }
    
    newLines[lineIndex].currencyCode = currencyCode;
    setEntry(prev => ({ ...prev, lines: newLines }));
  };

  // Open account modal
  const openAccountModal = (lineIndex) => {
    setCurrentLineIndex(lineIndex);
    setShowAccountModal(true);
  };

  // Handle account selection
  const handleAccountSelect = (account) => {
    if (currentLineIndex !== null) {
      const newLines = [...entry.lines];
      newLines[currentLineIndex] = {
        ...newLines[currentLineIndex],
        accountId: account.id,
        accountCode: account.code,
        accountName: account.name,
        // Auto-set default unit for accounts that support quantities
        unitOfMeasure: account.supportsQuantities && account.defaultUnit ? account.defaultUnit : newLines[currentLineIndex].unitOfMeasure
      };
      setEntry(prev => ({
        ...prev,
        lines: newLines
      }));
    }
    setShowAccountModal(false);
    setCurrentLineIndex(null);
  };

  // Add new line
  const addLine = () => {
    setEntry(prev => ({
      ...prev,
      lines: [
        ...prev.lines,
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
          counterpartId: selectedCounterpart?.id || null,
          linkedGroupId: null
        }
      ]
    }));
  };

  // Remove line
  const removeLine = (index) => {
    if (entry.lines.length > 2) {
      const newLines = entry.lines.filter((_, i) => i !== index);
      setEntry(prev => ({
        ...prev,
        lines: newLines
      }));
    }
  };

  // Update line amount
  const updateLineAmount = (index, field, value) => {
    const newLines = [...entry.lines];
    const amount = parseFloat(value) || 0;
    
    if (field === 'debit') {
      newLines[index].debit = amount;
      newLines[index].credit = 0; // Clear credit when debit is entered
    } else {
      newLines[index].credit = amount;
      newLines[index].debit = 0; // Clear debit when credit is entered
    }
    
    // Auto-calculate unit price if quantity is provided
    const quantity = parseFloat(newLines[index].quantity) || 0;
    if (quantity > 0) {
      const totalAmount = newLines[index].debit || newLines[index].credit;
      newLines[index].unitPrice = totalAmount / quantity;
    }
    
    setEntry(prev => ({
      ...prev,
      lines: newLines
    }));
  };

  // Update line quantity
  const updateLineQuantity = (index, value) => {
    const newLines = [...entry.lines];
    const quantity = parseFloat(value) || 0;
    newLines[index].quantity = quantity;
    
    // Auto-calculate unit price if quantity is provided
    if (quantity > 0) {
      const totalAmount = newLines[index].debit || newLines[index].credit;
      newLines[index].unitPrice = totalAmount / quantity;
    } else {
      newLines[index].unitPrice = 0;
    }
    
    setEntry(prev => ({
      ...prev,
      lines: newLines
    }));
  };

  // Create linked group
  const createLinkedGroup = (selectedIndexes) => {
    if (selectedIndexes.length < 2) return;
    
    const groupId = nextGroupId;
    const newLines = [...entry.lines];
    
    selectedIndexes.forEach(index => {
      newLines[index].linkedGroupId = groupId;
    });
    
    const groupName = `Група ${groupId}`;
    setLinkedGroups(prev => [...prev, { id: groupId, name: groupName, lineIndexes: selectedIndexes }]);
    setNextGroupId(prev => prev + 1);
    
    setEntry(prev => ({
      ...prev,
      lines: newLines
    }));
  };

  // Remove linked group
  const removeLinkedGroup = (groupId) => {
    const newLines = [...entry.lines];
    
    newLines.forEach(line => {
      if (line.linkedGroupId === groupId) {
        line.linkedGroupId = null;
      }
    });
    
    setLinkedGroups(prev => prev.filter(group => group.id !== groupId));
    setEntry(prev => ({
      ...prev,
      lines: newLines
    }));
  };

  // Get linked group for line
  const getLinkedGroupForLine = (index) => {
    const line = entry.lines[index];
    if (!line.linkedGroupId) return null;
    return linkedGroups.find(group => group.id === line.linkedGroupId);
  };

  // Toggle line selection for grouping
  const toggleLineSelection = (index) => {
    setSelectedLines(prev => 
      prev.includes(index) 
        ? prev.filter(i => i !== index)
        : [...prev, index]
    );
  };

  // Create group from selected lines
  const handleCreateGroup = () => {
    if (selectedLines.length >= 2) {
      createLinkedGroup(selectedLines);
      setSelectedLines([]);
      setShowGroupingMode(false);
    }
  };

  // Cancel grouping mode
  const handleCancelGrouping = () => {
    setSelectedLines([]);
    setShowGroupingMode(false);
  };

  // Handle counterpart selection
  const handleCounterpartSelect = (counterpart) => {
    setSelectedCounterpart(counterpart);
    setShowCounterpartModal(false);
    
    // Auto-populate counterpart in all lines
    if (counterpart) {
      const newLines = entry.lines.map(line => ({
        ...line,
        counterpartId: counterpart.id
      }));
      setEntry(prev => ({ ...prev, lines: newLines }));
    }
  };

  // Clear counterpart selection
  const handleCounterpartClear = () => {
    setSelectedCounterpart(null);
    const newLines = entry.lines.map(line => ({
      ...line,
      counterpartId: null
    }));
    setEntry(prev => ({ ...prev, lines: newLines }));
  };

  // Create new counterpart
  const handleCreateCounterpart = async (counterpartData) => {
    try {
      const input = {
        ...counterpartData,
        companyId: companyId
      };
      const data = await graphqlRequest(CREATE_COUNTERPART_MUTATION, { input });
      const newCounterpart = data.createCounterpart;
      
      // Add to counterparts list
      setCounterparts(prev => [...prev, newCounterpart]);
      
      // Select the new counterpart
      handleCounterpartSelect(newCounterpart);
      setShowAddCounterpartModal(false);
      
    } catch (err) {
      setError('Грешка при създаване на контрагент: ' + err.message);
    }
  };

  // Calculate totals
  const totals = entry.lines.reduce((acc, line) => ({
    debit: acc.debit + (parseFloat(line.debit) || 0),
    credit: acc.credit + (parseFloat(line.credit) || 0)
  }), { debit: 0, credit: 0 });

  const isBalanced = Math.abs(totals.debit - totals.credit) < 0.01;

  // Save entry - now saves all document groups
  const saveEntry = async () => {
    // Save current entry to its group first
    updateDocumentGroup(documentGroups[activeGroupIndex].id, entry);
    
    // Validate all groups
    let totalDebit = 0;
    let totalCredit = 0;
    const allValidEntries = [];

    for (const group of documentGroups) {
      if (!group.documentNumber || !group.description) {
        setError(`Моля попълнете всички задължителни полета в група "${group.name}"!`);
        return;
      }

      const validLines = group.lines.filter(line => 
        line.accountId && (line.debit > 0 || line.credit > 0)
      );

      if (validLines.length === 0) continue; // Skip empty groups

      const groupTotal = validLines.reduce((acc, line) => ({
        debit: acc.debit + (parseFloat(line.debit) || 0),
        credit: acc.credit + (parseFloat(line.credit) || 0)
      }), { debit: 0, credit: 0 });

      totalDebit += groupTotal.debit;
      totalCredit += groupTotal.credit;

      // Prepare entry for this group
      const entryInput = {
        documentNumber: group.documentNumber,
        description: group.description,
        documentDate: group.documentDate,
        accountingDate: group.accountingDate,
        companyId: companyId,
        lines: validLines.map(line => ({
          accountId: parseInt(line.accountId),
          debitAmount: parseFloat(line.debit) || null,
          creditAmount: parseFloat(line.credit) || null,
          description: line.description || group.description,
          counterpartId: line.counterpartId || null
        }))
      };

      allValidEntries.push(entryInput);
    }

    if (allValidEntries.length === 0) {
      setError('Няма валидни редове за запазване!');
      return;
    }

    // Check overall balance across all groups
    const isOverallBalanced = Math.abs(totalDebit - totalCredit) < 0.01;
    if (!isOverallBalanced) {
      setError(`Общият баланс не е изравнен! Дебит: ${totalDebit.toFixed(2)} лв., Кредит: ${totalCredit.toFixed(2)} лв.`);
      return;
    }

    try {
      if (isEditMode && editingEntryId) {
        // Update existing entry (only support single entry editing for now)
        const entryInput = allValidEntries[0];
        if (entryInput) {
          await graphqlRequest(UPDATE_JOURNAL_ENTRY_MUTATION, { 
            id: editingEntryId, 
            input: {
              documentNumber: entryInput.documentNumber,
              description: entryInput.description,
              documentDate: entryInput.documentDate,
              accountingDate: entryInput.accountingDate,
              lines: entryInput.lines
            }
          });
        }
      } else {
        // Create new entries
        for (const entryInput of allValidEntries) {
          await graphqlRequest(CREATE_JOURNAL_ENTRY_MUTATION, { input: entryInput });
        }
      }
      
      // Reset all groups
      setDocumentGroups([
        {
          id: 0,
          name: 'Основен документ',
          type: 'MAIN',
          documentNumber: '',
          description: '',
          documentDate: new Date().toISOString().split('T')[0],
          accountingDate: new Date().toISOString().split('T')[0],
          documentType: '',
          lines: [
            { 
              accountId: null, accountCode: '', accountName: '', debit: 0, credit: 0, 
              description: '', currencyCode: 'BGN', exchangeRate: 1, quantity: 0, 
              unitOfMeasure: '', unitPrice: 0, counterpartId: null
            },
            { 
              accountId: null, accountCode: '', accountName: '', debit: 0, credit: 0, 
              description: '', currencyCode: 'BGN', exchangeRate: 1, quantity: 0, 
              unitOfMeasure: '', unitPrice: 0, counterpartId: null
            }
          ]
        }
      ]);
      setActiveGroupIndex(0);
      setSelectedCounterpart(null);
      
      setError(null);
      alert(`Всички ${allValidEntries.length} статии са създадени успешно!`);
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
          <h1 className="text-2xl font-bold text-gray-900">Нов журнален запис</h1>
          <p className="mt-1 text-sm text-gray-500">
            Въведете счетоводен запис
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

      {/* Document Groups Tabs */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 overflow-hidden">
        <div className="border-b border-gray-200">
          <nav className="flex">
            {documentGroups.map((group, index) => {
              const groupType = groupTypes[group.type] || groupTypes['OTHER'];
              const isActive = index === activeGroupIndex;
              return (
                <button
                  key={group.id}
                  onClick={() => switchToGroup(index)}
                  className={`relative px-4 py-3 text-sm font-medium border-r border-gray-200 flex items-center space-x-2 ${
                    isActive 
                      ? `text-${groupType.color}-600 bg-${groupType.color}-50 border-b-2 border-${groupType.color}-600` 
                      : 'text-gray-500 hover:text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  <div className={`w-2 h-2 rounded-full bg-${groupType.color}-400`}></div>
                  <span>{group.name}</span>
                  {group.documentNumber && (
                    <span className="text-xs text-gray-400 font-mono">
                      ({group.documentNumber})
                    </span>
                  )}
                  {group.id !== 0 && (
                    <span
                      onClick={(e) => {
                        e.stopPropagation();
                        removeDocumentGroup(group.id);
                      }}
                      className="ml-2 text-red-400 hover:text-red-600 cursor-pointer"
                    >
                      ×
                    </span>
                  )}
                </button>
              );
            })}
            
            {/* Add Group Button */}
            <div className="relative">
              <button
                onClick={() => setShowGroupMenu(prev => !prev)}
                className="px-4 py-3 text-sm font-medium text-gray-500 hover:text-gray-700 hover:bg-gray-50 flex items-center space-x-2"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
                </svg>
                <span>Добави група</span>
              </button>
              
              {showGroupMenu && (
                <div className="absolute top-full left-0 mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-10 min-w-48">
                  {Object.entries(groupTypes).map(([key, type]) => {
                    if (key === 'MAIN') return null; // Don't show main in menu
                    return (
                      <button
                        key={key}
                        onClick={() => {
                          addDocumentGroup(key);
                          setShowGroupMenu(false);
                        }}
                        className="w-full px-4 py-2 text-left text-sm hover:bg-gray-50 flex items-center space-x-2"
                      >
                        <div className={`w-2 h-2 rounded-full bg-${type.color}-400`}></div>
                        <span>{type.label}</span>
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          </nav>
        </div>

        {/* Current Group Summary */}
        <div className="px-4 py-2 bg-gray-50 text-xs text-gray-600">
          <span className="font-medium">{documentGroups[activeGroupIndex]?.name}:</span>
          <span className="ml-2">
            {entry.lines?.filter(line => line.debit > 0 || line.credit > 0).length || 0} активни реда
          </span>
          {entry.documentNumber && (
            <span className="ml-4 font-mono">№ {entry.documentNumber}</span>
          )}
        </div>
      </div>

      {/* Basic Info */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Основна информация</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Вид документ *
            </label>
            <select
              value={entry.documentType}
              onChange={(e) => setEntry({ ...entry, documentType: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              required
            >
              <option value="">Избери вид...</option>
              {Object.entries(documentTypes).map(([key, type]) => (
                <option key={key} value={key}>
                  {type.label} ({type.code})
                </option>
              ))}
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Номер на документ *
            </label>
            <div className="flex space-x-2">
              <input
                type="text"
                value={entry.documentNumber}
                onChange={(e) => handleDocumentNumberChange(e.target.value)}
                onBlur={handleDocumentNumberBlur}
                className={`flex-1 px-3 py-2 border rounded-md text-sm ${
                  isTaxDocument ? 'font-mono' : ''
                } ${documentNumberError ? 'border-red-300' : 'border-gray-300'}`}
                placeholder={isTaxDocument ? "Въведи номер (напр. 354)" : "Номер на документа"}
                maxLength={isTaxDocument ? 10 : 20}
                required
              />
              {!isTaxDocument && (
                <button
                  onClick={generateDocumentNumber}
                  disabled={!entry.documentType}
                  className="px-3 py-2 bg-gray-100 text-gray-700 rounded-md text-sm hover:bg-gray-200 disabled:bg-gray-50 disabled:text-gray-400"
                >
                  Генерирай
                </button>
              )}
            </div>
            {documentNumberError && (
              <p className="text-red-600 text-xs mt-1">{documentNumberError}</p>
            )}
            {isTaxDocument && !documentNumberError && (
              <p className="text-gray-500 text-xs mt-1">
                Номерът ще бъде форматиран с водещи нули (10 цифри)
              </p>
            )}
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Описание *
            </label>
            <input
              type="text"
              value={entry.description}
              onChange={(e) => setEntry({ ...entry, description: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              placeholder="Описание на операцията"
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Дата на документ *
            </label>
            <input
              type="date"
              value={entry.documentDate}
              onChange={(e) => setEntry({ ...entry, documentDate: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Счетоводна дата *
            </label>
            <input
              type="date"
              value={entry.accountingDate}
              onChange={(e) => setEntry({ ...entry, accountingDate: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              required
            />
          </div>
        </div>
      </div>

      {/* Counterpart Selection */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Контрагент</h3>
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
                  {(selectedCounterpart.street || selectedCounterpart.address) && (
                    <div className="text-sm text-gray-500">
                      {selectedCounterpart.street || selectedCounterpart.address}
                      {selectedCounterpart.postalCode && `, ${selectedCounterpart.postalCode}`}
                      {selectedCounterpart.city && `, ${selectedCounterpart.city}`}
                    </div>
                  )}
                </div>
                <button
                  onClick={handleCounterpartClear}
                  className="text-red-600 hover:text-red-800 ml-2"
                  title="Премахни контрагент"
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
          
          <div className="flex space-x-2">
            {!selectedCounterpart && (
              <button
                onClick={() => setShowCounterpartModal(true)}
                className="inline-flex items-center px-3 py-2 border border-transparent text-sm font-medium rounded-md text-blue-600 bg-blue-50 hover:bg-blue-100"
              >
                <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
                Търси
              </button>
            )}
            <button
              onClick={() => setShowAddCounterpartModal(true)}
              className="inline-flex items-center px-3 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700"
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
              </svg>
              Нов
            </button>
          </div>
        </div>
      </div>

      {/* Entry Lines */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <h3 className="text-lg font-semibold text-gray-900">Редове на записа</h3>
              
              {/* Linked Groups Display */}
              {linkedGroups.length > 0 && (
                <div className="flex items-center space-x-2">
                  {linkedGroups.map(group => (
                    <div key={group.id} className="flex items-center bg-blue-100 text-blue-800 px-2 py-1 rounded-md text-sm">
                      <span className="mr-2">{group.name} ({group.lineIndexes.length} реда)</span>
                      <button
                        onClick={() => removeLinkedGroup(group.id)}
                        className="text-blue-600 hover:text-blue-800"
                      >
                        ×
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
            
            <div className="flex items-center space-x-2">
              {showGroupingMode ? (
                <>
                  <span className="text-sm text-gray-600">
                    Избрани: {selectedLines.length} реда
                  </span>
                  <button
                    onClick={handleCreateGroup}
                    disabled={selectedLines.length < 2}
                    className="inline-flex items-center px-3 py-1.5 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:bg-gray-400"
                  >
                    Групирай
                  </button>
                  <button
                    onClick={handleCancelGrouping}
                    className="inline-flex items-center px-3 py-1.5 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                  >
                    Отказ
                  </button>
                </>
              ) : (
                <>
                  <button
                    onClick={() => setShowGroupingMode(true)}
                    className="inline-flex items-center px-3 py-1.5 border border-transparent text-sm font-medium rounded-md text-blue-600 bg-blue-50 hover:bg-blue-100"
                  >
                    <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                    </svg>
                    Свържи редове
                  </button>
                  <button
                    onClick={addLine}
                    className="inline-flex items-center px-3 py-1.5 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
                  >
                    <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
                    </svg>
                    Добави ред
                  </button>
                </>
              )}
            </div>
          </div>
        </div>
        
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                {showGroupingMode && (
                  <th className="px-3 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-12">
                    Избор
                  </th>
                )}
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
              {entry.lines.map((line, index) => {
                const linkedGroup = getLinkedGroupForLine(index);
                const isInGroup = linkedGroup !== null;
                const isSelected = selectedLines.includes(index);
                
                return (
                  <tr key={index} className={isInGroup ? 'bg-blue-25 border-l-4 border-blue-300' : ''}>
                    {/* Selection Checkbox */}
                    {showGroupingMode && (
                      <td className="px-3 py-2 text-center">
                        <input
                          type="checkbox"
                          checked={isSelected}
                          onChange={() => toggleLineSelection(index)}
                          disabled={isInGroup}
                          className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                        />
                      </td>
                    )}
                    
                    {/* Сметка */}
                    <td className="px-3 py-2 relative">
                      {/* Group indicator */}
                      {isInGroup && (
                        <div className="absolute -left-1 top-0 bottom-0 w-1 bg-blue-400 rounded-r"></div>
                      )}
                      
                      <button
                        onClick={() => openAccountModal(index)}
                        className="w-full text-left px-2 py-1 border border-gray-300 rounded text-xs hover:bg-gray-50"
                      >
                        {line.accountCode ? (
                          <span>
                            <span className="font-mono font-medium">{line.accountCode}</span>
                            <br />
                            <span className="text-gray-600">{line.accountName}</span>
                            {isInGroup && (
                              <>
                                <br />
                                <span className="text-blue-600 text-xs">{linkedGroup.name}</span>
                              </>
                            )}
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
                    <div className="flex items-center space-x-1">
                      <input
                        type="number"
                        value={line.credit || ''}
                        onChange={(e) => updateLineAmount(index, 'credit', e.target.value)}
                        className="flex-1 px-2 py-1 border border-gray-300 rounded text-xs text-right"
                        placeholder="0.00"
                        step="0.01"
                        min="0"
                      />
                      {/* Show average cost button for material accounts (class 3 or supports quantities) */}
                      {line.accountId && (() => {
                        const account = accounts.find(a => a.id === line.accountId);
                        const isMaterialAccount = account && (
                          account.accountClass === 3 ||
                          account.supportsQuantities ||
                          (account.code && account.code.startsWith('3'))
                        );
                        return isMaterialAccount && line.quantity > 0 ? (
                          <AverageCostButton
                            accountId={line.accountId}
                            quantity={line.quantity}
                            onCalculated={(calculatedValue) => {
                              const newLines = [...entry.lines];
                              newLines[index].credit = parseFloat(calculatedValue);
                              newLines[index].debit = 0;
                              setEntry(prev => ({ ...prev, lines: newLines }));
                            }}
                          />
                        ) : null;
                      })()}
                    </div>
                  </td>

                  {/* Валута */}
                  <td className="px-3 py-2">
                    <select
                      value={line.currencyCode}
                      onChange={(e) => handleCurrencyChange(index, e.target.value)}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs"
                    >
                      <option value="BGN">BGN</option>
                      {currencies.map(currency => (
                        <option key={currency.code} value={currency.code}>
                          {currency.code}
                        </option>
                      ))}
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
                      onChange={(e) => updateLineQuantity(index, parseFloat(e.target.value) || 0)}
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
                      onChange={(e) => {
                        const newLines = [...entry.lines];
                        newLines[index].unitOfMeasure = e.target.value;
                        setEntry(prev => ({ ...prev, lines: newLines }));
                      }}
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
                      readOnly
                      className="w-full px-2 py-1 border border-gray-300 rounded text-xs text-right bg-gray-100"
                      placeholder="0.00"
                      step="0.01"
                      min="0"
                    />
                  </td>

                  {/* Действия */}
                  <td className="px-3 py-2 text-center">
                    {entry.lines.length > 2 && (
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
                );
              })}
              
              {/* Totals Row */}
              <tr className="bg-gray-50 font-semibold">
                {showGroupingMode && <td></td>}
                <td className="px-3 py-3 text-right">
                  Общо:
                </td>
                <td></td>
                <td className="px-3 py-3 text-right">
                  {totals.debit.toFixed(2)} лв.
                </td>
                <td className="px-3 py-3 text-right">
                  {totals.credit.toFixed(2)} лв.
                </td>
                <td colSpan="5"></td>
              </tr>
              
              {/* Balance Status */}
              <tr>
                <td colSpan={showGroupingMode ? "11" : "10"} className="px-6 py-3">
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
          disabled={!isBalanced}
          className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          Запази
        </button>
      </div>

      {/* Account Select Modal */}
      <AccountSelectModal
        show={showAccountModal}
        accounts={accounts}
        currentAccountId={entry.lines[currentLineIndex]?.accountId}
        onSelect={handleAccountSelect}
        onClose={() => setShowAccountModal(false)}
      />

      {/* Counterpart Select Modal */}
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

      {/* Add Counterpart Modal */}
      <AddCounterpartModal
        show={showAddCounterpartModal}
        onSave={handleCreateCounterpart}
        onClose={() => setShowAddCounterpartModal(false)}
      />
    </div>
  );
}
