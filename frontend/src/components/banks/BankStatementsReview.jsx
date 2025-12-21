import { useState, useEffect, useMemo, useRef, useCallback } from 'react';
import { graphqlRequest } from '../../utils/graphqlClient';

const BANK_IMPORTS_QUERY = `
  query BankImports(
    $companyId: Int!
    $bankProfileId: Int
    $limit: Int
    $offset: Int
    $fromDate: NaiveDate
    $toDate: NaiveDate
  ) {
    bankImports(
      companyId: $companyId
      bankProfileId: $bankProfileId
      limit: $limit
      offset: $offset
      fromDate: $fromDate
      toDate: $toDate
    ) {
      id
      fileName
      importFormat
      importedAt
      transactionsCount
      totalCredit
      totalDebit
      journalEntryIds
    }
  }
`;

const JOURNAL_ENTRY_WITH_LINES_QUERY = `
  query JournalEntryWithLines($id: Int!) {
    journalEntryWithLines(id: $id) {
      id
      entryNumber
      documentNumber
      documentDate
      accountingDate
      vatDate
      description
      totalAmount
      totalVatAmount
      isPosted
      createdAt
      updatedAt
      lines {
        id
        accountId
        debitAmount
        creditAmount
        currencyCode
        currencyAmount
        description
        lineOrder
        counterpartId
      }
    }
  }
`;

const COUNTERPARTS_QUERY = `
  query Counterparts($companyId: Int!) {
    counterparts(companyId: $companyId) {
      id
      name
      eik
      vatNumber
      isActive
    }
  }
`;

const UPDATE_JOURNAL_ENTRY_MUTATION = `
  mutation UpdateJournalEntry($id: Int!, $input: UpdateJournalEntryInput!) {
    updateJournalEntry(id: $id, input: $input) {
      id
    }
  }
`;

const IMPORT_FORMAT_LABELS = {
  UNICREDIT_MT940: 'UniCredit MT940',
  WISE_CAMT053: 'Wise CAMT.053',
  REVOLUT_CAMT053: 'Revolut CAMT.053',
  PAYSERA_CAMT053: 'Paysera CAMT.053',
  POSTBANK_XML: 'Postbank XML',
  OBB_XML: 'OBB XML',
  CCB_CSV: 'ЦКБ CSV',
};

const IMPORTS_PAGE_SIZE = 25;

const formatCurrency = (value) => {
  if (value === null || value === undefined) {
    return '—';
  }
  const numeric = typeof value === 'number' ? value : parseFloat(value);
  if (Number.isNaN(numeric)) {
    return '—';
  }
  return new Intl.NumberFormat('bg-BG', {
    style: 'currency',
    currency: 'BGN',
    minimumFractionDigits: 2,
  }).format(numeric);
};

const formatDate = (value) => {
  if (!value) {
    return '—';
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return '—';
  }
  return date.toLocaleDateString('bg-BG');
};

const formatDateTime = (value) => {
  if (!value) {
    return '—';
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return '—';
  }
  return `${date.toLocaleDateString('bg-BG')} ${date.toLocaleTimeString('bg-BG')}`;
};

const parseJournalEntryIds = (rawValue) => {
  if (!rawValue) {
    return [];
  }

  if (Array.isArray(rawValue)) {
    return rawValue.map((id) => Number(id)).filter((id) => !Number.isNaN(id));
  }

  if (typeof rawValue === 'string') {
    try {
      const parsed = JSON.parse(rawValue);
      if (Array.isArray(parsed)) {
        return parsed.map((id) => Number(id)).filter((id) => !Number.isNaN(id));
      }
    } catch (err) {
      return [];
    }
  }

  if (typeof rawValue === 'object' && rawValue !== null) {
    if (Array.isArray(rawValue.values)) {
      return rawValue.values.map((id) => Number(id)).filter((id) => !Number.isNaN(id));
    }
  }

  return [];
};

const classifyTransactionDirection = (entry, bankAccountId, bufferAccountId) => {
  if (!entry || !entry.lines) {
    return 'neutral';
  }

  let bankLine;
  let bufferLine;

  entry.lines.forEach((line) => {
    if (line.accountId === bankAccountId) {
      bankLine = line;
    }
    if (line.accountId === bufferAccountId) {
      bufferLine = line;
    }
  });

  if (!bankLine || !bufferLine) {
    return 'neutral';
  }

  const debit = parseFloat(bankLine.debitAmount || 0);
  const credit = parseFloat(bankLine.creditAmount || 0);

  if (debit > credit) {
    return 'inflow';
  }
  if (credit > debit) {
    return 'outflow';
  }

  return 'neutral';
};

const BADGE_TONE_CLASSES = {
  green: 'bg-green-100 text-green-800',
  yellow: 'bg-yellow-100 text-yellow-800',
  slate: 'bg-slate-100 text-slate-800',
};

const DocumentBadge = ({ label, tone = 'slate' }) => {
  const toneClasses = BADGE_TONE_CLASSES[tone] ?? BADGE_TONE_CLASSES.slate;
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${toneClasses}`}>
      {label}
    </span>
  );
};

const getDocumentBadge = (entry) => {
  if (!entry) {
    return null;
  }

  if (entry.isPosted) {
    return <DocumentBadge label="Приключен" tone="green" />;
  }

  return <DocumentBadge label="Чернова" tone="yellow" />;
};

export default function BankStatementsReview({ companyId, bankProfiles, accounts }) {
  const [selectedProfileId, setSelectedProfileId] = useState(() => bankProfiles[0]?.id ?? null);
  const [dateFilter, setDateFilter] = useState({ from: '', to: '' });
  const [imports, setImports] = useState([]);
  const [importsLoading, setImportsLoading] = useState(false);
  const [importsLoadingMore, setImportsLoadingMore] = useState(false);
  const [importsHasMore, setImportsHasMore] = useState(true);
  const [importsError, setImportsError] = useState(null);
  const [selectedImportId, setSelectedImportId] = useState(null);
  const [entriesCache, setEntriesCache] = useState({});
  const [entriesLoading, setEntriesLoading] = useState(false);
  const [entriesError, setEntriesError] = useState(null);
  const [counterparts, setCounterparts] = useState([]);
  const [counterpartsLoading, setCounterpartsLoading] = useState(false);
  const [counterpartsError, setCounterpartsError] = useState(null);
  const [editingContext, setEditingContext] = useState(null);
  const [editingValues, setEditingValues] = useState({ accountId: '', counterpartId: '' });
  const [editingStatus, setEditingStatus] = useState({ loading: false, error: null, success: null });

  const importsContainerRef = useRef(null);
  const importsRef = useRef([]);

  useEffect(() => {
    importsRef.current = imports;
  }, [imports]);

  const accountsMap = useMemo(() => {
    const map = new Map();
    if (Array.isArray(accounts)) {
      accounts.forEach((account) => {
        map.set(account.id, account);
      });
    }
    return map;
  }, [accounts]);

  const accountOptions = useMemo(() => {
    if (!Array.isArray(accounts)) {
      return [];
    }
    return accounts.map((account) => ({
      value: String(account.id),
      label: `${account.code} · ${account.name}`,
    }));
  }, [accounts]);

  const selectedProfile = useMemo(
    () => bankProfiles.find((profile) => profile.id === selectedProfileId) ?? null,
    [bankProfiles, selectedProfileId],
  );

  const fetchImports = useCallback(
    async ({ reset = false } = {}) => {
      if (!selectedProfileId) {
        return;
      }

      if (reset) {
        setImportsLoading(true);
        setImportsLoadingMore(false);
        setImportsError(null);
        setImportsHasMore(true);
      } else {
        if (importsLoading || importsLoadingMore || !importsHasMore) {
          return;
        }
        setImportsLoadingMore(true);
        setImportsError(null);
      }

      try {
        const currentImports = reset ? [] : importsRef.current;
        const offsetValue = reset ? 0 : currentImports.length;

        const variables = {
          companyId,
          bankProfileId: selectedProfileId,
          limit: IMPORTS_PAGE_SIZE,
          offset: offsetValue,
        };

        if (dateFilter.from) {
          variables.fromDate = dateFilter.from;
        }
        if (dateFilter.to) {
          variables.toDate = dateFilter.to;
        }

        const data = await graphqlRequest(BANK_IMPORTS_QUERY, variables);
        const fetchedImports = (data?.bankImports ?? []).filter(
          (importRecord, index, array) =>
            array.findIndex((item) => item.id === importRecord.id) === index,
        );

        setImports((prev) => {
          if (reset) {
            return fetchedImports;
          }

          if (prev.length === 0) {
            return fetchedImports;
          }

          const existingIds = new Set(prev.map((item) => item.id));
          const newRecords = fetchedImports.filter((item) => !existingIds.has(item.id));
          if (!newRecords.length) {
            return prev;
          }
          return [...prev, ...newRecords];
        });

        setImportsHasMore(fetchedImports.length === IMPORTS_PAGE_SIZE);

        if (reset) {
          if (fetchedImports.length > 0) {
            setSelectedImportId(fetchedImports[0].id);
          } else {
            setSelectedImportId(null);
          }
        }
      } catch (err) {
        setImportsError(err.message);
        if (reset) {
          setImports([]);
          setSelectedImportId(null);
          setImportsHasMore(false);
        }
      } finally {
        if (reset) {
          setImportsLoading(false);
        } else {
          setImportsLoadingMore(false);
        }
      }
    },
    [
      companyId,
      selectedProfileId,
      dateFilter.from,
      dateFilter.to,
      importsHasMore,
      importsLoading,
      importsLoadingMore,
    ],
  );

  const handleImportsScroll = useCallback(
    (event) => {
      if (importsLoading || importsLoadingMore || !importsHasMore) {
        return;
      }

      const { scrollTop, scrollHeight, clientHeight } = event.target;
      if (scrollHeight - scrollTop - clientHeight < 80) {
        fetchImports({ reset: false });
      }
    },
    [fetchImports, importsHasMore, importsLoading, importsLoadingMore],
  );

  useEffect(() => {
    let isMounted = true;
    const loadCounterparts = async () => {
      setCounterpartsLoading(true);
      setCounterpartsError(null);
      try {
        const data = await graphqlRequest(COUNTERPARTS_QUERY, { companyId });
        if (!isMounted) {
          return;
        }
        setCounterparts(data?.counterparts ?? []);
      } catch (err) {
        if (isMounted) {
          setCounterpartsError(err.message);
          setCounterparts([]);
        }
      } finally {
        if (isMounted) {
          setCounterpartsLoading(false);
        }
      }
    };

    loadCounterparts();

    return () => {
      isMounted = false;
    };
  }, [companyId]);

  const counterpartOptions = useMemo(() => {
    return counterparts
      .filter((counterpart) => counterpart.isActive !== false)
      .map((counterpart) => ({
        value: String(counterpart.id),
        label: counterpart.eik ? `${counterpart.name} (${counterpart.eik})` : counterpart.name,
      }));
  }, [counterparts]);

  const counterpartsMap = useMemo(() => {
    const map = new Map();
    counterparts.forEach((counterpart) => {
      map.set(counterpart.id, counterpart);
    });
    return map;
  }, [counterparts]);

  useEffect(() => {
    if (!bankProfiles.length) {
      setSelectedProfileId(null);
      return;
    }

    if (!selectedProfileId || !bankProfiles.some((profile) => profile.id === selectedProfileId)) {
      setSelectedProfileId(bankProfiles[0].id);
    }
  }, [bankProfiles, selectedProfileId]);

  useEffect(() => {
    if (!selectedProfileId) {
      setImports([]);
      setEntriesCache({});
      setSelectedImportId(null);
      setImportsHasMore(false);
      return;
    }

    setEntriesCache({});
    setSelectedImportId(null);
    fetchImports({ reset: true });
  }, [companyId, selectedProfileId, dateFilter.from, dateFilter.to, fetchImports]);

  useEffect(() => {
    if (importsLoading || importsLoadingMore || !importsHasMore) {
      return;
    }

    if (!selectedProfileId || imports.length === 0) {
      return;
    }

    const container = importsContainerRef.current;
    if (!container) {
      return;
    }

    if (container.scrollHeight <= container.clientHeight + 24) {
      fetchImports({ reset: false });
    }
  }, [
    imports,
    importsHasMore,
    importsLoading,
    importsLoadingMore,
    selectedProfileId,
    fetchImports,
  ]);

  useEffect(() => {
    const loadEntries = async () => {
      if (!selectedImportId) {
        return;
      }

      const importRecord = imports.find((item) => item.id === selectedImportId);
      if (!importRecord) {
        return;
      }

      const entryIds = Array.from(new Set(parseJournalEntryIds(importRecord.journalEntryIds)));
      const missingIds = entryIds.filter((id) => !entriesCache[id]);

      if (!missingIds.length) {
        return;
      }

      setEntriesLoading(true);
      setEntriesError(null);
      try {
        const results = await Promise.all(
          missingIds.map(async (entryId) => {
            const response = await graphqlRequest(JOURNAL_ENTRY_WITH_LINES_QUERY, { id: entryId });
            return { entryId, data: response?.journalEntryWithLines };
          }),
        );

        const newEntries = {};
        results.forEach(({ entryId, data }) => {
          if (data) {
            const { lines = [], ...entryFields } = data;
            const normalizedLines = (data.lines || []).map((line) => ({
              ...line,
              accountId: Number(line.accountId),
              counterpartId: line.counterpartId ? Number(line.counterpartId) : null,
            }));
            newEntries[entryId] = {
              entry: {
                ...entryFields,
                id: Number(entryFields.id),
              },
              lines: normalizedLines.sort((a, b) => (a.lineOrder ?? 0) - (b.lineOrder ?? 0)),
            };
          }
        });

        if (Object.keys(newEntries).length > 0) {
          setEntriesCache((prev) => ({ ...prev, ...newEntries }));
        }
      } catch (err) {
        setEntriesError(err.message);
      } finally {
        setEntriesLoading(false);
      }
    };

    loadEntries();
  }, [imports, selectedImportId]);

  useEffect(() => {
    setEditingContext(null);
    setEditingValues({ accountId: '', counterpartId: '' });
    setEditingStatus({ loading: false, error: null, success: null });
  }, [selectedImportId, selectedProfileId]);

  const selectedImport = useMemo(
    () => imports.find((item) => item.id === selectedImportId) ?? null,
    [imports, selectedImportId],
  );

  const entryIds = useMemo(() => {
    if (!selectedImport) {
      return [];
    }
    return Array.from(new Set(parseJournalEntryIds(selectedImport.journalEntryIds)));
  }, [selectedImport]);

  const entryDetails = useMemo(() => {
    const allDetails = entryIds.map((id) => entriesCache[id] ?? { missing: true, id });

    return allDetails.filter((detail) => {
      // Always skip missing/deleted entries
      if (!detail || detail.missing || !detail.entry) {
        return false;
      }

      // If no date filter is active, show all valid entries
      if (!dateFilter.from && !dateFilter.to) {
        return true;
      }

      const docDate = detail.entry.documentDate;
      if (!docDate) {
        return false; // No document date, skip
      }

      // Check if document date is within range
      if (dateFilter.from && docDate < dateFilter.from) {
        return false; // Before start date
      }
      if (dateFilter.to && docDate > dateFilter.to) {
        return false; // After end date
      }

      return true; // Within range
    });
  }, [entryIds, entriesCache, dateFilter.from, dateFilter.to]);

  const totals = useMemo(() => {
    if (!selectedProfile) {
      return { entries: 0, debit: 0, credit: 0, balance: 0 };
    }

    let entriesCount = 0;
    let debit = 0;
    let credit = 0;

    entryDetails.forEach((detail) => {
      if (!detail || detail.missing) {
        return;
      }
      entriesCount += 1;
      detail.lines.forEach((line) => {
        if (line.accountId === selectedProfile.accountId) {
          debit += Number(line.debitAmount || 0);
          credit += Number(line.creditAmount || 0);
        }
      });
    });

    return {
      entries: entriesCount,
      debit,
      credit,
      balance: debit - credit,
    };
  }, [entryDetails, selectedProfile]);

  const periodLabel = useMemo(() => {
    if (dateFilter.from && dateFilter.to) {
      return `${formatDate(dateFilter.from)} – ${formatDate(dateFilter.to)}`;
    }
    if (dateFilter.from) {
      return `от ${formatDate(dateFilter.from)}`;
    }
    if (dateFilter.to) {
      return `до ${formatDate(dateFilter.to)}`;
    }
    return 'Всички дати';
  }, [dateFilter]);

  const getAccountLabel = (accountId) => {
    if (!accountId) {
      return '—';
    }
    const account = accountsMap.get(accountId);
    if (!account) {
      return `Сметка № ${accountId}`;
    }
    return `${account.code} · ${account.name}`;
  };

  const getCounterpartLabel = (counterpartId) => {
    if (!counterpartId) {
      return '—';
    }
    const counterpart = counterpartsMap.get(counterpartId);
    if (!counterpart) {
      return `Контрагент № ${counterpartId}`;
    }
  return counterpart.eik ? `${counterpart.name} (${counterpart.eik})` : counterpart.name;
};

const SearchableSelect = ({
  options,
  value,
  onChange,
  placeholder,
  disabled,
  emptyMessage = 'Няма резултат',
}) => {
  const containerRef = useRef(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');

  const selectedOption = useMemo(
    () => options.find((option) => option.value === value),
    [options, value],
  );

  useEffect(() => {
    if (!open) {
      setQuery(selectedOption ? selectedOption.label : '');
    }
  }, [selectedOption, open]);

  useEffect(() => {
    const handleClickOutside = (event) => {
      if (containerRef.current && !containerRef.current.contains(event.target)) {
        setOpen(false);
        // Restore the selected value when clicking outside
        setQuery(selectedOption ? selectedOption.label : '');
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [selectedOption]);

  const filteredOptions = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) {
      return options;
    }
    return options.filter((option) => option.label.toLowerCase().includes(normalizedQuery));
  }, [options, query]);

  return (
    <div ref={containerRef} className="relative">
      <input
        type="text"
        value={query}
        onChange={(event) => {
          setQuery(event.target.value);
          setOpen(true);
        }}
        onFocus={() => {
          setQuery('');
          setOpen(true);
        }}
        placeholder={placeholder}
        disabled={disabled}
        className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500 disabled:bg-gray-100"
      />
      {value && !disabled && (
        <button
          type="button"
          className="absolute inset-y-0 right-0 px-3 text-gray-400 hover:text-gray-600"
          onClick={() => {
            onChange('');
            setQuery('');
          }}
        >
          ✕
        </button>
      )}
      {open && !disabled && (
        <div className="absolute z-20 mt-1 w-full bg-white border border-gray-200 rounded-md shadow-lg max-h-48 overflow-y-auto">
          {filteredOptions.length === 0 ? (
            <div className="px-3 py-2 text-sm text-gray-500">{emptyMessage}</div>
          ) : (
            filteredOptions.map((option) => (
              <button
                key={option.value}
                type="button"
                className={`w-full text-left px-3 py-2 text-sm hover:bg-blue-50 ${
                  option.value === value ? 'bg-blue-100 text-blue-700' : 'text-gray-700'
                }`}
                onMouseDown={(event) => {
                  event.preventDefault();
                  onChange(option.value);
                  setQuery(option.label);
                  setOpen(false);
                }}
              >
                {option.label}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
};

  const refreshEntry = async (entryId) => {
    try {
      const response = await graphqlRequest(JOURNAL_ENTRY_WITH_LINES_QUERY, { id: entryId });
      const data = response?.journalEntryWithLines;
      setEntriesCache((prev) => {
        const updated = { ...prev };
        if (data) {
          const { lines = [], ...entryFields } = data;
          const normalizedLines = lines.map((line) => ({
            ...line,
            accountId: Number(line.accountId),
            counterpartId: line.counterpartId ? Number(line.counterpartId) : null,
          }));
          updated[entryId] = {
            entry: {
              ...entryFields,
              id: Number(entryFields.id),
            },
            lines: normalizedLines.sort((a, b) => (a.lineOrder ?? 0) - (b.lineOrder ?? 0)),
          };
        } else {
          updated[entryId] = { missing: true, id: entryId };
        }
        return updated;
      });
    } catch (err) {
      setEntriesError(err.message);
    }
  };

  const startEditingBufferLine = (entryId, line) => {
    setEditingContext({ entryId, lineId: line.id });
    setEditingValues({
      accountId: String(line.accountId ?? ''),
      counterpartId: line.counterpartId ? String(line.counterpartId) : '',
    });
    setEditingStatus({ loading: false, error: null, success: null });
  };

  const cancelEditing = () => {
    setEditingContext(null);
    setEditingValues({ accountId: '', counterpartId: '' });
    setEditingStatus({ loading: false, error: null, success: null });
  };

  const handleEditingChange = (field, value) => {
    setEditingValues((prev) => ({ ...prev, [field]: value }));
  };

  const saveBufferLine = async () => {
    if (!editingContext) {
      return;
    }

    const entryData = entriesCache[editingContext.entryId];
    if (!entryData || entryData.missing) {
      setEditingStatus({ loading: false, error: 'Журналният запис не е наличен.', success: null });
      return;
    }

    if (!editingValues.accountId) {
      setEditingStatus({ loading: false, error: 'Моля изберете счетоводна сметка.', success: null });
      return;
    }

    const { entry, lines } = entryData;
    const updatedLinesPayload = lines.map((line) => {
      const isTarget = line.id === editingContext.lineId;
      const accountId = isTarget ? Number(editingValues.accountId) : Number(line.accountId);
      const counterpartId = isTarget
        ? editingValues.counterpartId
          ? Number(editingValues.counterpartId)
          : null
        : line.counterpartId ?? null;

      return {
        accountId,
        debitAmount:
          line.debitAmount !== null && line.debitAmount !== undefined
            ? Number(line.debitAmount)
            : null,
        creditAmount:
          line.creditAmount !== null && line.creditAmount !== undefined
            ? Number(line.creditAmount)
            : null,
        description: line.description ?? entry.description ?? '',
        counterpartId,
        currencyCode: line.currencyCode || null,
        currencyAmount:
          line.currencyAmount !== null && line.currencyAmount !== undefined
            ? Number(line.currencyAmount)
            : null,
      };
    });

    setEditingStatus({ loading: true, error: null, success: null });
    try {
      await graphqlRequest(UPDATE_JOURNAL_ENTRY_MUTATION, {
        id: entry.id,
        input: {
          lines: updatedLinesPayload,
        },
      });

      await refreshEntry(entry.id);
      setEditingStatus({ loading: false, error: null, success: null });
      setEditingContext(null);
    } catch (err) {
      setEditingStatus({ loading: false, error: err.message, success: null });
    }
  };

  if (!bankProfiles.length) {
    return (
      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <p className="text-gray-600 text-sm">
          Няма конфигурирани банкови профили. Добавете профил, за да преглеждате банковите извлечения.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="bg-white border border-gray-200 rounded-lg p-6 shadow-sm">
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold text-gray-900">
              Банкови извлечения и журнални записи
            </h2>
            <p className="text-sm text-gray-500 mt-1">
              Преглед на импортнатите банкови транзакции и автоматично генерираните счетоводни записи.
            </p>
          </div>
          <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
            <div className="flex items-center space-x-3">
              <label htmlFor="bank-profile" className="text-sm font-medium text-gray-700">
                Банка
              </label>
              <select
                id="bank-profile"
                value={selectedProfileId ?? ''}
                onChange={(event) => setSelectedProfileId(Number(event.target.value))}
                className="border border-gray-300 rounded-md px-3 py-2 text-sm"
              >
                {bankProfiles.map((profile) => (
                  <option key={profile.id} value={profile.id}>
                    {profile.name} ({profile.currencyCode})
                  </option>
                ))}
              </select>
            </div>
            <div className="flex items-center space-x-4">
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">От дата</label>
                <input
                  type="date"
                  value={dateFilter.from}
                  onChange={(event) => setDateFilter((prev) => ({ ...prev, from: event.target.value }))}
                  className="border border-gray-300 rounded-md px-3 py-2 text-sm"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">До дата</label>
                <input
                  type="date"
                  value={dateFilter.to}
                  onChange={(event) => setDateFilter((prev) => ({ ...prev, to: event.target.value }))}
                  className="border border-gray-300 rounded-md px-3 py-2 text-sm"
                />
              </div>
              {(dateFilter.from || dateFilter.to) && (
                <button
                  type="button"
                  onClick={() => setDateFilter({ from: '', to: '' })}
                  className="mt-5 px-3 py-2 text-xs font-medium text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50"
                >
                  Изчисти
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <div className="lg:col-span-1 space-y-4">
          <div className="bg-white border border-gray-200 rounded-lg shadow-sm">
            <div className="px-4 py-3 border-b border-gray-200">
              <h3 className="text-sm font-semibold text-gray-900">Импорти</h3>
            </div>

            <div className="divide-y divide-gray-100 max-h-[520px] overflow-y-auto">
              {importsLoading ? (
                <div className="p-4 text-sm text-gray-500">Зареждане...</div>
              ) : importsError ? (
                <div className="p-4 text-sm text-red-600">Грешка: {importsError}</div>
              ) : imports.length === 0 ? (
                <div className="p-4 text-sm text-gray-500">
                  Няма импорти за избраната банка.
                </div>
              ) : (
                imports.map((record) => {
                  const isSelected = record.id === selectedImportId;
                  return (
                    <button
                      key={record.id}
                      type="button"
                      onClick={() => {
                        setSelectedImportId(record.id);
                        setEntriesError(null);
                      }}
                      className={`w-full text-left px-4 py-3 text-sm transition ${
                        isSelected ? 'bg-blue-50 border-l-4 border-blue-400' : 'hover:bg-gray-50'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-gray-900">
                          {formatDateTime(record.importedAt)}
                        </span>
                        <span className="text-xs text-gray-500">
                          {record.transactionsCount} записа
                        </span>
                      </div>
                      <div className="mt-1 text-xs text-gray-500">
                        {formatCurrency(record.totalDebit)} / {formatCurrency(record.totalCredit)}
                      </div>
                    </button>
                  );
                })
              )}
            </div>
          </div>
        </div>

        <div className="lg:col-span-3 space-y-4">
          <div className="bg-white border border-gray-200 rounded-lg shadow-sm p-5">
            <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-base font-semibold text-gray-900">Обобщение</h3>
                    <p className="text-sm text-gray-500">Период: {periodLabel}</p>
                    {selectedProfile && (
                      <p className="text-xs text-gray-500">
                        Банкова сметка: {getAccountLabel(selectedProfile.accountId)} • Буфер: {getAccountLabel(selectedProfile.bufferAccountId)}
                      </p>
                    )}
                  </div>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4 text-sm text-gray-700">
                <div>
                  <div className="text-xs text-gray-500 uppercase tracking-wide">Записи</div>
                  <div className="text-base font-semibold text-gray-900">{totals.entries}</div>
                </div>
                <div>
                  <div className="text-xs text-gray-500 uppercase tracking-wide">Оборот Дт</div>
                  <div className="text-base font-semibold text-gray-900">{formatCurrency(totals.debit)}</div>
                </div>
                <div>
                  <div className="text-xs text-gray-500 uppercase tracking-wide">Оборот Кт</div>
                  <div className="text-base font-semibold text-gray-900">{formatCurrency(totals.credit)}</div>
                </div>
                <div>
                  <div className="text-xs text-gray-500 uppercase tracking-wide">Салдо</div>
                  <div className={`text-base font-semibold ${totals.balance === 0 ? 'text-gray-900' : totals.balance > 0 ? 'text-green-600' : 'text-red-600'}`}>{formatCurrency(totals.balance)}</div>
                </div>
              </div>
            </div>
          </div>

          {entriesError && (
            <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
              {entriesError}
            </div>
          )}

          {entriesLoading && entryDetails.length === 0 && (
            <div className="bg-white border border-gray-200 rounded-lg p-6 text-center text-gray-500">
              Зареждане на журналните записи...
            </div>
          )}

          {!entryDetails.length && !entriesLoading && (
            <div className="bg-white border border-gray-200 rounded-lg p-6 text-center text-gray-500">
              Няма налични журнални записи за избрания импорт.
            </div>
          )}

          {entryDetails.map((detail) => {
            if (!detail) {
              return null;
            }

            if (detail.missing) {
              return (
                <div
                  key={`missing-${detail.id}`}
                  className="bg-white border border-red-200 rounded-lg shadow-sm px-6 py-4 text-sm text-red-600"
                >
                  Журналният запис с идентификатор {detail.id} не е наличен (вероятно е изтрит).
                </div>
              );
            }

            const { entry, lines } = detail;
            const direction = classifyTransactionDirection(
              { lines },
              selectedProfile?.accountId,
              selectedProfile?.bufferAccountId,
            );

            const bankLine = lines.find((line) => line.accountId === selectedProfile?.accountId);
            const bufferLine = lines.find((line) => line.accountId === selectedProfile?.bufferAccountId);

            return (
              <div key={entry.id} className="bg-white border border-gray-200 rounded-lg shadow-sm overflow-hidden">
                <div className="border-b border-gray-200 px-6 py-4 flex flex-col md:flex-row md:items-center md:justify-between gap-3">
                  <div>
                    <div className="flex items-center gap-3">
                      <h4 className="text-base font-semibold text-gray-900">
                        Запис № {entry.entryNumber}
                      </h4>
                      {getDocumentBadge(entry)}
                    </div>
                    <div className="text-sm text-gray-500">
                      Документ № {entry.documentNumber || '—'} • Дата {formatDate(entry.documentDate)}
                    </div>
                  </div>
                  <div className="flex items-center gap-4 text-sm text-gray-600">
                    <span>
                      Стойност: <strong>{formatCurrency(entry.totalAmount)}</strong>
                    </span>
                    <a
                      href={`/accounting/entries?edit=${entry.id}`}
                      className="text-blue-600 hover:text-blue-800"
                    >
                      Отвори в журналните записи →
                    </a>
                  </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-2">
                  <div className={`p-6 border-b lg:border-b-0 lg:border-r border-gray-200 ${direction === 'inflow' ? 'bg-green-50' : direction === 'outflow' ? 'bg-red-50' : 'bg-slate-50'}`}>
                    <h5 className="text-sm font-semibold text-gray-800 mb-4">Детайли за банковата транзакция</h5>
                    <dl className="space-y-3 text-sm text-gray-700">
                      <div>
                        <dt className="font-medium text-gray-600">Описание</dt>
                        <dd className="mt-1 text-gray-900 whitespace-pre-line">
                          {entry.description || '—'}
                        </dd>
                      </div>
                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div>
                          <dt className="font-medium text-gray-600">Дата на документа</dt>
                          <dd className="mt-1 text-gray-900">{formatDate(entry.documentDate)}</dd>
                        </div>
                        <div>
                          <dt className="font-medium text-gray-600">Вальор</dt>
                          <dd className="mt-1 text-gray-900">{formatDate(entry.vatDate)}</dd>
                        </div>
                      </div>
                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div>
                          <dt className="font-medium text-gray-600">Сметка</dt>
                          <dd className="mt-1 text-gray-900">
                            {selectedProfile ? getAccountLabel(selectedProfile.accountId) : '—'}
                          </dd>
                        </div>
                        <div>
                          <dt className="font-medium text-gray-600">Буферна сметка</dt>
                          <dd className="mt-1 text-gray-900">
                            {selectedProfile ? getAccountLabel(selectedProfile.bufferAccountId) : '—'}
                          </dd>
                        </div>
                      </div>
                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div>
                          <dt className="font-medium text-gray-600">Дебит по банка</dt>
                          <dd className="mt-1 text-gray-900">
                            {bankLine ? formatCurrency(bankLine.debitAmount || 0) : '—'}
                          </dd>
                        </div>
                        <div>
                          <dt className="font-medium text-gray-600">Кредит по банка</dt>
                          <dd className="mt-1 text-gray-900">
                            {bankLine ? formatCurrency(bankLine.creditAmount || 0) : '—'}
                          </dd>
                        </div>
                      </div>
                      <div>
                        <dt className="font-medium text-gray-600">Посока</dt>
                        <dd className="mt-1 text-gray-900">
                          {direction === 'inflow'
                            ? 'Входящ превод към банковата сметка'
                            : direction === 'outflow'
                            ? 'Изходящ превод от банковата сметка'
                            : 'Неутрален'}
                        </dd>
                      </div>
                    </dl>
                  </div>
                  <div className="p-6">
                    <h5 className="text-sm font-semibold text-gray-800 mb-4">Счетоводни редове</h5>
                    <div className="space-y-3">
                      {lines.map((line) => {
                        const isBankLine = line.accountId === selectedProfile?.accountId;
                        const isBufferLine = line.accountId === selectedProfile?.bufferAccountId;
                        const isEditing =
                          editingContext &&
                          editingContext.entryId === entry.id &&
                          editingContext.lineId === line.id;

                        const baseClasses = isBankLine
                          ? 'border-blue-300 bg-blue-50'
                          : isBufferLine
                          ? 'border-amber-300 bg-amber-50 cursor-pointer hover:border-amber-400'
                          : 'border-gray-200 bg-white';

                        return (
                          <div
                            key={line.id}
                            className={`rounded-lg border px-4 py-3 text-sm transition ${baseClasses}`}
                            role={isBufferLine ? 'button' : undefined}
                            tabIndex={isBufferLine ? 0 : undefined}
                            onClick={() => {
                              if (isBufferLine) {
                                startEditingBufferLine(entry.id, line);
                              }
                            }}
                            onKeyDown={(event) => {
                              if (isBufferLine && (event.key === 'Enter' || event.key === ' ')) {
                                event.preventDefault();
                                startEditingBufferLine(entry.id, line);
                              }
                            }}
                          >
                            <div className="flex items-center justify-between">
                              <div className="font-medium text-gray-900">
                                {getAccountLabel(line.accountId)}
                              </div>
                              <div className="ml-4 text-xs text-gray-500">
                                Ред {line.lineOrder ?? '—'}
                              </div>
                            </div>
                            {line.description && (
                              <div className="mt-1 text-xs text-gray-600">
                                {line.description}
                              </div>
                            )}
                            {isBufferLine && line.counterpartId && (
                              <div className="mt-1 text-xs text-gray-500">
                                Контрагент: {getCounterpartLabel(line.counterpartId)}
                              </div>
                            )}
                            {isBufferLine && !isEditing && (
                              <div className="mt-1 text-xs text-amber-600">
                                Щракнете, за да разпределите буферния ред.
                              </div>
                            )}
                            <div className="mt-3 grid grid-cols-1 sm:grid-cols-2 gap-3">
                              <div>
                                <div className="text-xs text-gray-500 uppercase tracking-wide">Дебит</div>
                                <div className="text-sm font-semibold text-gray-900">
                                  {formatCurrency(line.debitAmount)}
                                </div>
                              </div>
                              <div>
                                <div className="text-xs text-gray-500 uppercase tracking-wide">Кредит</div>
                                <div className="text-sm font-semibold text-gray-900">
                                  {formatCurrency(line.creditAmount)}
                                </div>
                              </div>
                            </div>

                            {isBufferLine && isEditing && (
                              <div
                                className="mt-4 border-t border-amber-200 pt-3 space-y-3"
                                onClick={(event) => event.stopPropagation()}
                              >
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                  <div>
                                    <label className="block text-xs font-medium text-gray-600 mb-1">
                                      Нова сметка
                                    </label>
                                    <SearchableSelect
                                      options={accountOptions}
                                      value={editingValues.accountId}
                                      onChange={(val) => handleEditingChange('accountId', val)}
                                      placeholder="Изберете сметка"
                                      disabled={editingStatus.loading}
                                    />
                                  </div>
                                  <div>
                                    <label className="block text-xs font-medium text-gray-600 mb-1">
                                      Контрагент
                                    </label>
                                    <SearchableSelect
                                      options={counterpartOptions}
                                      value={editingValues.counterpartId}
                                      onChange={(val) => handleEditingChange('counterpartId', val)}
                                      placeholder="Изберете контрагент"
                                      disabled={editingStatus.loading || counterpartsLoading}
                                      emptyMessage="Няма контрагенти"
                                    />
                                  </div>
                                </div>
                                <div className="text-xs text-gray-500">
                                  Описание: {line.description || entry.description || '—'}
                                </div>
                                <div className="text-xs text-gray-500">
                                  Сума: Дт {formatCurrency(line.debitAmount)} / Кт {formatCurrency(line.creditAmount)}
                                </div>
                                {counterpartsError && (
                                  <div className="text-xs text-red-600">
                                    Грешка при зареждане на контрагенти: {counterpartsError}
                                  </div>
                                )}
                                {editingStatus.error && (
                                  <div className="text-xs text-red-600">{editingStatus.error}</div>
                                )}
                                {editingStatus.success && (
                                  <div className="text-xs text-green-600">{editingStatus.success}</div>
                                )}
                                <div className="flex items-center gap-2 pt-1">
                                  <button
                                    type="button"
                                    className="px-3 py-1.5 text-xs font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:bg-blue-300"
                                    disabled={editingStatus.loading || !editingValues.accountId}
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      saveBufferLine();
                                    }}
                                  >
                                    {editingStatus.loading ? 'Запис...' : 'Запази'}
                                  </button>
                                  <button
                                    type="button"
                                    className="px-3 py-1.5 text-xs font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
                                    disabled={editingStatus.loading}
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      cancelEditing();
                                    }}
                                  >
                                    Отказ
                                  </button>
                                </div>
                              </div>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
