import React, { useState, useEffect } from 'react';

export default function SafTExport() {
  const [formData, setFormData] = useState({
    companyId: parseInt(localStorage.getItem('currentCompanyId')) || null,
    periodStart: 1, // January
    periodStartYear: new Date().getFullYear(),
    periodEnd: 12, // December
    periodEndYear: new Date().getFullYear(),
    fileType: 'Monthly', // Monthly, Annual, OnDemand
    taxAccountingBasis: 'A' // A=General commercial, P=Public, BANK, INSURANCE
  });

  const [validationResult, setValidationResult] = useState(null);
  const [isExporting, setIsExporting] = useState(false);
  const [exportResult, setExportResult] = useState(null);
  const [companies, setCompanies] = useState([]);

  // Mock companies data - this will be replaced with actual GraphQL queries later
  useEffect(() => {
    // Simulate loading companies
    setCompanies([
      { id: 1, name: "Тестова Компания ЕООД", eik: "123456789", vatNumber: "BG123456789" },
      { id: 2, name: "Друга Фирма АД", eik: "987654321", vatNumber: "BG987654321" }
    ]);
  }, []);

  const handleInputChange = (field, value) => {
    setFormData(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const handleValidate = async () => {
    if (!formData.companyId) {
      return;
    }

    try {
      // Mock validation - replace with actual GraphQL mutation
      const mockValidation = {
        isValid: true,
        validationErrors: [],
        numberOfTransactions: 1250,
        fileSizeBytes: 1250000
      };
      
      setValidationResult(mockValidation);
    } catch (error) {
      console.error('Validation error:', error);
      setValidationResult({ isValid: false, validationErrors: [error.message] });
    }
  };

  const handleExport = async () => {
    if (!validationResult?.isValid) {
      return;
    }

    setIsExporting(true);
    setExportResult(null);

    try {
      // Mock export - replace with actual GraphQL mutation
      await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate export time

      // Mock XML content for v1.0.1
      const mockXmlContent = `<?xml version="1.0" encoding="utf-8"?>
<nsSAFT:AuditFile xmlns:doc="urn:schemas-OECD:schema-extensions:documentation xml:lang=en" xmlns:nsSAFT="mf:nra:dgti:dxxxx:declaration:v1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <nsSAFT:Header>
    <nsSAFT:AuditFileVersion>007</nsSAFT:AuditFileVersion>
    <nsSAFT:AuditFileCountry>BG</nsSAFT:AuditFileCountry>
    <nsSAFT:AuditFileRegion>BG-22</nsSAFT:AuditFileRegion>
    <nsSAFT:AuditFileDateCreated>${new Date().toISOString().split('T')[0]}</nsSAFT:AuditFileDateCreated>
    <nsSAFT:SoftwareCompanyName>RS Accounting BG</nsSAFT:SoftwareCompanyName>
    <nsSAFT:SoftwareID>RS-AC-BG</nsSAFT:SoftwareID>
    <nsSAFT:SoftwareVersion>001</nsSAFT:SoftwareVersion>
    <nsSAFT:DefaultCurrencyCode>BGN</nsSAFT:DefaultCurrencyCode>
    <nsSAFT:HeaderComment>${formData.fileType === 'Annual' ? 'A' : formData.fileType === 'Monthly' ? 'M' : 'O'}</nsSAFT:HeaderComment>
    <nsSAFT:TaxAccountingBasis>${formData.taxAccountingBasis}</nsSAFT:TaxAccountingBasis>
  </nsSAFT:Header>
  <!-- Master Files and other sections based on file type -->
</nsSAFT:AuditFile>`;

      const fileName = `saft_${formData.companyId}_${formData.periodStartYear}_${String(formData.periodStart).padStart(2, '0')}_to_${formData.periodEndYear}_${String(formData.periodEnd).padStart(2, '0')}.xml`;
      
      // Create download
      const blob = new Blob([mockXmlContent], { type: 'application/xml' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = fileName;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      setExportResult({
        success: true,
        message: `SAF-T файлът "${fileName}" беше успешно експортиран!`
      });
    } catch (error) {
      console.error('Export error:', error);
      setExportResult({
        success: false,
        message: `Грешка при експортиране: ${error.message}`
      });
    } finally {
      setIsExporting(false);
    }
  };

  const formatFileSize = (bytes) => {
    if (!bytes) return '0 B';
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  const fileTypeOptions = [
    { value: 'Monthly', label: 'Месечен отчет' },
    { value: 'Annual', label: 'Годишен отчет' },
    { value: 'OnDemand', label: 'При поискване' }
  ];

  const taxAccountingBasisOptions = [
    { value: 'A', label: 'Търговски предприятия (A)' },
    { value: 'P', label: 'Бюджетни предприятия (P)' },
    { value: 'BANK', label: 'Банки' },
    { value: 'INSURANCE', label: 'Застрахователи' }
  ];

  const months = [
    { value: 1, label: 'Януари' },
    { value: 2, label: 'Февруари' },
    { value: 3, label: 'Март' },
    { value: 4, label: 'Април' },
    { value: 5, label: 'Май' },
    { value: 6, label: 'Юни' },
    { value: 7, label: 'Юли' },
    { value: 8, label: 'Август' },
    { value: 9, label: 'Септември' },
    { value: 10, label: 'Октомври' },
    { value: 11, label: 'Ноември' },
    { value: 12, label: 'Декември' }
  ];

  const currentYear = new Date().getFullYear();
  const years = Array.from({ length: 10 }, (_, i) => currentYear - i);

  const selectedCompany = companies.find(c => c.id === formData.companyId);

  return (
    <div className="space-y-6">
      <div className="bg-white shadow-sm rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <h1 className="text-2xl font-semibold text-gray-900">
            SAF-T Export
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            Експорт на данни в стандартен одитен файл за данъци (SAF-T) за България
          </p>
        </div>

        <div className="p-6">
          <form className="space-y-6">
            {/* Company Selection */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Компания
              </label>
              <select
                value={formData.companyId || ''}
                onChange={(e) => handleInputChange('companyId', parseInt(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">Изберете компания</option>
                {companies.map((company) => (
                  <option key={company.id} value={company.id}>
                    {company.name} ({company.eik})
                  </option>
                ))}
              </select>
              {selectedCompany && (
                <p className="mt-1 text-sm text-gray-500">
                  ЕИК: {selectedCompany.eik}
                  {selectedCompany.vatNumber && ` • ДДС: ${selectedCompany.vatNumber}`}
                </p>
              )}
            </div>

            {/* Period Selection */}
            <div className="border border-gray-200 rounded-lg p-4">
              <h3 className="text-lg font-medium text-gray-900 mb-4">Период за експорт</h3>
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Начален месец
                  </label>
                  <select
                    value={formData.periodStart}
                    onChange={(e) => handleInputChange('periodStart', parseInt(e.target.value))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  >
                    {months.map((month) => (
                      <option key={month.value} value={month.value}>
                        {month.label}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Начална година
                  </label>
                  <select
                    value={formData.periodStartYear}
                    onChange={(e) => handleInputChange('periodStartYear', parseInt(e.target.value))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  >
                    {years.map((year) => (
                      <option key={year} value={year}>
                        {year}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Краен месец
                  </label>
                  <select
                    value={formData.periodEnd}
                    onChange={(e) => handleInputChange('periodEnd', parseInt(e.target.value))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  >
                    {months.map((month) => (
                      <option key={month.value} value={month.value}>
                        {month.label}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Крайна година
                  </label>
                  <select
                    value={formData.periodEndYear}
                    onChange={(e) => handleInputChange('periodEndYear', parseInt(e.target.value))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  >
                    {years.map((year) => (
                      <option key={year} value={year}>
                        {year}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            </div>

            {/* File Type and Tax Accounting Basis */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Тип файл
                </label>
                <select
                  value={formData.fileType}
                  onChange={(e) => handleInputChange('fileType', e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                >
                  {fileTypeOptions.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Тип счетоводство
                </label>
                <select
                  value={formData.taxAccountingBasis}
                  onChange={(e) => handleInputChange('taxAccountingBasis', e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                >
                  {taxAccountingBasisOptions.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Information about selected file type */}
            {formData.fileType && (
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <h3 className="text-sm font-medium text-blue-900 mb-2">
                  Информация за типа файл: {fileTypeOptions.find(opt => opt.value === formData.fileType)?.label}
                </h3>
                <p className="text-sm text-blue-800">
                  {formData.fileType === 'Monthly' && 
                    'Месечен отчет включва основни данни, главна книга и документи за избрания период.'
                  }
                  {formData.fileType === 'Annual' && 
                    'Годишен отчет включва информация за собственост и активи.'
                  }
                  {formData.fileType === 'OnDemand' && 
                    'При поискване включва данни за стоки и складови наличности.'
                  }
                </p>
              </div>
            )}
          </form>
        </div>
      </div>

      {/* Validation Results */}
      {validationResult && (
        <div className={`bg-white shadow-sm rounded-lg border-l-4 ${
          validationResult.isValid ? 'border-green-400' : 'border-red-400'
        }`}>
          <div className="p-6">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                {validationResult.isValid ? (
                  <svg className="h-5 w-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                ) : (
                  <svg className="h-5 w-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
                  </svg>
                )}
              </div>
              <div className="ml-3 flex-1">
                <h3 className={`text-sm font-medium ${
                  validationResult.isValid ? 'text-green-800' : 'text-red-800'
                }`}>
                  {validationResult.isValid ? 'Валидацията премина успешно' : 'Грешки при валидация'}
                </h3>
                
                {validationResult.isValid ? (
                  <div className="mt-2 text-sm text-green-700">
                    <p>Брой транзакции: {validationResult.numberOfTransactions?.toLocaleString()}</p>
                    <p>Очакван размер на файла: {formatFileSize(validationResult.fileSizeBytes)}</p>
                  </div>
                ) : (
                  <div className="mt-2 text-sm text-red-700">
                    <ul className="list-disc list-inside space-y-1">
                      {validationResult.validationErrors?.map((error, index) => (
                        <li key={index}>{error}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Export Result */}
      {exportResult && (
        <div className={`bg-white shadow-sm rounded-lg border-l-4 ${
          exportResult.success ? 'border-green-400' : 'border-red-400'
        }`}>
          <div className="p-6">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                {exportResult.success ? (
                  <svg className="h-5 w-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                ) : (
                  <svg className="h-5 w-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
                  </svg>
                )}
              </div>
              <div className="ml-3">
                <p className={`text-sm ${
                  exportResult.success ? 'text-green-800' : 'text-red-800'
                }`}>
                  {exportResult.message}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Action Buttons */}
      <div className="bg-white shadow-sm rounded-lg">
        <div className="px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <button
                type="button"
                onClick={handleValidate}
                disabled={!formData.companyId}
                className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Валидирай
              </button>
            </div>
            
            <div>
              <button
                type="button"
                onClick={handleExport}
                disabled={!validationResult?.isValid || isExporting}
                className="px-6 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isExporting ? (
                  <span className="flex items-center">
                    <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Експортиране...
                  </span>
                ) : 'Експортирай SAF-T'}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Information */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="h-5 w-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-blue-800">Информация за SAF-T</h3>
            <div className="mt-2 text-sm text-blue-700">
              <p>
                SAF-T (Standard Audit File for Tax) е международен стандарт за електронни одитни файлове, 
                който улеснява данъчните проверки и анализи.
              </p>
              <p className="mt-2">
                Експортираният файл съдържа структурирана информация за счетоводните записи, 
                сметкоплана, контрагентите и други данни за избрания период.
              </p>
              <p className="mt-2">
                <strong>Забележка:</strong> SAF-T за България ще стане задължителен от 2026 г. 
                за големите предприятия съгласно европейското законодателство.
              </p>
              <p className="mt-2 text-xs">
                <strong>Статус на интеграцията:</strong> Интерфейсът е готов, но все още не е свързан с backend API. 
                В момента използва демонстрационни данни.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}