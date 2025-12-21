import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import AccountSelectModal from '../../components/AccountSelectModal';

// GraphQL query for accounts
const ACCOUNTS_QUERY = `
  query GetAccounts($companyId: Int!) {
    accountHierarchy(companyId: $companyId) {
      id
      code
      name
      accountType
      isAnalytical
      isActive
    }
  }
`;

// GraphQL query for technology cards
const TECHNOLOGY_CARDS_QUERY = `
  query GetTechnologyCards($companyId: Int!) {
    technologyCards(companyId: $companyId) {
      id
      companyId
      name
      description
      outputUnit
      isActive
      createdAt
      updatedAt
    }
  }
`;

// GraphQL query for single technology card with stages
const TECHNOLOGY_CARD_QUERY = `
  query GetTechnologyCard($id: Int!) {
    technologyCard(id: $id) {
      card {
        id
        companyId
        name
        description
        outputUnit
        isActive
        createdAt
        updatedAt
      }
      stages {
        id
        technologyCardId
        stageNumber
        name
        debitAccountId
        creditAccountId
        quantityFormula
        unitOfMeasure
        amountFormula
        description
        createdAt
      }
    }
  }
`;

// GraphQL mutation for creating technology card with stages
const CREATE_TECHNOLOGY_CARD_MUTATION = `
  mutation CreateTechnologyCardWithStages($input: CreateTechnologyCardWithStagesInput!) {
    createTechnologyCardWithStages(input: $input) {
      card {
        id
        companyId
        name
        description
        outputUnit
        isActive
        createdAt
        updatedAt
      }
      stages {
        id
        technologyCardId
        stageNumber
        name
        debitAccountId
        creditAccountId
        quantityFormula
        unitOfMeasure
        amountFormula
        description
        createdAt
      }
    }
  }
`;

// GraphQL mutation for updating technology card
const UPDATE_TECHNOLOGY_CARD_MUTATION = `
  mutation UpdateTechnologyCard($input: UpdateTechnologyCardInput!) {
    updateTechnologyCard(input: $input) {
      id
      companyId
      name
      description
      outputUnit
      isActive
      createdAt
      updatedAt
    }
  }
`;

// GraphQL mutation for updating technology card stages
const UPDATE_TECHNOLOGY_CARD_STAGES_MUTATION = `
  mutation UpdateTechnologyCardStages($technologyCardId: Int!, $stages: [StageInput!]!) {
    updateTechnologyCardStages(technologyCardId: $technologyCardId, stages: $stages) {
      id
      technologyCardId
      stageNumber
      name
      debitAccountId
      creditAccountId
      quantityFormula
      unitOfMeasure
      amountFormula
      description
      createdAt
    }
  }
`;

// GraphQL mutation for deleting technology card
const DELETE_TECHNOLOGY_CARD_MUTATION = `
  mutation DeleteTechnologyCard($id: Int!) {
    deleteTechnologyCard(id: $id)
  }
`;

const graphqlRequest = async (query, variables = {}) => {
  const token = localStorage.getItem('authToken');
  const headers = {
    'Content-Type': 'application/json',
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch('/graphql', {
    method: 'POST',
    headers,
    body: JSON.stringify({ query, variables }),
  });

  const result = await response.json();
  if (result.errors) {
    console.error('GraphQL Errors:', result.errors);
    throw new Error(result.errors[0].message);
  }
  return result.data;
};

export default function TechnologyCards() {
  const [cards, setCards] = useState([]);
  const [showModal, setShowModal] = useState(false);
  const [editingCard, setEditingCard] = useState(null);
  const [accounts, setAccounts] = useState([]);
  const [showDebitAccountModal, setShowDebitAccountModal] = useState(false);
  const [showCreditAccountModal, setShowCreditAccountModal] = useState(false);
  const [activeStageIndex, setActiveStageIndex] = useState(null);
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);

  // Form state
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    outputUnit: 'кг',
    stages: []
  });

  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch technology cards
        console.log('Fetching technology cards for companyId:', companyId);
        const cardsData = await graphqlRequest(TECHNOLOGY_CARDS_QUERY, { companyId });
        console.log('Fetched technology cards:', cardsData.technologyCards);
        setCards(cardsData.technologyCards || []);

        // Fetch accounts
        console.log('Fetching accounts for companyId:', companyId);
        const accountsData = await graphqlRequest(ACCOUNTS_QUERY, { companyId });
        console.log('Fetched accounts:', accountsData.accountHierarchy);
        setAccounts(accountsData.accountHierarchy || []);
      } catch (error) {
        console.error('Error fetching data:', error);
        alert('Грешка при зареждане на данни: ' + error.message);
        setCards([]);
        setAccounts([]);
      }
    };

    fetchData();
  }, [companyId]);

  const handleCreate = () => {
    setEditingCard(null);
    setFormData({
      name: '',
      description: '',
      outputUnit: 'кг',
      stages: [
        {
          stageNumber: 1,
          name: '',
          debitAccountId: '',
          creditAccountId: '',
          quantityFormula: 'input_quantity',
          unitOfMeasure: 'кг',
          amountFormula: '',
          description: ''
        }
      ]
    });
    setShowModal(true);
  };

  const handleEdit = async (card) => {
    try {
      // Fetch full card details with stages
      console.log('Fetching card details for id:', card.id);
      const data = await graphqlRequest(TECHNOLOGY_CARD_QUERY, { id: card.id });
      const cardData = data.technologyCard;

      setEditingCard(cardData.card);
      setFormData({
        name: cardData.card.name,
        description: cardData.card.description || '',
        outputUnit: cardData.card.outputUnit,
        stages: cardData.stages.map(stage => ({
          stageNumber: stage.stageNumber,
          name: stage.name,
          debitAccountId: stage.debitAccountId.toString(),
          creditAccountId: stage.creditAccountId.toString(),
          quantityFormula: stage.quantityFormula || 'input_quantity',
          unitOfMeasure: stage.unitOfMeasure || 'кг',
          amountFormula: stage.amountFormula || '',
          description: stage.description || ''
        }))
      });
      setShowModal(true);
    } catch (error) {
      console.error('Error loading card details:', error);
      alert('Грешка при зареждане на картата: ' + error.message);
    }
  };

  const handleAddStage = () => {
    setFormData({
      ...formData,
      stages: [
        ...formData.stages,
        {
          stageNumber: formData.stages.length + 1,
          name: '',
          debitAccountId: '',
          creditAccountId: '',
          quantityFormula: 'input_quantity',
          unitOfMeasure: 'кг',
          amountFormula: '',
          description: ''
        }
      ]
    });
  };

  const handleRemoveStage = (index) => {
    const newStages = formData.stages.filter((_, i) => i !== index);
    // Renumber stages
    newStages.forEach((stage, i) => {
      stage.stageNumber = i + 1;
    });
    setFormData({ ...formData, stages: newStages });
  };

  const handleStageChange = (index, field, value) => {
    const newStages = [...formData.stages];
    newStages[index] = { ...newStages[index], [field]: value };
    setFormData({ ...formData, stages: newStages });
  };

  const handleSave = async () => {
    try {
      console.log('Saving technology card:', formData);

      if (editingCard) {
        // Update existing card
        const updateCardInput = {
          id: editingCard.id,
          name: formData.name,
          description: formData.description,
          outputUnit: formData.outputUnit
        };

        await graphqlRequest(UPDATE_TECHNOLOGY_CARD_MUTATION, { input: updateCardInput });

        // Update stages
        const stagesInput = formData.stages.map(stage => ({
          stageNumber: stage.stageNumber,
          name: stage.name,
          debitAccountId: parseInt(stage.debitAccountId),
          creditAccountId: parseInt(stage.creditAccountId),
          quantityFormula: stage.quantityFormula || null,
          unitOfMeasure: stage.unitOfMeasure || null,
          amountFormula: stage.amountFormula || null,
          description: stage.description || null
        }));

        await graphqlRequest(UPDATE_TECHNOLOGY_CARD_STAGES_MUTATION, {
          technologyCardId: editingCard.id,
          stages: stagesInput
        });

        alert('Технологичната карта беше обновена успешно!');
      } else {
        // Create new card
        const createInput = {
          companyId,
          name: formData.name,
          description: formData.description || null,
          outputUnit: formData.outputUnit,
          stages: formData.stages.map(stage => ({
            stageNumber: stage.stageNumber,
            name: stage.name,
            debitAccountId: parseInt(stage.debitAccountId),
            creditAccountId: parseInt(stage.creditAccountId),
            quantityFormula: stage.quantityFormula || null,
            unitOfMeasure: stage.unitOfMeasure || null,
            amountFormula: stage.amountFormula || null,
            description: stage.description || null
          }))
        };

        await graphqlRequest(CREATE_TECHNOLOGY_CARD_MUTATION, { input: createInput });
        alert('Технологичната карта беше създадена успешно!');
      }

      // Refresh cards list
      const cardsData = await graphqlRequest(TECHNOLOGY_CARDS_QUERY, { companyId });
      setCards(cardsData.technologyCards || []);

      setShowModal(false);
    } catch (error) {
      console.error('Error saving technology card:', error);
      alert('Грешка при запазване: ' + error.message);
    }
  };

  const handleDelete = async (card) => {
    if (!window.confirm(`Сигурни ли сте, че искате да изтриете технологичната карта "${card.name}"?`)) {
      return;
    }

    try {
      console.log('Deleting technology card:', card.id);
      await graphqlRequest(DELETE_TECHNOLOGY_CARD_MUTATION, { id: card.id });
      alert('Технологичната карта беше изтрита успешно!');

      // Refresh cards list
      const cardsData = await graphqlRequest(TECHNOLOGY_CARDS_QUERY, { companyId });
      setCards(cardsData.technologyCards || []);
    } catch (error) {
      console.error('Error deleting technology card:', error);
      alert('Грешка при изтриване: ' + error.message);
    }
  };

  const getAccountLabel = (accountId) => {
    const account = accounts.find(a => a.id === parseInt(accountId));
    return account ? `${account.code} - ${account.name}` : '';
  };

  return (
    <div className="container mx-auto px-4 py-6">
      {/* Page Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Производство</h1>
        <p className="text-sm text-gray-600">
          Производствено счетоводство с технологични карти и автоматични операции
        </p>
      </div>

      {/* Tabs Navigation */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="-mb-px flex space-x-8">
          <Link
            to="/production/technology-cards"
            className="border-blue-500 text-blue-600 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Технологични карти
          </Link>
          <Link
            to="/production/batches"
            className="border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Производствени партиди
          </Link>
          <Link
            to="/production/reports"
            className="border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Справки
          </Link>
        </nav>
      </div>

      {/* Section Header */}
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-semibold text-gray-900">Технологични карти</h2>
        <button
          onClick={handleCreate}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          + Нова технологична карта
        </button>
      </div>

      {/* Cards List */}
      <div className="bg-white shadow-sm rounded-lg overflow-hidden">
        {cards.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <p className="mb-2">Няма създадени технологични карти</p>
            <p className="text-sm">Създайте първата си технологична карта за производствен процес</p>
          </div>
        ) : (
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Наименование
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Описание
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Ед. мярка
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Етапи
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Статус
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Действия
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {cards.map((card) => (
                <tr key={card.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                    {card.name}
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-500">
                    {card.description}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {card.outputUnit}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {card.stages?.length || 0} етапа
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                      card.isActive ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                    }`}>
                      {card.isActive ? 'Активна' : 'Неактивна'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <button
                      onClick={() => handleEdit(card)}
                      className="text-blue-600 hover:text-blue-900 mr-3"
                    >
                      Редактирай
                    </button>
                    <button
                      onClick={() => handleDelete(card)}
                      className="text-red-600 hover:text-red-900"
                    >
                      Изтрий
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-10 mx-auto p-5 border w-11/12 max-w-5xl shadow-lg rounded-md bg-white mb-10">
            {/* Modal Header */}
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold text-gray-900">
                {editingCard ? 'Редактиране на технологична карта' : 'Нова технологична карта'}
              </h3>
              <button
                onClick={() => setShowModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Basic Info */}
            <div className="mb-6 p-4 bg-gray-50 rounded-lg">
              <h4 className="text-sm font-medium text-gray-700 mb-3">Основна информация</h4>
              <div className="grid grid-cols-3 gap-4">
                <div className="col-span-2">
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Наименование *
                  </label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                    placeholder="Напр. Производство на хляб"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Ед. мярка на продукт *
                  </label>
                  <input
                    type="text"
                    value={formData.outputUnit}
                    onChange={(e) => setFormData({ ...formData, outputUnit: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                    placeholder="кг, бр, л"
                  />
                </div>
                <div className="col-span-3">
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Описание
                  </label>
                  <textarea
                    value={formData.description}
                    onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                    rows={2}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                    placeholder="Кратко описание на технологичния процес"
                  />
                </div>
              </div>
            </div>

            {/* Stages */}
            <div className="mb-6">
              <div className="flex justify-between items-center mb-3">
                <h4 className="text-sm font-medium text-gray-700">Етапи на производство</h4>
                <button
                  onClick={handleAddStage}
                  className="px-3 py-1 text-sm bg-green-600 text-white rounded-md hover:bg-green-700"
                >
                  + Добави етап
                </button>
              </div>

              {formData.stages.length === 0 ? (
                <div className="p-4 bg-gray-50 rounded-lg text-center text-gray-500 text-sm">
                  Добавете първия етап на производствения процес
                </div>
              ) : (
                <div className="space-y-4">
                  {formData.stages.map((stage, index) => (
                    <div key={index} className="p-4 border border-gray-200 rounded-lg bg-white">
                      <div className="flex justify-between items-center mb-3">
                        <h5 className="text-sm font-medium text-gray-800">
                          Етап {stage.stageNumber}
                        </h5>
                        <button
                          onClick={() => handleRemoveStage(index)}
                          className="text-red-600 hover:text-red-800 text-sm"
                        >
                          Изтрий
                        </button>
                      </div>

                      <div className="grid grid-cols-2 gap-3">
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Наименование на етап *
                          </label>
                          <input
                            type="text"
                            value={stage.name}
                            onChange={(e) => handleStageChange(index, 'name', e.target.value)}
                            className="w-full px-2 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                            placeholder="Напр. Изписване на материали"
                          />
                        </div>

                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Ед. мярка
                          </label>
                          <input
                            type="text"
                            value={stage.unitOfMeasure}
                            onChange={(e) => handleStageChange(index, 'unitOfMeasure', e.target.value)}
                            className="w-full px-2 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                            placeholder="кг, бр, л"
                          />
                        </div>

                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Дебит сметка *
                          </label>
                          <button
                            type="button"
                            onClick={() => {
                              setActiveStageIndex(index);
                              setShowDebitAccountModal(true);
                            }}
                            className="w-full px-2 py-1.5 text-sm border border-gray-300 rounded-md text-left hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                          >
                            {stage.debitAccountId ? (
                              <div className="flex items-center space-x-2">
                                <span className="font-mono font-semibold text-gray-900">
                                  {accounts.find(a => a.id === parseInt(stage.debitAccountId))?.code}
                                </span>
                                <span className="text-gray-700">
                                  {accounts.find(a => a.id === parseInt(stage.debitAccountId))?.name}
                                </span>
                              </div>
                            ) : (
                              <span className="text-gray-500">Изберете сметка...</span>
                            )}
                          </button>
                        </div>

                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Кредит сметка *
                          </label>
                          <button
                            type="button"
                            onClick={() => {
                              setActiveStageIndex(index);
                              setShowCreditAccountModal(true);
                            }}
                            className="w-full px-2 py-1.5 text-sm border border-gray-300 rounded-md text-left hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                          >
                            {stage.creditAccountId ? (
                              <div className="flex items-center space-x-2">
                                <span className="font-mono font-semibold text-gray-900">
                                  {accounts.find(a => a.id === parseInt(stage.creditAccountId))?.code}
                                </span>
                                <span className="text-gray-700">
                                  {accounts.find(a => a.id === parseInt(stage.creditAccountId))?.name}
                                </span>
                              </div>
                            ) : (
                              <span className="text-gray-500">Изберете сметка...</span>
                            )}
                          </button>
                        </div>

                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Формула за количество
                          </label>
                          <input
                            type="text"
                            value={stage.quantityFormula}
                            onChange={(e) => handleStageChange(index, 'quantityFormula', e.target.value)}
                            className="w-full px-2 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 font-mono"
                            placeholder="input_quantity или input_quantity * 1.05"
                          />
                        </div>

                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Формула за сума
                          </label>
                          <input
                            type="text"
                            value={stage.amountFormula}
                            onChange={(e) => handleStageChange(index, 'amountFormula', e.target.value)}
                            className="w-full px-2 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 font-mono"
                            placeholder="quantity * 5.50 или previous_stage_amount"
                          />
                        </div>

                        <div className="col-span-2">
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Описание на етап
                          </label>
                          <textarea
                            value={stage.description}
                            onChange={(e) => handleStageChange(index, 'description', e.target.value)}
                            rows={2}
                            className="w-full px-2 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                            placeholder="Допълнителна информация за етапа"
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Modal Footer */}
            <div className="flex justify-end space-x-3 pt-4 border-t">
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
              >
                Отказ
              </button>
              <button
                onClick={handleSave}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                Запази
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Account Selection Modals */}
      <AccountSelectModal
        show={showDebitAccountModal}
        accounts={accounts}
        currentAccountId={activeStageIndex !== null ? parseInt(formData.stages[activeStageIndex]?.debitAccountId) : null}
        title="Избор на дебитна сметка"
        onSelect={(account) => {
          if (activeStageIndex !== null) {
            handleStageChange(activeStageIndex, 'debitAccountId', account.id.toString());
          }
          setShowDebitAccountModal(false);
        }}
        onClose={() => setShowDebitAccountModal(false)}
      />

      <AccountSelectModal
        show={showCreditAccountModal}
        accounts={accounts}
        currentAccountId={activeStageIndex !== null ? parseInt(formData.stages[activeStageIndex]?.creditAccountId) : null}
        title="Избор на кредитна сметка"
        onSelect={(account) => {
          if (activeStageIndex !== null) {
            handleStageChange(activeStageIndex, 'creditAccountId', account.id.toString());
          }
          setShowCreditAccountModal(false);
        }}
        onClose={() => setShowCreditAccountModal(false)}
      />
    </div>
  );
}
