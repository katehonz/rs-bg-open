import { createContext, useContext, useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

const CompanyContext = createContext(null);

export const useCompany = () => {
  const context = useContext(CompanyContext);
  if (!context) {
    throw new Error('useCompany must be used within CompanyProvider');
  }
  return context;
};

const COMPANY_QUERY = `
  query GetCompany($id: Int!) {
    company(id: $id) {
      id
      name
      eik
      vatNumber
      baseCurrencyId
      baseCurrency {
        id
        code
        name
        nameBg
      }
    }
  }
`;

export const CompanyProvider = ({ children }) => {
  const [company, setCompany] = useState(null);
  const [baseCurrency, setBaseCurrency] = useState('BGN'); // Default to BGN
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadCompany = async () => {
      try {
        const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

        const response = await graphqlRequest(COMPANY_QUERY, { id: companyId });

        if (response.company) {
          setCompany(response.company);
          setBaseCurrency(response.company.baseCurrency?.code || 'BGN');
        }
      } catch (error) {
        console.error('Error loading company:', error);
        // Fallback to BGN on error
        setBaseCurrency('BGN');
      } finally {
        setLoading(false);
      }
    };

    loadCompany();
  }, []);

  const value = {
    company,
    baseCurrency,
    loading,
  };

  return <CompanyContext.Provider value={value}>{children}</CompanyContext.Provider>;
};
