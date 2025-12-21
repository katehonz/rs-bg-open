import { useState, useEffect, useRef, useCallback } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

const DOCUMENT_TYPE_LABELS = {
  bank_statement: 'Банково извлечение',
  sales_invoice: 'Фактура за продажба',
  purchase_invoice: 'Фактура за покупка',
  protocol: 'Протокол',
  cash_receipt: 'Приходен касов ордер',
  cash_payment: 'Разходен касов ордер',
  customs_declaration: 'Митническа декларация',
  vat_document: 'ДДС документ',
  vat_return: 'ДДС декларация',
  journal: 'Журнален запис',
};

const DOCUMENT_TYPE_OPTIONS = [
  { value: 'all', label: 'Всички документи' },
  { value: 'bank_statement', label: DOCUMENT_TYPE_LABELS.bank_statement },
  { value: 'sales_invoice', label: DOCUMENT_TYPE_LABELS.sales_invoice },
  { value: 'purchase_invoice', label: DOCUMENT_TYPE_LABELS.purchase_invoice },
  { value: 'protocol', label: DOCUMENT_TYPE_LABELS.protocol },
  { value: 'cash_receipt', label: DOCUMENT_TYPE_LABELS.cash_receipt },
  { value: 'cash_payment', label: DOCUMENT_TYPE_LABELS.cash_payment },
  { value: 'customs_declaration', label: DOCUMENT_TYPE_LABELS.customs_declaration },
  { value: 'vat_document', label: DOCUMENT_TYPE_LABELS.vat_document },
  { value: 'vat_return', label: DOCUMENT_TYPE_LABELS.vat_return },
  { value: 'journal', label: DOCUMENT_TYPE_LABELS.journal },
];

const getDocumentCategory = (entry) => {
  if (!entry) {
    return 'journal';
  }

  if (entry.isVatReturn) {
    return 'vat_return';
  }

  const entryNumber = entry.entryNumber || '';
  const description = (entry.description || '').toLowerCase();

  if (entryNumber.startsWith('BANK-') || description.includes('банково извлечение')) {
    return 'bank_statement';
  }

  if (entry.vatDocumentType) {
    switch (entry.vatDocumentType) {
      case '01':
        return 'sales_invoice';
      case '02':
        return 'protocol';
      case '03':
        return 'purchase_invoice';
      case '04':
        return 'cash_receipt';
      case '05':
        return 'cash_payment';
      case '07':
        return 'customs_declaration';
      default:
        return 'vat_document';
    }
  }

  if (entryNumber.startsWith('CTRL-')) {
    if (description.includes('продаж') || description.includes('издаден')) {
      return 'sales_invoice';
    }
    if (description.includes('покуп') || description.includes('достав')) {
      return 'purchase_invoice';
    }
    return 'vat_document';
  }

  if (description.includes('фактура')) {
    return description.includes('покуп') ? 'purchase_invoice' : 'sales_invoice';
  }

  return 'journal';
};

const getDocumentTypeLabelFromEntry = (entry) => {
  const category = getDocumentCategory(entry);
  return DOCUMENT_TYPE_LABELS[category] || DOCUMENT_TYPE_LABELS.journal;
};

const PAGE_SIZE = 50;

export default function JournalEntriesList() {
  const [entries, setEntries] = useState([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [error, setError] = useState(null);
  const [selectedEntries, setSelectedEntries] = useState(new Set());
  const [offset, setOffset] = useState(0);
  const [filters, setFilters] = useState({
    startDate: '',
    endDate: '',
    searchQuery: '',
    status: 'all', // all, draft, posted
    documentType: 'all',
  });

  const tableContainerRef = useRef(null);

  const JOURNAL_ENTRIES_QUERY = `
    query GetJournalEntries($filter: JournalEntryFilter, $limit: Int, $offset: Int) {
      journalEntries(filter: $filter, limit: $limit, offset: $offset) {
        id
        entryNumber
        documentDate
        documentNumber
        description
        totalAmount
        isPosted
        createdAt
        vatDocumentType
      }
    }
  `;

  const VAT_RETURNS_QUERY = `
    query GetVatReturns($filter: VatReturnFilter, $limit: Int, $offset: Int) {
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
        status
        dueDate
        notes
        createdAt
        updatedAt
      }
    }
  `;

  const DELETE_JOURNAL_ENTRY_MUTATION = `
    mutation DeleteJournalEntry($id: Int!) {
      deleteJournalEntry(id: $id)
    }
  `;

  const DELETE_JOURNAL_ENTRIES_MUTATION = `
    mutation DeleteJournalEntries($ids: [Int!]!) {
      deleteJournalEntries(ids: $ids)
    }
  `;

  const UNPOST_JOURNAL_ENTRY_MUTATION = `
    mutation UnpostJournalEntry($id: Int!) {
      unpostJournalEntry(id: $id) {
        id
        isPosted
      }
    }
  `;

  const UNPOST_JOURNAL_ENTRIES_MUTATION = `
    mutation UnpostJournalEntries($ids: [Int!]!) {
      unpostJournalEntries(ids: $ids)
    }
  `;

  const POST_JOURNAL_ENTRY_MUTATION = `
    mutation PostJournalEntry($id: Int!) {
      postJournalEntry(id: $id) {
        id
        isPosted
      }
    }
  `;

  const loadEntries = useCallback(async (reset = false) => {
    if (reset) {
      setLoading(true);
      setLoadingMore(false);
      setError(null);
      setHasMore(true);
      setOffset(0);
    } else {
      if (loading || loadingMore || !hasMore) {
        return;
      }
      setLoadingMore(true);
      setError(null);
    }

    try {
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      const currentOffset = reset ? 0 : offset;

      // Load journal entries
      const journalFilter = {
        companyId: companyId
      };

      if (filters.startDate) journalFilter.fromDate = filters.startDate;
      if (filters.endDate) journalFilter.toDate = filters.endDate;
      if (filters.searchQuery) journalFilter.documentNumber = filters.searchQuery;
      if (filters.status === 'draft') journalFilter.isPosted = false;
      if (filters.status === 'posted') journalFilter.isPosted = true;

      const journalVariables = {
        filter: journalFilter,
        limit: PAGE_SIZE,
        offset: currentOffset
      };

      // Load VAT returns only on first page
      let vatEntries = [];
      if (reset || currentOffset === 0) {
        const vatFilter = {
          companyId: companyId
        };

        const vatVariables = {
          filter: vatFilter,
          limit: 100,
          offset: 0
        };

        const [journalData, vatData] = await Promise.all([
          graphqlRequest(JOURNAL_ENTRIES_QUERY, journalVariables),
          graphqlRequest(VAT_RETURNS_QUERY, vatVariables)
        ]);

        const journalEntries = journalData.journalEntries || [];
        const vatReturns = vatData.vatReturns || [];

        // Convert VAT returns to entry-like format for uniform display
        vatEntries = vatReturns.map(vat => ({
          id: `vat-${vat.id}`,
          entryNumber: `ДДС-${vat.periodYear}-${String(vat.periodMonth).padStart(2, '0')}`,
          documentDate: vat.periodTo,
          documentNumber: `ДДС декларация ${vat.periodMonth}/${vat.periodYear}`,
          description: `ДДС декларация за ${getMonthName(vat.periodMonth)} ${vat.periodYear}`,
          totalAmount: vat.vatToPay || (vat.outputVatAmount - vat.inputVatAmount),
          isPosted: vat.status === 'SUBMITTED' || vat.status === 'APPROVED',
          createdAt: vat.createdAt,
          vatDocumentType: 'VAT_RETURN',
          isVatReturn: true,
          vatReturn: vat // Store original VAT return data
        }));

        // Combine and sort by date
        const allEntries = [...journalEntries, ...vatEntries].sort((a, b) =>
          new Date(b.documentDate) - new Date(a.documentDate)
        );

        setEntries(allEntries);
        setHasMore(journalEntries.length === PAGE_SIZE);
        setOffset(journalEntries.length);
      } else {
        // Load more journal entries
        const journalData = await graphqlRequest(JOURNAL_ENTRIES_QUERY, journalVariables);
        const journalEntries = journalData.journalEntries || [];

        setEntries(prev => {
          const combined = [...prev, ...journalEntries];
          return combined.sort((a, b) => new Date(b.documentDate) - new Date(a.documentDate));
        });
        setHasMore(journalEntries.length === PAGE_SIZE);
        setOffset(prev => prev + journalEntries.length);
      }
    } catch (err) {
      setError('Грешка при зареждане на журналните записи: ' + err.message);
      if (reset) {
        setEntries([]);
        setHasMore(false);
      }
    } finally {
      if (reset) {
        setLoading(false);
      } else {
        setLoadingMore(false);
      }
    }
  }, [filters, loading, loadingMore, hasMore, offset]);

  useEffect(() => {
    loadEntries(true);
  }, []);

  useEffect(() => {
    setSelectedEntries(new Set());
  }, [filters.documentType]);

  const handleScroll = useCallback((event) => {
    if (loading || loadingMore || !hasMore) {
      return;
    }

    const { scrollTop, scrollHeight, clientHeight } = event.target;
    if (scrollHeight - scrollTop - clientHeight < 100) {
      loadEntries(false);
    }
  }, [loadEntries, hasMore, loading, loadingMore]);

  const getMonthName = (month) => {
    const monthNames = [
      'Януари', 'Февруари', 'Март', 'Април', 'Май', 'Юни',
      'Юли', 'Август', 'Септември', 'Октомври', 'Ноември', 'Декември'
    ];
    return monthNames[month - 1] || 'Неизвестен';
  };

  const handleDelete = async (entryId) => {
    if (!confirm('Сигурни ли сте, че искате да изтриете този журнален запис?')) {
      return;
    }

    try {
      await graphqlRequest(DELETE_JOURNAL_ENTRY_MUTATION, { id: entryId });
      await loadEntries(true); // Reload the list
      alert('Журналният запис е изтрит успешно!');
    } catch (err) {
      alert('Грешка при изтриването: ' + err.message);
    }
  };

  const handlePost = async (entryId) => {
    if (!confirm('Сигурни ли сте, че искате да приключите този журнален запис?')) {
      return;
    }

    try {
      await graphqlRequest(POST_JOURNAL_ENTRY_MUTATION, { id: entryId });
      await loadEntries(true); // Reload the list
      alert('Журналният запис е приключен успешно!');
    } catch (err) {
      alert('Грешка при приключването: ' + err.message);
    }
  };

  const handleUnpost = async (entryId) => {
    if (!confirm('Сигурни ли сте, че искате да върнете този журнален запис към чернова?')) {
      return;
    }

    try {
      await graphqlRequest(UNPOST_JOURNAL_ENTRY_MUTATION, { id: entryId });
      await loadEntries(true); // Reload the list
      alert('Журналният запис е върнат към чернова успешно!');
    } catch (err) {
      alert('Грешка при връщане към чернова: ' + err.message);
    }
  };

  const handleBulkPost = async () => {
    if (selectedEntries.size === 0) {
      alert('Моля изберете записи за приключване.');
      return;
    }

    if (!confirm(`Сигурни ли сте, че искате да приключите ${selectedEntries.size} записа?`)) {
      return;
    }

    try {
      const promises = Array.from(selectedEntries).map(id => 
        graphqlRequest(POST_JOURNAL_ENTRY_MUTATION, { id: parseInt(id) })
      );
      
      await Promise.all(promises);
      await loadEntries(true); // Reload the list
      setSelectedEntries(new Set()); // Clear selection
      alert(`Успешно приключени ${promises.length} записа!`);
    } catch (err) {
      alert('Грешка при приключването: ' + err.message);
    }
  };

  const handleBulkUnpost = async () => {
    if (selectedEntries.size === 0) {
      alert('Моля изберете записи за връщане към чернова.');
      return;
    }

    if (!confirm(`Сигурни ли сте, че искате да върнете ${selectedEntries.size} записа към чернова?`)) {
      return;
    }

    try {
      const ids = Array.from(selectedEntries).map(id => parseInt(id));
      const result = await graphqlRequest(UNPOST_JOURNAL_ENTRIES_MUTATION, { ids });

      await loadEntries(true); // Reload the list
      setSelectedEntries(new Set()); // Clear selection
      alert(`Успешно върнати ${result.unpostJournalEntries} записа към чернова!`);
    } catch (err) {
      alert('Грешка при връщането към чернова: ' + err.message);
    }
  };

  const handleBulkDelete = async () => {
    if (selectedEntries.size === 0) {
      alert('Моля изберете записи за изтриване.');
      return;
    }

    if (!confirm(`Сигурни ли сте, че искате да изтриете ${selectedEntries.size} записа? Това действие не може да бъде отменено!`)) {
      return;
    }

    try {
      const ids = Array.from(selectedEntries).map(id => parseInt(id));
      const result = await graphqlRequest(DELETE_JOURNAL_ENTRIES_MUTATION, { ids });

      await loadEntries(true); // Reload the list
      setSelectedEntries(new Set()); // Clear selection
      alert(`Успешно изтрити ${result.deleteJournalEntries} записа!`);
    } catch (err) {
      alert('Грешка при изтриването: ' + err.message);
    }
  };

  const handleSelectEntry = (entryId, checked) => {
    const newSelected = new Set(selectedEntries);
    if (checked) {
      newSelected.add(entryId);
    } else {
      newSelected.delete(entryId);
    }
    setSelectedEntries(newSelected);
  };

  const handleSelectAll = (checked, onlyDrafts = false) => {
    if (checked) {
      let entriesToSelect;
      if (onlyDrafts) {
        // Select only draft entries
        entriesToSelect = filteredEntries.filter(entry => !entry.isPosted && !entry.isVatReturn);
      } else {
        // Select all non-VAT return entries
        entriesToSelect = filteredEntries.filter(entry => !entry.isVatReturn);
      }
      const ids = entriesToSelect.map(entry => entry.id);
      setSelectedEntries(new Set(ids));
    } else {
      setSelectedEntries(new Set());
    }
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      style: 'currency',
      currency: 'BGN'
    }).format(amount || 0);
  };

  const formatDate = (dateString) => {
    if (!dateString) return '-';
    return new Date(dateString).toLocaleDateString('bg-BG');
  };

  const getDocumentTypeLabel = (entry) => getDocumentTypeLabelFromEntry(entry);

  const isVATEntry = (entry) => {
    // Entry is VAT type if it has vatDocumentType or is a VAT return
    return entry.isVatReturn || (entry.vatDocumentType && entry.vatDocumentType !== '');
  };

  const getEditUrl = (entry) => {
    if (entry.isVatReturn) {
      // VAT return entries go to VAT returns page
      return `/vat/returns`;
    } else if (isVATEntry(entry)) {
      // VAT entries go to VAT entry form
      return `/accounting/vat-entry?edit=${entry.id}`;
    } else {
      // Regular entries go to regular entry form
      return `/accounting/entries?edit=${entry.id}`;
    }
  };

  const getPostStatus = (isPosted) => {
    return isPosted ? (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
        Приключен
      </span>
    ) : (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
        Чернова
      </span>
    );
  };

  const filteredEntries = entries.filter(entry => {
    if (filters.documentType !== 'all' && getDocumentCategory(entry) !== filters.documentType) {
      return false;
    }

    if (filters.searchQuery) {
      const query = filters.searchQuery.toLowerCase();
      const matchesNumber = entry.documentNumber?.toLowerCase().includes(query);
      const matchesDescription = entry.description?.toLowerCase().includes(query);
      return matchesNumber || matchesDescription;
    }
    return true;
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <h2 className="text-lg font-semibold text-red-800">Грешка</h2>
          <p className="text-red-600 mt-2">{error}</p>
          <button
            onClick={() => loadEntries(true)}
            className="mt-4 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
          >
            Опитай отново
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Журнални записи</h1>
            <p className="mt-1 text-sm text-gray-500">
              Преглед, редактиране и управление на журнални записи
            </p>
          </div>
          <a 
            href="/accounting/entries"
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
          >
            <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4" />
            </svg>
            Нов запис
          </a>
        </div>

        {/* Filters */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mb-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              От дата
            </label>
            <input
              type="date"
              value={filters.startDate}
              onChange={(e) => setFilters({...filters, startDate: e.target.value})}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              До дата
            </label>
            <input
              type="date"
              value={filters.endDate}
              onChange={(e) => setFilters({...filters, endDate: e.target.value})}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Статус
            </label>
            <select
              value={filters.status}
              onChange={(e) => setFilters({...filters, status: e.target.value})}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="all">Всички</option>
              <option value="draft">Чернови</option>
              <option value="posted">Приключени</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Вид документ
            </label>
            <select
              value={filters.documentType}
              onChange={(e) => setFilters({ ...filters, documentType: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              {DOCUMENT_TYPE_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
          <div className="md:col-span-2 lg:col-span-1 lg:col-start-auto">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Търсене
            </label>
            <input
              type="text"
              placeholder="Номер документ, описание..."
              value={filters.searchQuery}
              onChange={(e) => setFilters({...filters, searchQuery: e.target.value})}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>
        </div>

        <div className="flex items-center gap-4">
          <button
            onClick={() => loadEntries(true)}
            className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 text-sm"
          >
            Приложи филтри
          </button>
          
          {selectedEntries.size > 0 && (
            <>
              <button
                onClick={handleBulkPost}
                className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 text-sm"
              >
                Приключи избраните ({selectedEntries.size})
              </button>
              <button
                onClick={handleBulkUnpost}
                className="px-4 py-2 bg-orange-600 text-white rounded hover:bg-orange-700 text-sm ml-2"
              >
                Чернова ({selectedEntries.size})
              </button>
              <button
                onClick={handleBulkDelete}
                className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 text-sm ml-2"
              >
                Изтрий избраните ({selectedEntries.size})
              </button>
            </>
          )}
        </div>
      </div>

      {/* Journal Entries List */}
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-medium text-gray-900">
            Намерени {filteredEntries.length} записа
          </h3>
        </div>

        {filteredEntries.length === 0 ? (
          <div className="p-6 text-center">
            <p className="text-gray-500">Няма намерени журнални записи.</p>
          </div>
        ) : (
          <div
            ref={tableContainerRef}
            onScroll={handleScroll}
            className="overflow-x-auto max-h-[calc(100vh-400px)]"
          >
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    <input
                      type="checkbox"
                      onChange={(e) => handleSelectAll(e.target.checked)}
                      checked={selectedEntries.size > 0 && selectedEntries.size === filteredEntries.filter(e => !e.isVatReturn).length}
                      className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    />
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Дата / Документ
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Описание
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Сума
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Статус
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Действия
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {filteredEntries.map((entry) => (
                  <tr key={entry.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      {!entry.isVatReturn && (
                        <input
                          type="checkbox"
                          checked={selectedEntries.has(entry.id)}
                          onChange={(e) => handleSelectEntry(entry.id, e.target.checked)}
                          className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                        />
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-gray-900">
                        {formatDate(entry.documentDate)}
                      </div>
                      <div className="text-sm text-gray-500">
                        {entry.isVatReturn ? (
                          <>
                            <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 mr-2">
                              Глава III
                            </span>
                            ДДС декларация
                          </>
                        ) : entry.vatDocumentType ? (
                          <>
                            <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 mr-2">
                              ДДС
                            </span>
                            {getDocumentTypeLabel(entry)}
                          </>
                        ) : entry.entryNumber && entry.entryNumber.startsWith('CTRL-') ? (
                          <>
                            <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 mr-2">
                              Controlisy
                            </span>
                            {getDocumentTypeLabel(entry)}
                          </>
                        ) : (
                          getDocumentTypeLabel(entry)
                        )}
                        {entry.documentNumber && ` № ${entry.documentNumber}`}
                      </div>
                      <div className="text-xs text-gray-400">
                        Запис № {entry.entryNumber}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-gray-900 max-w-xs truncate">
                        {entry.description || '-'}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">
                        Създаден: {formatDate(entry.createdAt)}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-gray-900">
                        {formatCurrency(entry.totalAmount)}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {getPostStatus(entry.isPosted)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                      <div className="flex items-center space-x-2">
                        <a
                          href={getEditUrl(entry)}
                          className="text-blue-600 hover:text-blue-900"
                        >
                          {entry.isVatReturn ? 'Преглед' : 'Редактирай'}
                        </a>
                        {!entry.isPosted && !entry.isVatReturn && (
                          <button
                            onClick={() => handlePost(entry.id)}
                            className="text-green-600 hover:text-green-900"
                          >
                            Приключи
                          </button>
                        )}
                        {entry.isPosted && !entry.isVatReturn && (
                          <button
                            onClick={() => handleUnpost(entry.id)}
                            className="text-orange-600 hover:text-orange-900"
                          >
                            Чернова
                          </button>
                        )}
                        {!entry.isVatReturn && (
                          <button
                            onClick={() => handleDelete(entry.id)}
                            className="text-red-600 hover:text-red-900"
                          >
                            Изтрий
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {loadingMore && (
              <div className="p-4 text-center border-t border-gray-200">
                <div className="animate-spin h-6 w-6 border-2 border-blue-500 border-t-transparent rounded-full mx-auto"></div>
                <p className="text-sm text-gray-500 mt-2">Зареждане на още записи...</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
