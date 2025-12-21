import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

export default function FixedAssetsDebug() {
  const [stats, setStats] = useState({
    totalAssets: 0,
    totalAcquisitionCost: 0,
    totalBookValue: 0,
    activeAssets: 0
  });
  const [error, setError] = useState(null);
  const [loading, setLoading] = useState(true);

  const STATS_QUERY = `
    query GetFixedAssetsStats($companyId: Int!) {
      fixedAssetsSummary(companyId: $companyId) {
        totalAssets
        totalAcquisitionCost
        totalAccountingBookValue
        totalTaxBookValue
        totalAccumulatedDepreciation
        activeAssets
        disposedAssets
      }
    }
  `;

  useEffect(() => {
    loadStats();
  }, []);

  const loadStats = async () => {
    try {
      setLoading(true);
      setError(null);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      console.log('Loading stats for company:', companyId);
      
      const response = await graphqlRequest(STATS_QUERY, { companyId });
      console.log('GraphQL response:', response);
      
      if (response.fixedAssetsSummary) {
        setStats({
          totalAssets: response.fixedAssetsSummary.totalAssets,
          totalAcquisitionCost: response.fixedAssetsSummary.totalAcquisitionCost,
          totalBookValue: response.fixedAssetsSummary.totalAccountingBookValue,
          activeAssets: response.fixedAssetsSummary.activeAssets
        });
      }
    } catch (err) {
      console.error('Error loading stats:', err);
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="p-6">
        <h1>Loading Fixed Assets...</h1>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <h1 className="text-red-600">Error Loading Fixed Assets</h1>
        <p className="text-sm text-gray-600">{error}</p>
        <button 
          onClick={() => loadStats()} 
          className="mt-4 px-4 py-2 bg-blue-500 text-white rounded"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Fixed Assets Debug</h1>
      <div className="bg-white rounded-lg shadow p-4">
        <h2 className="text-lg font-semibold mb-2">Stats</h2>
        <div className="grid grid-cols-2 gap-4">
          <div>Total Assets: {stats.totalAssets}</div>
          <div>Acquisition Cost: {stats.totalAcquisitionCost}</div>
          <div>Book Value: {stats.totalBookValue}</div>
          <div>Active Assets: {stats.activeAssets}</div>
        </div>
      </div>
    </div>
  );
}