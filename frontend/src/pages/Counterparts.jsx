import React, { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';
import {
  validateVatNumber,
  parseAddress,
  isViesValidationEnabled,
  isAiMappingEnabled
} from '../services/contragentApi';

export default function Counterparts() {
  const [counterparts, setCounterparts] = useState([]);
  const [filteredCounterparts, setFilteredCounterparts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [companies, setCompanies] = useState([]);
  const [selectedCompany, setSelectedCompany] = useState(null);
  const [showModal, setShowModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [showValidateModal, setShowValidateModal] = useState(false);
  const [editingCounterpart, setEditingCounterpart] = useState(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState('ALL');
  const [importData, setImportData] = useState('');
  const [importProgress, setImportProgress] = useState(null);
  const [validationProgress, setValidationProgress] = useState(null);
  const [selectedForValidation, setSelectedForValidation] = useState([]);

  // Form data
  const [formData, setFormData] = useState({
    name: '',
    eik: '',
    vat_number: '',
    street: '',
    address: '',
    city: '',
    postal_code: '',
    country: 'България',
    phone: '',
    email: '',
    contact_person: '',
    counterpart_type: 'OTHER',
    is_customer: false,
    is_supplier: false,
    is_vat_registered: false,
    company_id: ''
  });

  const [validatingVat, setValidatingVat] = useState(false);
  const [vatValidationResult, setVatValidationResult] = useState(null);
  const [parsingAddress, setParsingAddress] = useState(false);

  const COUNTERPART_FIELD_MAP = {
    name: 'name',
    eik: 'eik',
    vat_number: 'vatNumber',
    street: 'street',
    address: 'address',
    city: 'city',
    postal_code: 'postalCode',
    country: 'country',
    phone: 'phone',
    email: 'email',
    contact_person: 'contactPerson',
    counterpart_type: 'counterpartType',
    is_customer: 'isCustomer',
    is_supplier: 'isSupplier',
    is_vat_registered: 'isVatRegistered',
    is_active: 'isActive',
    company_id: 'companyId',
  };

  const toGraphQLCounterpartInput = (data, { includeCompanyId = false, companyId } = {}) => {
    const payload = { ...data };

    if (includeCompanyId) {
      payload.company_id = companyId ?? data.company_id;
    } else {
      delete payload.company_id;
    }

    return Object.entries(payload).reduce((acc, [key, value]) => {
      if (value === undefined) {
        return acc;
      }

      const mappedKey = COUNTERPART_FIELD_MAP[key] || key;
      acc[mappedKey] = value === '' ? null : value;
      return acc;
    }, {});
  };

  const COUNTERPART_TYPES = [
    { value: 'CUSTOMER', label: 'Клиент', icon: '👤' },
    { value: 'SUPPLIER', label: 'Доставчик', icon: '🏭' },
    { value: 'EMPLOYEE', label: 'Служител', icon: '👨‍💼' },
    { value: 'BANK', label: 'Банка', icon: '🏦' },
    { value: 'GOVERNMENT', label: 'Държавна институция', icon: '🏛️' },
    { value: 'OTHER', label: 'Друго', icon: '📋' }
  ];

  const GET_COMPANIES_QUERY = `
    query GetCompanies {
      companies {
        id
        name
        eik
        isActive
      }
    }
  `;

  const GET_COUNTERPARTS_QUERY = `
    query GetCounterparts($companyId: Int!) {
      counterparts(companyId: $companyId) {
        id
        name
        eik
        vatNumber
        street
        address
        city
        postalCode
        country
        phone
        email
        contactPerson
        counterpartType
        isCustomer
        isSupplier
        isVatRegistered
        isActive
        companyId
        createdAt
        updatedAt
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
        counterpartType
        isVatRegistered
      }
    }
  `;

  const UPDATE_COUNTERPART_MUTATION = `
    mutation UpdateCounterpart($id: Int!, $input: UpdateCounterpartInput!) {
      updateCounterpart(id: $id, input: $input) {
        id
        name
        eik
        vatNumber
        counterpartType
        isVatRegistered
      }
    }
  `;

  useEffect(() => {
    loadCompanies();
  }, []);

  useEffect(() => {
    if (selectedCompany) {
      loadCounterparts();
    }
  }, [selectedCompany]);

  // Filter and search counterparts
  useEffect(() => {
    let filtered = counterparts;

    // Apply type filter
    if (filterType !== 'ALL') {
      filtered = filtered.filter(cp => cp.counterpartType === filterType);
    }

    // Apply search
    if (searchTerm) {
      const search = searchTerm.toLowerCase();
      filtered = filtered.filter(cp =>
        cp.name.toLowerCase().includes(search) ||
        (cp.eik && cp.eik.toLowerCase().includes(search)) ||
        (cp.vatNumber && cp.vatNumber.toLowerCase().includes(search))
      );
    }

    setFilteredCounterparts(filtered);
  }, [counterparts, filterType, searchTerm]);

  const loadCompanies = async () => {
    try {
      const result = await graphqlRequest(GET_COMPANIES_QUERY);
      const activeCompanies = result.companies.filter(c => c.isActive);
      setCompanies(activeCompanies);
      if (activeCompanies.length > 0) {
        setSelectedCompany(activeCompanies[0]);
      }
    } catch (err) {
      setError('Грешка при зареждане на фирми: ' + err.message);
    }
  };

  const loadCounterparts = async () => {
    if (!selectedCompany) return;

    try {
      setLoading(true);
      const result = await graphqlRequest(GET_COUNTERPARTS_QUERY, {
        companyId: selectedCompany.id
      });
      setCounterparts(result.counterparts || []);
    } catch (err) {
      setError('Грешка при зареждане на контрагенти: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();

    try {
      const input = toGraphQLCounterpartInput(
        {
          ...formData,
          is_vat_registered: formData.is_vat_registered,
        },
        {
          includeCompanyId: !editingCounterpart,
          companyId: selectedCompany?.id,
        }
      );

      if (editingCounterpart) {
        await graphqlRequest(UPDATE_COUNTERPART_MUTATION, {
          id: editingCounterpart.id,
          input,
        });
      } else {
        await graphqlRequest(CREATE_COUNTERPART_MUTATION, { input });
      }

      setShowModal(false);
      setEditingCounterpart(null);
      resetForm();
      await loadCounterparts();
    } catch (err) {
      alert('Грешка при запазване: ' + err.message);
    }
  };

  const resetForm = () => {
    setFormData({
      name: '',
      eik: '',
      vat_number: '',
      street: '',
      address: '',
      city: '',
      postal_code: '',
      country: 'България',
      phone: '',
      email: '',
      contact_person: '',
      counterpart_type: 'OTHER',
      is_vat_registered: false,
      company_id: ''
    });
  };

  const handleEdit = (counterpart) => {
    setEditingCounterpart(counterpart);
    setFormData({
      name: counterpart.name || '',
      eik: counterpart.eik || '',
      vat_number: counterpart.vatNumber || '',
      street: counterpart.street || '',
      address: counterpart.address || '',
      city: counterpart.city || '',
      postal_code: counterpart.postalCode || '',
      country: counterpart.country || 'България',
      phone: counterpart.phone || '',
      email: counterpart.email || '',
      contact_person: counterpart.contactPerson || '',
      counterpart_type: counterpart.counterpartType || 'OTHER',
      is_customer: counterpart.isCustomer || false,
      is_supplier: counterpart.isSupplier || false,
      is_vat_registered: counterpart.isVatRegistered || false,
      company_id: counterpart.companyId
    });
    setVatValidationResult(null); // Изчистваме предишната валидация
    setShowModal(true);
  };

  const handleNew = () => {
    setEditingCounterpart(null);
    resetForm();
    setVatValidationResult(null);
    setShowModal(true);
  };


  const getTypeLabel = (type) => {
    const typeData = COUNTERPART_TYPES.find(t => t.value === type);
    return typeData ? typeData.label : type;
  };

  const getTypeIcon = (type) => {
    const typeData = COUNTERPART_TYPES.find(t => t.value === type);
    return typeData ? typeData.icon : '📋';
  };

  const handleVatValidation = async () => {
    if (!formData.vat_number || !isViesValidationEnabled()) return;

    setValidatingVat(true);
    setVatValidationResult(null);

    try {
      const result = await validateVatNumber(formData.vat_number);
      setVatValidationResult(result);

      if (result.valid) {
        // Auto-fill company data from VIES
        setFormData(prev => ({
          ...prev,
          name: result.companyName || prev.name,
          is_vat_registered: true
        }));

        // If AI mapping is enabled and we have an address, parse it
        if (isAiMappingEnabled()) {
          await handleAddressParsing(result);
        }
      }
    } catch (error) {
      console.error('Error validating VAT:', error);
      setVatValidationResult({ valid: false, error: error.message });
    } finally {
      setValidatingVat(false);
    }
  };

  // Функция за автоматично извличане при промяна на ДДС номера
  const handleVatNumberChange = (e) => {
    const vatNumber = e.target.value.toUpperCase();
    setFormData({...formData, vat_number: vatNumber});

    // Изчистваме предишния резултат при промяна
    if (vatValidationResult) {
      setVatValidationResult(null);
    }

    // Автоматично извличане ако е валиден ДДС номер формат
    if (vatNumber.length >= 10 && vatNumber.match(/^[A-Z]{2}[0-9]+$/) && isViesValidationEnabled()) {
      // Използваме setTimeout за debouncing - изчакваме потребителят да престане да пише
      clearTimeout(window.vatValidationTimeout);
      window.vatValidationTimeout = setTimeout(async () => {
        try {
          setValidatingVat(true);
          const result = await validateVatNumber(vatNumber);
          setVatValidationResult(result);

          if (result.valid) {
            // Auto-fill company data from VIES
            setFormData(prev => ({
              ...prev,
              name: result.companyName || prev.name,
              is_vat_registered: true
            }));

            // If AI mapping is enabled and we have an address, parse it
            if (isAiMappingEnabled()) {
              await handleAddressParsing(result);
            }
          }
        } catch (error) {
          console.error('Error validating VAT:', error);
          setVatValidationResult({ valid: false, error: error.message });
        } finally {
          setValidatingVat(false);
        }
      }, 1500); // Изчакваме 1.5 секунди след като потребителят престане да пише
    }
  };

  const handleAddressParsing = async (validationData) => {
    if (!validationData || !isAiMappingEnabled()) return;

    setParsingAddress(true);

    try {
      const parsed = await parseAddress(validationData);
      const street = parsed?.street ? parsed.street.trim() : '';
      const number = parsed?.number ? parsed.number.trim() : '';
      const combinedStreet = [street, number].filter(Boolean).join(' ');
      const fallbackAddress = validationData.streetName
        || validationData.address
        || validationData.contragent?.address
        || validationData.contragent?.longAddress
        || '';

      setFormData(prev => ({
        ...prev,
        street: combinedStreet || fallbackAddress || prev.street,
        address: fallbackAddress || combinedStreet || prev.address,
        city: parsed?.city || validationData.city || validationData.contragent?.city || prev.city,
        postal_code: parsed?.postalCode
          || validationData.postalCode
          || validationData.contragent?.postalCode
          || prev.postal_code,
        country: parsed?.country || validationData.country || validationData.contragent?.country || prev.country
      }));
    } catch (error) {
      console.error('Error parsing address:', error);
    } finally {
      setParsingAddress(false);
    }
  };

  const handleImport = async () => {
    if (!importData) {
      alert('Моля, въведете данни за импорт');
      return;
    }

    setImportProgress({ status: 'processing', message: 'Обработка на данните...' });

    try {
      // Parse CSV or JSON data
      const rows = parseImportData(importData);
      const totalRows = rows.length;
      let processed = 0;
      let errors = [];

      for (const row of rows) {
        processed++;
        setImportProgress({
          status: 'processing',
          message: `Импортиране ${processed}/${totalRows}...`,
          percent: Math.round((processed / totalRows) * 100)
        });

        try {
          // Директно импортиране без валидация
          const input = toGraphQLCounterpartInput(row, {
            includeCompanyId: true,
            companyId: selectedCompany.id,
          });

          await graphqlRequest(CREATE_COUNTERPART_MUTATION, { input });
        } catch (error) {
          errors.push(`Ред ${processed}: ${error.message}`);
        }
      }

      if (errors.length > 0) {
        setImportProgress({
          status: 'warning',
          message: `Импортирани ${processed - errors.length}/${totalRows} записа с ${errors.length} грешки`,
          errors: errors
        });
      } else {
        setImportProgress({
          status: 'success',
          message: `Успешно импортирани ${processed} контрагента! Можете да ги валидирате чрез VIES.`
        });
      }

      await loadCounterparts();
      setTimeout(() => {
        setShowImportModal(false);
        setImportData('');
        setImportProgress(null);
      }, 3000);
    } catch (error) {
      setImportProgress({
        status: 'error',
        message: 'Грешка при импорт: ' + error.message
      });
    }
  };

  const handleBatchValidation = async () => {
    if (selectedForValidation.length === 0) {
      alert('Моля, изберете контрагенти за валидация');
      return;
    }

    setValidationProgress({ status: 'processing', message: 'Започване на валидация...' });

    try {
      const total = selectedForValidation.length;
      let processed = 0;
      let updated = 0;
      let errors = [];

      for (const counterpartId of selectedForValidation) {
        processed++;
        const counterpart = counterparts.find(c => c.id === counterpartId);

        setValidationProgress({
          status: 'processing',
          message: `Валидация ${processed}/${total}: ${counterpart.name}`,
          percent: Math.round((processed / total) * 100)
        });

        try {
          if (counterpart.vatNumber) {
            const validation = await validateVatNumber(counterpart.vatNumber);

            if (validation.valid && validation.companyName) {
              // Актуализираме само ако има нова информация от VIES
              const updateData = toGraphQLCounterpartInput({
                name: validation.companyName,
                is_vat_registered: true,
              });

              // Ако има адрес и е включено AI мапване
              if (validation.address && isAiMappingEnabled()) {
                try {
                  const parsed = await parseAddress(validation.address);
                  const parsedStreet = [parsed.street, parsed.number].filter(Boolean).join(' ').trim();
                  if (parsedStreet) {
                    updateData.street = parsedStreet;
                  }
                  if (validation.address) {
                    updateData.address = validation.address;
                  }
                  updateData.city = parsed.city || counterpart.city;
                  if (parsed.postalCode) {
                    updateData.postalCode = parsed.postalCode;
                  }
                  updateData.country = parsed.country || counterpart.country;
                } catch (addressError) {
                  console.warn('Address parsing failed:', addressError);
                }
              }

              await graphqlRequest(UPDATE_COUNTERPART_MUTATION, {
                id: counterpart.id,
                input: updateData
              });

              updated++;
            }
          }
        } catch (error) {
          errors.push(`${counterpart.name}: ${error.message}`);
        }
      }

      if (errors.length > 0) {
        setValidationProgress({
          status: 'warning',
          message: `Валидирани ${updated}/${total} контрагенти. ${errors.length} грешки.`,
          errors: errors
        });
      } else {
        setValidationProgress({
          status: 'success',
          message: `Успешно валидирани и актуализирани ${updated}/${total} контрагенти!`
        });
      }

      await loadCounterparts();
      setSelectedForValidation([]);

      setTimeout(() => {
        setShowValidateModal(false);
        setValidationProgress(null);
      }, 3000);
    } catch (error) {
      setValidationProgress({
        status: 'error',
        message: 'Грешка при валидация: ' + error.message
      });
    }
  };

  const parseImportData = (data) => {
    // Try to parse as JSON first
    try {
      const parsed = JSON.parse(data);
      return Array.isArray(parsed) ? parsed : [parsed];
    } catch {
      // Parse as CSV
      const lines = data.trim().split('\n');
      const headers = lines[0].split(',').map(h => h.trim());

      return lines.slice(1).map(line => {
        const values = line.split(',').map(v => v.trim());
        const obj = {};
        headers.forEach((header, index) => {
          obj[header] = values[index] || '';
        });
        return obj;
      });
    }
  };

  if (error) {
    return (
      <div className="p-6">
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow">
      {/* Header */}
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Контрагенти</h1>
          <p className="text-gray-600 mt-1">
            Управление на клиенти, доставчици и други контрагенти (SAF-T съвместими)
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowImportModal(true)}
            disabled={!selectedCompany}
            className="bg-green-600 text-white px-4 py-2 rounded-lg hover:bg-green-700 disabled:opacity-50"
          >
            📥 Импорт
          </button>
          <button
            onClick={() => setShowValidateModal(true)}
            disabled={!selectedCompany || filteredCounterparts.length === 0}
            className="bg-yellow-600 text-white px-4 py-2 rounded-lg hover:bg-yellow-700 disabled:opacity-50"
          >
            ✅ Валидирай VIES
          </button>
          <button
            onClick={handleNew}
            disabled={!selectedCompany}
            className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            ➕ Нов контрагент
          </button>
        </div>
      </div>

      {/* Company Selection */}
      <div className="mb-6">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Избери фирма:
        </label>
        <select
          value={selectedCompany?.id || ''}
          onChange={(e) => {
            const company = companies.find(c => c.id === parseInt(e.target.value));
            setSelectedCompany(company);
          }}
          className="border border-gray-300 rounded-md px-3 py-2 w-64"
        >
          <option value="">Избери фирма...</option>
          {companies.map(company => (
            <option key={company.id} value={company.id}>
              {company.name} ({company.eik})
            </option>
          ))}
        </select>
      </div>

      {selectedCompany && (
        <>
          {/* Filters */}
          <div className="mb-6 flex flex-wrap gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Търсене:
              </label>
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Име, ЕИК, ДДС номер..."
                className="border border-gray-300 rounded-md px-3 py-2 w-64"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Тип:
              </label>
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="border border-gray-300 rounded-md px-3 py-2"
              >
                <option value="ALL">Всички типове</option>
                {COUNTERPART_TYPES.map(type => (
                  <option key={type.value} value={type.value}>
                    {type.icon} {type.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Stats */}
          <div className="mb-6 grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-blue-50 p-4 rounded-lg">
              <div className="text-2xl font-bold text-blue-600">{filteredCounterparts.length}</div>
              <div className="text-sm text-blue-800">Общо контрагенти</div>
            </div>
            <div className="bg-green-50 p-4 rounded-lg">
              <div className="text-2xl font-bold text-green-600">
                {filteredCounterparts.filter(cp => cp.counterpartType === 'CUSTOMER').length}
              </div>
              <div className="text-sm text-green-800">Клиенти</div>
            </div>
            <div className="bg-orange-50 p-4 rounded-lg">
              <div className="text-2xl font-bold text-orange-600">
                {filteredCounterparts.filter(cp => cp.counterpartType === 'SUPPLIER').length}
              </div>
              <div className="text-sm text-orange-800">Доставчици</div>
            </div>
            <div className="bg-purple-50 p-4 rounded-lg">
              <div className="text-2xl font-bold text-purple-600">
                {filteredCounterparts.filter(cp => cp.isVatRegistered).length}
              </div>
              <div className="text-sm text-purple-800">ДДС регистрирани</div>
            </div>
          </div>

          {/* Table */}
          {loading ? (
            <div className="text-center py-8">
              <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
              <p className="mt-2 text-gray-600">Зареждане...</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full bg-white border border-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      <input
                        type="checkbox"
                        checked={selectedForValidation.length === filteredCounterparts.length && filteredCounterparts.length > 0}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setSelectedForValidation(filteredCounterparts.map(c => c.id));
                          } else {
                            setSelectedForValidation([]);
                          }
                        }}
                        className="h-4 w-4 text-blue-600"
                      />
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Контрагент
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Тип
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      ЕИК / ДДС номер
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Контакти
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      SAF-T роли
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      ДДС статус
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Действия
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {filteredCounterparts.length === 0 ? (
                    <tr>
                      <td colSpan="8" className="px-6 py-8 text-center text-gray-500">
                        {searchTerm || filterType !== 'ALL'
                          ? 'Няма контрагенти, отговарящи на филтрите'
                          : 'Няма добавени контрагенти'
                        }
                      </td>
                    </tr>
                  ) : (
                    filteredCounterparts.map((counterpart) => (
                      <tr key={counterpart.id} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap">
                          <input
                            type="checkbox"
                            checked={selectedForValidation.includes(counterpart.id)}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setSelectedForValidation([...selectedForValidation, counterpart.id]);
                              } else {
                                setSelectedForValidation(selectedForValidation.filter(id => id !== counterpart.id));
                              }
                            }}
                            className="h-4 w-4 text-blue-600"
                          />
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <div>
                            <div className="text-sm font-medium text-gray-900">
                              {counterpart.name}
                            </div>
                            {(counterpart.street || counterpart.address) && (
                              <div className="text-sm text-gray-500">
                                {counterpart.street || counterpart.address}
                                {counterpart.postalCode && `, ${counterpart.postalCode}`}
                                {counterpart.city && `, ${counterpart.city}`}
                              </div>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                            {getTypeIcon(counterpart.counterpartType)} {getTypeLabel(counterpart.counterpartType)}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <div>
                            {counterpart.eik && (
                              <div>ЕИК: {counterpart.eik}</div>
                            )}
                            {counterpart.vatNumber && (
                              <div>ДДС: {counterpart.vatNumber}</div>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <div>
                            {counterpart.phone && (
                              <div>📞 {counterpart.phone}</div>
                            )}
                            {counterpart.email && (
                              <div>📧 {counterpart.email}</div>
                            )}
                            {counterpart.contactPerson && (
                              <div>👤 {counterpart.contactPerson}</div>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <div className="flex flex-wrap gap-1">
                            {counterpart.isCustomer && (
                              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                👤 Клиент
                              </span>
                            )}
                            {counterpart.isSupplier && (
                              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                                🏭 Доставчик
                              </span>
                            )}
                            {!counterpart.isCustomer && !counterpart.isSupplier && (
                              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                                📋 Друго
                              </span>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            counterpart.isVatRegistered
                              ? 'bg-green-100 text-green-800'
                              : 'bg-gray-100 text-gray-800'
                          }`}>
                            {counterpart.isVatRegistered ? '✅ ДДС регистриран' : '❌ Не е ДДС регистриран'}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                          <button
                            onClick={() => handleEdit(counterpart)}
                            className="text-indigo-600 hover:text-indigo-900 mr-3"
                          >
                            ✏️ Редактиране
                          </button>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          )}
        </>
      )}

      {/* Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-2/3 lg:w-1/2 shadow-lg rounded-md bg-white">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-medium text-gray-900">
                {editingCounterpart ? `Редактиране на: ${editingCounterpart.name}` : 'Нов контрагент'}
              </h3>
              {editingCounterpart && (
                <div className="text-sm text-gray-500">
                  ID: {editingCounterpart.id} | Създаден: {new Date(editingCounterpart.createdAt).toLocaleDateString('bg-BG')}
                </div>
              )}
            </div>

            {/* VIES валидация статус */}
            {vatValidationResult && (
              <div className={`p-3 rounded-md mb-4 ${vatValidationResult.valid ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'}`}>
                <div className="flex items-center">
                  <span className={`text-sm font-medium ${vatValidationResult.valid ? 'text-green-800' : 'text-red-800'}`}>
                    {vatValidationResult.valid ? '✅ VIES: Валиден ДДС номер' : '❌ VIES: Невалиден ДДС номер'}
                  </span>
                </div>
                {vatValidationResult.companyName && (
                  <div className="text-sm text-green-700 mt-1">
                    Официално име: {vatValidationResult.companyName}
                  </div>
                )}
                {vatValidationResult.address && (
                  <div className="text-sm text-green-700">
                    Регистриран адрес: {vatValidationResult.address}
                  </div>
                )}
              </div>
            )}

            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Име/Наименование *
                  </label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({...formData, name: e.target.value})}
                    required
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Тип контрагент *
                  </label>
                  <select
                    value={formData.counterpart_type}
                    onChange={(e) => setFormData({...formData, counterpart_type: e.target.value})}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                    required
                  >
                    {COUNTERPART_TYPES.map(type => (
                      <option key={type.value} value={type.value}>
                        {type.icon} {type.label}
                      </option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    ЕИК
                  </label>
                  <input
                    type="text"
                    value={formData.eik}
                    onChange={(e) => setFormData({...formData, eik: e.target.value})}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    ДДС номер
                    {isViesValidationEnabled() && (
                      <span className="text-xs text-blue-600 ml-1">(автоматична VIES валидация)</span>
                    )}
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={formData.vat_number}
                      onChange={handleVatNumberChange}
                      placeholder="BG123456789"
                      className="mt-1 flex-1 border border-gray-300 rounded-md px-3 py-2"
                      autoComplete="off"
                    />
                    {isViesValidationEnabled() && (
                      <button
                        type="button"
                        onClick={handleVatValidation}
                        disabled={validatingVat || !formData.vat_number}
                        className="mt-1 px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 text-sm whitespace-nowrap font-medium"
                      >
                        {validatingVat ? '🔄 Извличане...' : '📥 Извлечи от VIES'}
                      </button>
                    )}
                  </div>
                  <p className="mt-1 text-xs text-gray-500">
                    Въведете ДДС номер с код на държава (напр. BG123456789)
                    {isViesValidationEnabled() && (
                      <span className="text-green-600 font-medium"> - автоматично извличане и AI мапване</span>
                    )}
                  </p>
                </div>

                <div className="md:col-span-2">
                  <label className="block text-sm font-medium text-gray-700">
                    Адрес
                    {isAiMappingEnabled() && (
                      <span className="text-xs text-purple-600 ml-1">(AI мапване включено)</span>
                    )}
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={formData.address}
                      onChange={(e) => setFormData({...formData, address: e.target.value})}
                      placeholder="ул. Иван Вазов 10"
                      className="mt-1 flex-1 border border-gray-300 rounded-md px-3 py-2"
                    />
                    {isAiMappingEnabled() && formData.address && (
                      <button
                        type="button"
                        onClick={() => handleAddressParsing(formData.address)}
                        disabled={parsingAddress}
                        className="mt-1 px-3 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:opacity-50 text-sm whitespace-nowrap"
                      >
                        {parsingAddress ? '🔄 AI мапва...' : '🤖 AI мапване'}
                      </button>
                    )}
                  </div>
                  <p className="mt-1 text-xs text-gray-500">
                    AI ще раздели адреса на компоненти (улица, номер, град)
                  </p>
                </div>

                <div className="md:col-span-2">
                  <label className="block text-sm font-medium text-gray-700">
                    Улица / булевард и номер
                  </label>
                  <input
                    type="text"
                    value={formData.street}
                    onChange={(e) => setFormData({...formData, street: e.target.value})}
                    placeholder="ул. Иван Вазов 10"
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Град
                  </label>
                  <input
                    type="text"
                    value={formData.city}
                    onChange={(e) => setFormData({...formData, city: e.target.value})}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Пощенски код
                  </label>
                  <input
                    type="text"
                    value={formData.postal_code}
                    onChange={(e) => setFormData({...formData, postal_code: e.target.value})}
                    placeholder="1000"
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Държава
                  </label>
                  <input
                    type="text"
                    value={formData.country}
                    onChange={(e) => setFormData({...formData, country: e.target.value})}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Телефон
                  </label>
                  <input
                    type="text"
                    value={formData.phone}
                    onChange={(e) => setFormData({...formData, phone: e.target.value})}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Имейл
                  </label>
                  <input
                    type="email"
                    value={formData.email}
                    onChange={(e) => setFormData({...formData, email: e.target.value})}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div className="md:col-span-2">
                  <label className="block text-sm font-medium text-gray-700">
                    Лице за контакт
                  </label>
                  <input
                    type="text"
                    value={formData.contact_person}
                    onChange={(e) => setFormData({...formData, contact_person: e.target.value})}
                    className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
                  />
                </div>

                <div className="md:col-span-2">
                  <div className="space-y-3">
                    <p className="text-sm font-medium text-gray-700">SAF-T роли:</p>
                    <div className="flex flex-wrap gap-4">
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          checked={formData.is_customer}
                          onChange={(e) => setFormData({...formData, is_customer: e.target.checked})}
                          className="h-4 w-4 text-blue-600"
                        />
                        <span className="ml-2 text-sm text-gray-900">
                          👤 Клиент (Customer)
                        </span>
                      </label>
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          checked={formData.is_supplier}
                          onChange={(e) => setFormData({...formData, is_supplier: e.target.checked})}
                          className="h-4 w-4 text-blue-600"
                        />
                        <span className="ml-2 text-sm text-gray-900">
                          🏭 Доставчик (Supplier)
                        </span>
                      </label>
                    </div>
                    <label className="flex items-center">
                      <input
                        type="checkbox"
                        checked={formData.is_vat_registered}
                        onChange={(e) => setFormData({...formData, is_vat_registered: e.target.checked})}
                        className="h-4 w-4 text-blue-600"
                      />
                      <span className="ml-2 text-sm text-gray-900">
                        💰 ДДС регистриран
                      </span>
                    </label>
                  </div>
                </div>
              </div>

              <div className="flex justify-end space-x-3 pt-6">
                <button
                  type="button"
                  onClick={() => {
                    setShowModal(false);
                    setEditingCounterpart(null);
                    resetForm();
                  }}
                  className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
                >
                  Отказ
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
                >
                  {editingCounterpart ? 'Запазване' : 'Създаване'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Import Modal */}
      {showImportModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-2/3 lg:w-1/2 shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-medium text-gray-900 mb-4">
              Импорт на контрагенти
            </h3>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Данни за импорт (CSV или JSON формат)
                </label>
                <textarea
                  value={importData}
                  onChange={(e) => setImportData(e.target.value)}
                  placeholder={`CSV формат:\nname,eik,vat_number,address,city,country,phone,email\n"Фирма АД","123456789","BG123456789","ул. Тест 1","София","България","02/1234567","test@example.com"\n\nJSON формат:\n[{"name":"Фирма АД","eik":"123456789","vat_number":"BG123456789"}]`}
                  rows={10}
                  className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                />
              </div>

              <div className="bg-blue-50 p-3 rounded-md">
                <p className="text-sm text-blue-800">
                  ℹ️ Импортът ще добави контрагентите директно без валидация. След импорт можете да използвате бутона "Валидирай VIES" за проверка и актуализация на данните.
                </p>
              </div>

              {importProgress && (
                <div className={`p-4 rounded-md ${
                  importProgress.status === 'success' ? 'bg-green-50 text-green-800' :
                  importProgress.status === 'error' ? 'bg-red-50 text-red-800' :
                  importProgress.status === 'warning' ? 'bg-yellow-50 text-yellow-800' :
                  'bg-blue-50 text-blue-800'
                }`}>
                  <div className="font-medium">{importProgress.message}</div>
                  {importProgress.percent && (
                    <div className="mt-2">
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-blue-600 h-2 rounded-full"
                          style={{ width: `${importProgress.percent}%` }}
                        />
                      </div>
                    </div>
                  )}
                  {importProgress.errors && (
                    <div className="mt-2 text-sm max-h-32 overflow-y-auto">
                      {importProgress.errors.map((err, idx) => (
                        <div key={idx}>• {err}</div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              <div className="flex justify-end space-x-3 pt-4">
                <button
                  type="button"
                  onClick={() => {
                    setShowImportModal(false);
                    setImportData('');
                    setImportProgress(null);
                  }}
                  className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
                >
                  Отказ
                </button>
                <button
                  onClick={handleImport}
                  disabled={!importData || importProgress?.status === 'processing'}
                  className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 disabled:opacity-50"
                >
                  Импортирай
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Validation Modal */}
      {showValidateModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-2/3 lg:w-1/2 shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-medium text-gray-900 mb-4">
              VIES валидация на контрагенти
            </h3>

            <div className="space-y-4">
              <div className="bg-yellow-50 p-4 rounded-md">
                <h4 className="font-medium text-yellow-800 mb-2">
                  Избрани за валидация: {selectedForValidation.length} контрагента
                </h4>
                <p className="text-sm text-yellow-700">
                  Тази операция ще провери ДДС номерата през EU VIES системата и ще актуализира:
                </p>
                <ul className="list-disc list-inside text-sm text-yellow-700 mt-2">
                  <li>Официалното име на фирмата</li>
                  <li>ДДС статуса (регистрация)</li>
                  {isAiMappingEnabled() && <li>Адресните данни (чрез AI мапване)</li>}
                </ul>
              </div>

              {selectedForValidation.length === 0 && (
                <div className="bg-red-50 p-3 rounded-md">
                  <p className="text-sm text-red-800">
                    ❌ Моля, изберете контрагенти от таблицата за валидация (чрез чекбоксовете)
                  </p>
                </div>
              )}

              {validationProgress && (
                <div className={`p-4 rounded-md ${
                  validationProgress.status === 'success' ? 'bg-green-50 text-green-800' :
                  validationProgress.status === 'error' ? 'bg-red-50 text-red-800' :
                  validationProgress.status === 'warning' ? 'bg-yellow-50 text-yellow-800' :
                  'bg-blue-50 text-blue-800'
                }`}>
                  <div className="font-medium">{validationProgress.message}</div>
                  {validationProgress.percent && (
                    <div className="mt-2">
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-blue-600 h-2 rounded-full"
                          style={{ width: `${validationProgress.percent}%` }}
                        />
                      </div>
                    </div>
                  )}
                  {validationProgress.errors && (
                    <div className="mt-2 text-sm max-h-32 overflow-y-auto">
                      {validationProgress.errors.map((err, idx) => (
                        <div key={idx}>• {err}</div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              <div className="flex justify-end space-x-3 pt-4">
                <button
                  type="button"
                  onClick={() => {
                    setShowValidateModal(false);
                    setValidationProgress(null);
                    setSelectedForValidation([]);
                  }}
                  className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
                >
                  Отказ
                </button>
                <button
                  onClick={handleBatchValidation}
                  disabled={selectedForValidation.length === 0 || validationProgress?.status === 'processing'}
                  className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-yellow-600 hover:bg-yellow-700 disabled:opacity-50"
                >
                  Започни валидация
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
