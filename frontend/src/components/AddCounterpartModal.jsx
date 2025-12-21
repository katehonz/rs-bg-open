import { useState, useEffect, useRef } from 'react';
import {
  validateVatNumber,
  parseAddress,
  isViesValidationEnabled,
  isAiMappingEnabled
} from '../services/contragentApi';

export default function AddCounterpartModal({ 
  show, 
  onSave, 
  onClose 
}) {
  const [formData, setFormData] = useState({
    name: '',
    eik: '',
    vatNumber: '',
    isVatRegistered: false,
    address: '',
    street: '',
    city: '',
    postalCode: '',
    country: 'България'
  });
  const [errors, setErrors] = useState({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [vatValidationResult, setVatValidationResult] = useState(null);
  const [validatingVat, setValidatingVat] = useState(false);
  const [parsingAddress, setParsingAddress] = useState(false);
  const nameInputRef = useRef(null);
  const vatValidationTimeoutRef = useRef(null);

  const viesEnabled = isViesValidationEnabled();
  const aiEnabled = isAiMappingEnabled();

  // Focus name input when modal opens
  useEffect(() => {
    if (show && nameInputRef.current) {
      nameInputRef.current.focus();
    }
  }, [show]);

  // Reset form when modal opens
  useEffect(() => {
    if (show) {
      setFormData({
        name: '',
        eik: '',
        vatNumber: '',
        isVatRegistered: false,
        address: '',
        street: '',
        city: '',
        postalCode: '',
        country: 'България'
      });
      setErrors({});
      setIsSubmitting(false);
      setVatValidationResult(null);
      setValidatingVat(false);
      setParsingAddress(false);
      if (vatValidationTimeoutRef.current) {
        clearTimeout(vatValidationTimeoutRef.current);
        vatValidationTimeoutRef.current = null;
      }
    }
  }, [show]);

  useEffect(() => {
    return () => {
      if (vatValidationTimeoutRef.current) {
        clearTimeout(vatValidationTimeoutRef.current);
      }
    };
  }, []);

  const validateForm = () => {
    const newErrors = {};

    // Name validation
    if (!formData.name.trim()) {
      newErrors.name = 'Името е задължително';
    } else if (formData.name.length < 2) {
      newErrors.name = 'Името трябва да е поне 2 символа';
    }

    // BULSTAT validation
    if (formData.eik) {
      const cleanBulstat = formData.eik.replace(/\D/g, '');
      if (cleanBulstat.length !== 9 && cleanBulstat.length !== 13) {
        newErrors.eik = 'БУЛСТАТ трябва да е 9 или 13 цифри';
      }
    }

    // VAT number validation
    if (formData.isVatRegistered) {
      if (!formData.vatNumber.trim()) {
        newErrors.vatNumber = 'ДДС номерът е задължителен за ДДС регистрирани лица';
      } else {
        const cleanVatNumber = formData.vatNumber.replace(/\D/g, '');
        if (formData.country === 'България' && cleanVatNumber.length !== 9) {
          newErrors.vatNumber = 'ДДС номерът в България трябва да е 9 цифри';
        }
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleInputChange = (field, value) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    
    // Clear field error when user starts typing
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }));
    }
  };

  const handleBulstatChange = (value) => {
    // Allow only digits and limit to 13 characters
    const cleanValue = value.replace(/\D/g, '').slice(0, 13);
    handleInputChange('eik', cleanValue);
  };

  const handleVatNumberChange = (value) => {
    const normalized = value.toUpperCase().replace(/\s+/g, '').slice(0, 20);

    setFormData(prev => ({
      ...prev,
      vatNumber: normalized,
      isVatRegistered: prev.isVatRegistered || normalized.length > 0
    }));

    if (errors.vatNumber) {
      setErrors(prev => ({ ...prev, vatNumber: '' }));
    }

    if (vatValidationResult) {
      setVatValidationResult(null);
    }

    if (vatValidationTimeoutRef.current) {
      clearTimeout(vatValidationTimeoutRef.current);
      vatValidationTimeoutRef.current = null;
    }

    if (
      viesEnabled &&
      normalized.length >= 10 &&
      /^[A-Z]{2}[A-Z0-9]+$/.test(normalized)
    ) {
      vatValidationTimeoutRef.current = setTimeout(() => {
        handleVatValidation(normalized);
      }, 1200);
    }
  };

  const handleVatRegisteredChange = (isRegistered) => {
    handleInputChange('isVatRegistered', isRegistered);
    if (!isRegistered) {
      handleInputChange('vatNumber', '');
      setVatValidationResult(null);
      if (vatValidationTimeoutRef.current) {
        clearTimeout(vatValidationTimeoutRef.current);
        vatValidationTimeoutRef.current = null;
      }
    }
  };

  const handleVatValidation = async (vatNumberOverride) => {
    if (!viesEnabled) {
      return;
    }

    const vat = (vatNumberOverride || formData.vatNumber || '')
      .toUpperCase()
      .replace(/\s+/g, '');

    if (!vat) {
      return;
    }

    setValidatingVat(true);
    setVatValidationResult(null);

    try {
      const result = await validateVatNumber(vat);
      setVatValidationResult(result);

      if (result?.valid) {
        const fallbackAddress =
          result.address ||
          result.streetName ||
          result.contragent?.address ||
          result.contragent?.longAddress ||
          '';

        setFormData(prev => ({
          ...prev,
          name: result.companyName || prev.name,
          eik: result.eik || prev.eik,
          vatNumber: result.vatNumber || vat,
          isVatRegistered: true,
          address: fallbackAddress || prev.address,
          street: result.streetName || prev.street,
          city: result.city || result.contragent?.city || prev.city,
          postalCode: result.postalCode || result.contragent?.postalCode || prev.postalCode,
          country: result.country || result.contragent?.country || prev.country
        }));

        if (aiEnabled) {
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

  const handleAddressParsing = async (source) => {
    if (!aiEnabled) {
      return;
    }

    const addressSource = source || formData.address;
    if (!addressSource) {
      return;
    }

    setParsingAddress(true);

    try {
      const parsed = await parseAddress(addressSource);
      const street = parsed?.street ? parsed.street.trim() : '';
      const number = parsed?.number ? parsed.number.trim() : '';
      const combinedStreet = [street, number].filter(Boolean).join(' ');

      setFormData(prev => ({
        ...prev,
        street: combinedStreet || prev.street,
        address:
          typeof addressSource === 'string'
            ? addressSource
            : prev.address || combinedStreet || prev.address,
        city: parsed?.city || prev.city,
        postalCode: parsed?.postalCode || prev.postalCode,
        country: parsed?.country || prev.country
      }));
    } catch (error) {
      console.error('Error parsing address:', error);
    } finally {
      setParsingAddress(false);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }

    setIsSubmitting(true);
    
    try {
      const normalizedVat = formData.vatNumber ? formData.vatNumber.trim() : '';

      // Clean up the data
      const cleanData = {
        ...formData,
        name: formData.name.trim(),
        eik: formData.eik ? formData.eik.trim() : null,
        vatNumber: formData.isVatRegistered || normalizedVat ? normalizedVat || null : null,
        isVatRegistered: formData.isVatRegistered || Boolean(normalizedVat),
        address: formData.address ? formData.address.trim() || null : null,
        street: formData.street ? formData.street.trim() || null : null,
        city: formData.city ? formData.city.trim() || null : null,
        postalCode: formData.postalCode ? formData.postalCode.trim() || null : null,
        country: formData.country ? formData.country.trim() : 'България'
      };

      await onSave(cleanData);
      handleClose();
    } catch (error) {
      console.error('Error saving counterpart:', error);
      setErrors({ general: 'Грешка при запазване на контрагента' });
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleClose = () => {
    setFormData({
      name: '',
      eik: '',
      vatNumber: '',
      isVatRegistered: false,
      address: '',
      street: '',
      city: '',
      postalCode: '',
      country: 'България'
    });
    setErrors({});
    setIsSubmitting(false);
    setVatValidationResult(null);
    setValidatingVat(false);
    setParsingAddress(false);
    if (vatValidationTimeoutRef.current) {
      clearTimeout(vatValidationTimeoutRef.current);
      vatValidationTimeoutRef.current = null;
    }
    onClose();
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Escape' && !isSubmitting) {
      handleClose();
    }
  };

  if (!show) return null;

  return (
    <div 
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onClick={!isSubmitting ? handleClose : undefined}
      onKeyDown={handleKeyDown}
    >
      <div 
        className="bg-white rounded-lg w-full max-w-2xl max-h-[90vh] flex flex-col shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h3 className="text-xl font-semibold text-gray-900">Добави нов контрагент</h3>
          <button
            onClick={handleClose}
            disabled={isSubmitting}
            className="text-gray-400 hover:text-gray-600 transition-colors disabled:opacity-50"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto p-6 space-y-4">
          {errors.general && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <div className="text-sm text-red-800">{errors.general}</div>
            </div>
          )}

          {viesEnabled && vatValidationResult && (
            <div
              className={`rounded-lg p-4 text-sm ${
                vatValidationResult.valid
                  ? 'bg-green-50 border border-green-200 text-green-800'
                  : 'bg-red-50 border border-red-200 text-red-800'
              }`}
            >
              <div>
                {vatValidationResult.valid
                  ? '✅ VIES: Валиден ДДС номер'
                  : '❌ VIES: Невалиден ДДС номер'}
              </div>
              {vatValidationResult.companyName && (
                <div className="mt-1 text-green-700">
                  Официално име: {vatValidationResult.companyName}
                </div>
              )}
              {vatValidationResult.address && (
                <div className="text-green-700">
                  Регистриран адрес: {vatValidationResult.address}
                </div>
              )}
              {vatValidationResult.error && (
                <div className="text-red-700">
                  {vatValidationResult.error}
                </div>
              )}
            </div>
          )}

          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Име на контрагента *
            </label>
            <input
              ref={nameInputRef}
              type="text"
              value={formData.name}
              onChange={(e) => handleInputChange('name', e.target.value)}
              className={`w-full px-3 py-2 border rounded-md text-sm ${
                errors.name ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : 'border-gray-300 focus:ring-blue-500 focus:border-blue-500'
              }`}
              placeholder="Въведете името на контрагента"
              disabled={isSubmitting}
              required
            />
            {errors.name && (
              <p className="text-red-600 text-xs mt-1">{errors.name}</p>
            )}
          </div>

          {/* BULSTAT */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              БУЛСТАТ
            </label>
            <input
              type="text"
              value={formData.eik}
              onChange={(e) => handleBulstatChange(e.target.value)}
              className={`w-full px-3 py-2 border rounded-md text-sm font-mono ${
                errors.eik ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : 'border-gray-300 focus:ring-blue-500 focus:border-blue-500'
              }`}
              placeholder="123456789"
              maxLength="13"
              disabled={isSubmitting}
            />
            {errors.eik && (
              <p className="text-red-600 text-xs mt-1">{errors.eik}</p>
            )}
            <p className="text-gray-500 text-xs mt-1">9 цифри за ЕИК, 13 цифри за БУЛСТАТ</p>
          </div>

          {/* VAT Registration */}
          <div>
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="isVatRegistered"
                checked={formData.isVatRegistered}
                onChange={(e) => handleVatRegisteredChange(e.target.checked)}
                className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                disabled={isSubmitting}
              />
              <label htmlFor="isVatRegistered" className="text-sm font-medium text-gray-700">
                ДДС регистриран
              </label>
            </div>
          </div>

          {/* VAT Number */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              ДДС номер
              {viesEnabled && (
                <span className="text-xs text-blue-600 ml-1">(автоматична VIES валидация)</span>
              )}
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={formData.vatNumber}
                onChange={(e) => handleVatNumberChange(e.target.value)}
                className={`mt-1 flex-1 px-3 py-2 border rounded-md text-sm font-mono ${
                  errors.vatNumber ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : 'border-gray-300 focus:ring-blue-500 focus:border-blue-500'
                }`}
                placeholder="BG123456789"
                disabled={isSubmitting}
                autoComplete="off"
                required={formData.isVatRegistered}
              />
              {viesEnabled && (
                <button
                  type="button"
                  onClick={() => handleVatValidation()}
                  disabled={isSubmitting || validatingVat || !formData.vatNumber}
                  className="mt-1 px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 text-sm whitespace-nowrap font-medium"
                >
                  {validatingVat ? '🔄 Извличане...' : '📥 Извлечи от VIES'}
                </button>
              )}
            </div>
            <p className="text-gray-500 text-xs mt-1">
              Въведете ДДС номер с код на държава (напр. BG123456789)
              {viesEnabled && (
                <span className="text-green-600 font-medium">
                  {' '}– автоматично попълване и AI мапване
                </span>
              )}
            </p>
            {errors.vatNumber && (
              <p className="text-red-600 text-xs mt-1">{errors.vatNumber}</p>
            )}
          </div>

          {/* Address */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Адрес
              {aiEnabled && (
                <span className="text-xs text-purple-600 ml-1">(AI мапване)</span>
              )}
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={formData.address}
                onChange={(e) => handleInputChange('address', e.target.value)}
                className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-blue-500 focus:border-blue-500"
                placeholder="ул. Иван Вазов 10, София"
                disabled={isSubmitting}
              />
              {aiEnabled && formData.address && (
                <button
                  type="button"
                  onClick={() => handleAddressParsing(formData.address)}
                  disabled={isSubmitting || parsingAddress}
                  className="mt-1 px-3 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:opacity-50 text-sm whitespace-nowrap"
                >
                  {parsingAddress ? '🔄 AI мапва...' : '🤖 AI мапване'}
                </button>
              )}
            </div>
            <p className="text-gray-500 text-xs mt-1">
              AI разделя адреса на компоненти (улица, град, пощенски код).
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Улица / булевард и номер
            </label>
            <input
              type="text"
              value={formData.street}
              onChange={(e) => handleInputChange('street', e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-blue-500 focus:border-blue-500"
              placeholder="ул. Витоша 10"
              disabled={isSubmitting}
            />
          </div>

          {/* City and Country */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Град
              </label>
              <input
                type="text"
                value={formData.city}
                onChange={(e) => handleInputChange('city', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-blue-500 focus:border-blue-500"
                placeholder="Въведете града"
                disabled={isSubmitting}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Пощенски код
              </label>
              <input
                type="text"
                value={formData.postalCode}
                onChange={(e) => handleInputChange('postalCode', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-blue-500 focus:border-blue-500"
                placeholder="1000"
                disabled={isSubmitting}
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Държава
              </label>
              <select
                value={formData.country}
                onChange={(e) => handleInputChange('country', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-blue-500 focus:border-blue-500"
                disabled={isSubmitting}
              >
                <option value="България">България</option>
                <option value="Германия">Германия</option>
                <option value="Гърция">Гърция</option>
                <option value="Румъния">Румъния</option>
                <option value="Сърбия">Сърбия</option>
                <option value="Турция">Турция</option>
                <option value="Други">Други</option>
              </select>
            </div>
          </div>
        </form>

        {/* Footer */}
        <div className="flex items-center justify-end space-x-3 p-6 border-t border-gray-200">
          <button
            type="button"
            onClick={handleClose}
            disabled={isSubmitting}
            className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Отказ
          </button>
          <button
            type="button"
            onClick={handleSubmit}
            disabled={isSubmitting}
            className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? (
              <>
                <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white inline" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Запазва...
              </>
            ) : (
              'Запази'
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
