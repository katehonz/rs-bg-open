import { useState } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

export default function AverageCostButton({ accountId, quantity, onCalculated }) {
  const [loading, setLoading] = useState(false);

  const AVERAGE_COST_QUERY = `
    query GetAverageCost($companyId: Int!, $accountId: Int!) {
      getAverageCost(companyId: $companyId, accountId: $accountId) {
        averageCost
        currentQuantity
        currentAmount
      }
    }
  `;

  const handleCalculate = async () => {
    if (!accountId || !quantity || quantity <= 0) {
      alert('Моля въведете валидно количество');
      return;
    }

    try {
      setLoading(true);
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;

      const response = await graphqlRequest(AVERAGE_COST_QUERY, {
        companyId,
        accountId: parseInt(accountId)
      });

      const averageCost = parseFloat(response.getAverageCost.averageCost);
      const totalValue = averageCost * parseFloat(quantity);

      if (onCalculated) {
        onCalculated(totalValue.toFixed(2));
      }
    } catch (err) {
      alert('Грешка при изчисление на СПЦ: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <button
      type="button"
      onClick={handleCalculate}
      disabled={loading}
      className="px-3 py-1 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50 text-sm"
      title="Изчисли средно претеглена цена"
    >
      {loading ? '⏳' : '💰 СПЦ'}
    </button>
  );
}
