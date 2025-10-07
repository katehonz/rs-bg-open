// Service layer for contragent AI utilities backed by the GraphQL API

import { graphqlRequest } from '../utils/graphqlClient';

const VALIDATE_VAT_MUTATION = `
  mutation ValidateVat($vatNumber: String!) {
    validateVat(vatNumber: $vatNumber) {
      contragent {
        vatNumber
        eik
        companyName
        companyNameBg
        address
        longAddress
        streetName
        city
        postalCode
        country
        vatValid
        eikValid
        valid
      }
      existedInDatabase
      source
    }
  }
`;

const addressCache = new Map();

// Validate VAT number through backend GraphQL (VIES + Mistral enrichment)
export const validateVatNumber = async (vatNumber) => {
  try {
    const response = await graphqlRequest(VALIDATE_VAT_MUTATION, { vatNumber });
    const payload = response?.validateVat;

    if (!payload) {
      throw new Error('Липсва отговор от сървъра');
    }

    const contragent = payload.contragent || {};
    const companyName = contragent.companyName || contragent.companyNameBg || null;
    const address = contragent.address || contragent.longAddress || null;
    const streetName = contragent.streetName || '';
    const city = contragent.city || null;
    const postalCode = contragent.postalCode || null;
    const country = contragent.country || null;

    if (address) {
      addressCache.set(address, {
        street: streetName,
        city,
        postalCode,
        country,
      });
    }

    return {
      valid: !!contragent.valid,
      vatValid: !!contragent.vatValid,
      eikValid: !!contragent.eikValid,
      vatNumber: contragent.vatNumber || vatNumber,
      eik: contragent.eik || null,
      companyName,
      address,
      streetName,
      city,
      postalCode,
      country,
      existedInDatabase: !!payload.existedInDatabase,
      source: payload.source,
      contragent,
    };
  } catch (error) {
    console.error('Error validating VAT number via GraphQL:', error);
    throw error;
  }
};

// Resolve address components from the latest validation (kept async for caller parity)
export const parseAddress = async (validationResultOrAddress) => {
  const contragent = validationResultOrAddress?.contragent || validationResultOrAddress;
  const directStreet = contragent?.streetName || contragent?.street || '';
  const directCity = contragent?.city || null;
  const directPostal = contragent?.postalCode || null;
  const directCountry = contragent?.country || null;

  if (directStreet || directCity || directPostal || directCountry) {
    return {
      street: directStreet,
      number: contragent?.number || '',
      city: directCity,
      postalCode: directPostal,
      country: directCountry,
    };
  }

  if (typeof validationResultOrAddress === 'string') {
    const cached = addressCache.get(validationResultOrAddress);
    if (cached) {
      return {
        street: cached.street || '',
        number: '',
        city: cached.city,
        postalCode: cached.postalCode,
        country: cached.country,
      };
    }
  }

  return {
    street: '',
    number: '',
    city: null,
    postalCode: null,
    country: null,
  };
};

// Batch validate multiple VAT numbers (for import)
export const batchValidateVatNumbers = async (vatNumbers) => {
  const results = [];

  for (const vatNumber of vatNumbers) {
    try {
      const result = await validateVatNumber(vatNumber);
      results.push({ vatNumber, success: true, data: result });
    } catch (error) {
      results.push({ vatNumber, success: false, error: error.message });
    }
  }

  return { results };
};

// Check if VIES validation is enabled
export const isViesValidationEnabled = () => {
  return localStorage.getItem('enableViesValidation') === 'true';
};

// Check if AI mapping is enabled
export const isAiMappingEnabled = () => {
  return localStorage.getItem('enableAiMapping') === 'true';
};

// Check if auto-validation on import is enabled
export const isAutoValidateOnImportEnabled = () => {
  return localStorage.getItem('autoValidateOnImport') === 'true';
};
